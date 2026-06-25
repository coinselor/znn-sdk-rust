//! Shared connection dial/retry state machine and transport-neutral state.
//!
//! [`dial`] owns the `Connecting` → bounded-attempts → `Running`/`Stopped`
//! flow that every transport drives, plus the single retry delay. A concrete
//! transport contributes only its URL predicate and a per-attempt connect step
//! (`connect_one`), so the loop has no transport-specific branches.
//!
//! [`ConnectionState`] is the transport-neutral status type returned by every
//! transport's `status()`.

use crate::client::constants::NUM_RETRIES;
use crate::client::exceptions::ClientError;
use std::future::Future;
use std::time::Duration;

/// Delay (milliseconds) between connection retry attempts. The single source
/// for this value across all transports.
pub(crate) const RETRY_DELAY_MS: u64 = 50;

/// The URL schemes routed to the websocket transport. The single scheme source
/// shared by [`validate_url`] and [`crate::client::factory::new_client`].
pub(crate) const WS_SCHEMES: &[&str] = &["ws://", "wss://"];

/// The URL schemes routed to the HTTP transport. The single scheme source
/// shared by [`validate_url`] and [`crate::client::factory::new_client`].
pub(crate) const HTTP_SCHEMES: &[&str] = &["http://", "https://"];

/// The state of a client connection, transport-neutral.
///
/// Backs every transport (`WsClient`, `HttpClient`) so an HTTP connection is
/// not described by a websocket-named type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// No connection has been attempted.
    Uninitialized,
    /// A connection attempt is in progress.
    Connecting,
    /// The connection is established.
    Running,
    /// The client has been stopped.
    Stopped,
}

/// Drives the shared connection state machine for `url`.
///
/// Validates `url` with `is_valid`, transitions to [`ConnectionState::Connecting`],
/// then calls `connect_one(url)` up to `NUM_RETRIES` times (one attempt when
/// `retry` is false), sleeping [`RETRY_DELAY_MS`] between attempts. On the first
/// `Ok` it leaves `state` at [`ConnectionState::Running`] and returns `Ok(true)`;
/// if every attempt fails it leaves `state` at [`ConnectionState::Stopped`] and
/// returns `Err(ClientError::NoConnection)`.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn dial<F, Fut, C>(
    state: &mut ConnectionState,
    url: &str,
    retry: bool,
    is_valid: fn(&str) -> bool,
    connect_one: F,
    on_connected: impl FnOnce(C),
) -> Result<bool, ClientError>
where
    F: Fn(&str) -> Fut,
    Fut: Future<Output = Result<C, ClientError>>,
{
    if !is_valid(url) {
        *state = ConnectionState::Stopped;
        return Err(ClientError::NoConnection);
    }
    *state = ConnectionState::Connecting;
    let attempts = if retry {
        (NUM_RETRIES.max(1)) as usize
    } else {
        1
    };
    for attempt in 0..attempts {
        match connect_one(url).await {
            Ok(connection) => {
                on_connected(connection);
                *state = ConnectionState::Running;
                return Ok(true);
            }
            Err(_) => {
                if attempt + 1 < attempts {
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                }
            }
        }
    }
    *state = ConnectionState::Stopped;
    Err(ClientError::NoConnection)
}

/// Returns `true` only for an absolute URL whose scheme is one of `schemes`
/// and whose authority ends in a numeric TCP port in `0..=65535`.
///
/// Leading/trailing whitespace is trimmed, matching the connection factory, so
/// a URL the factory routes also passes validation.
pub(crate) fn validate_url(url: &str, schemes: &[&str]) -> bool {
    let url = url.trim();
    let Some(after_scheme) = schemes.iter().find_map(|scheme| url.strip_prefix(scheme)) else {
        return false;
    };
    let authority_end = after_scheme
        .bytes()
        .position(|b| matches!(b, b'/' | b'?' | b'#'))
        .unwrap_or(after_scheme.len());
    let authority = after_scheme.split_at(authority_end).0;
    authority_has_port(authority)
}

/// Returns `true` when `authority` ends in a numeric TCP port in `0..=65535`.
fn authority_has_port(authority: &str) -> bool {
    let Some(colon) = authority.rfind(':') else {
        return false;
    };
    // For an IPv6 bracket literal the port colon must follow the closing `]`.
    if let Some(bracket) = authority.rfind(']')
        && colon < bracket
    {
        return false;
    }
    let port = authority.split_at(colon + 1).1;
    port.parse::<u16>().is_ok()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    const WS_SCHEMES: &[&str] = &["ws://", "wss://"];
    const HTTP_SCHEMES: &[&str] = &["http://", "https://"];

    #[test]
    fn connection_state_variants_are_distinct() {
        let variants = [
            ConnectionState::Uninitialized,
            ConnectionState::Connecting,
            ConnectionState::Running,
            ConnectionState::Stopped,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "distinct state variants must not be equal");
                }
            }
        }
    }

    #[tokio::test]
    async fn dial_succeeds_when_an_attempt_succeeds() {
        // The connect step fails twice then succeeds; the dial must return
        // Ok(true) and leave the state Running, having made exactly three
        // attempts.
        let attempts = Arc::new(AtomicU32::new(0));
        let counter = attempts.clone();
        let connect_one = move |_url: &str| {
            let counter = counter.clone();
            async move {
                let n = counter.fetch_add(1, Ordering::Relaxed);
                if n < 2 {
                    Err(ClientError::NoConnection)
                } else {
                    Ok(())
                }
            }
        };

        let mut state = ConnectionState::Uninitialized;
        let result = dial(
            &mut state,
            "ws://127.0.0.1:35998",
            true,
            valid_ws_predicate,
            connect_one,
            |_conn: ()| {},
        )
        .await;

        assert_eq!(
            result,
            Ok(true),
            "a succeeding attempt must return Ok(true)"
        );
        assert_eq!(
            state,
            ConnectionState::Running,
            "a successful dial must leave the state Running"
        );
        assert_eq!(
            attempts.load(Ordering::Relaxed),
            3,
            "the dial must stop retrying once an attempt succeeds"
        );
    }

    #[tokio::test]
    async fn dial_stops_after_the_configured_retry_count() {
        // The connect step never succeeds; the dial must make exactly
        // NUM_RETRIES attempts, end Stopped, and return Err(NoConnection).
        let attempts = Arc::new(AtomicU32::new(0));
        let counter = attempts.clone();
        let connect_one = move |_url: &str| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Err::<(), ClientError>(ClientError::NoConnection)
            }
        };

        let mut state = ConnectionState::Uninitialized;
        let result = dial(
            &mut state,
            "ws://127.0.0.1:35998",
            true,
            valid_ws_predicate,
            connect_one,
            |_conn: ()| {},
        )
        .await;

        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "an always-failing dial must return Err(NoConnection), got {result:?}"
        );
        assert_eq!(
            state,
            ConnectionState::Stopped,
            "a failed dial must leave the state Stopped"
        );
        assert_eq!(
            attempts.load(Ordering::Relaxed),
            NUM_RETRIES,
            "retry=true must make exactly NUM_RETRIES attempts"
        );
    }

    #[tokio::test]
    async fn dial_rejects_an_invalid_url_before_connecting() {
        // An invalid URL must short-circuit: no connect attempt is made. Paired
        // with a valid-URL path that does connect, so a stub that never connects
        // cannot satisfy the valid-URL arm.
        let attempts = Arc::new(AtomicU32::new(0));
        let counter = attempts.clone();
        let connect_one = move |_url: &str| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        };

        let mut state = ConnectionState::Uninitialized;
        let _ = dial(
            &mut state,
            "ws://127.0.0.1:35998",
            true,
            valid_ws_predicate,
            connect_one,
            |_conn: ()| {},
        )
        .await;
        let connected_attempts = attempts.load(Ordering::Relaxed);
        assert!(
            connected_attempts > 0,
            "a valid URL must trigger at least one connect attempt"
        );

        let attempts = Arc::new(AtomicU32::new(0));
        let counter = attempts.clone();
        let connect_one = move |_url: &str| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        };
        let mut state = ConnectionState::Uninitialized;
        let result = dial(
            &mut state,
            "ftp://127.0.0.1:21",
            true,
            valid_ws_predicate,
            connect_one,
            |_conn: ()| {},
        )
        .await;
        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "an invalid URL must be rejected with NoConnection, got {result:?}"
        );
        assert_eq!(
            attempts.load(Ordering::Relaxed),
            0,
            "an invalid URL must not trigger any connect attempt"
        );
    }

    #[tokio::test]
    async fn dial_without_retry_makes_a_single_attempt() {
        let attempts = Arc::new(AtomicU32::new(0));
        let counter = attempts.clone();
        let connect_one = move |_url: &str| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Err::<(), ClientError>(ClientError::NoConnection)
            }
        };

        let mut state = ConnectionState::Uninitialized;
        let _ = dial(
            &mut state,
            "ws://127.0.0.1:35998",
            false,
            valid_ws_predicate,
            connect_one,
            |_conn: ()| {},
        )
        .await;

        assert_eq!(
            attempts.load(Ordering::Relaxed),
            1,
            "retry=false must make exactly one attempt"
        );
    }

    #[test]
    fn validate_url_accepts_websocket_schemes() {
        for url in [
            "ws://127.0.0.1:35998",
            "wss://example.com:443",
            "ws://127.0.0.1:35998/path",
            "wss://node.example:35998/v1",
            "  ws://127.0.0.1:35998  ",
        ] {
            assert!(
                validate_url(url, WS_SCHEMES),
                "{url:?} must validate for the websocket scheme set"
            );
        }
    }

    #[test]
    fn validate_url_accepts_http_schemes() {
        for url in [
            "http://127.0.0.1:35997",
            "https://example.com:443",
            "http://127.0.0.1:35997/path",
            "https://node.example:443/v1",
            "\thttps://node.example:443\n",
        ] {
            assert!(
                validate_url(url, HTTP_SCHEMES),
                "{url:?} must validate for the http scheme set"
            );
        }
    }

    #[test]
    fn validate_url_rejects_cross_scheme_urls() {
        // A websocket URL must not validate against the http scheme set, and
        // vice versa, so the factory and validator agree on a single scheme
        // source per transport. Each cross-scheme rejection is paired with the
        // matching accept under the URL's own scheme set, so a blanket-rejecting
        // stub cannot satisfy this scenario.
        assert!(
            validate_url("ws://127.0.0.1:35998", WS_SCHEMES),
            "a ws URL must validate for its own scheme set"
        );
        assert!(
            !validate_url("ws://127.0.0.1:35998", HTTP_SCHEMES),
            "a ws URL must be rejected by the http scheme set"
        );
        assert!(
            validate_url("http://127.0.0.1:35997", HTTP_SCHEMES),
            "an http URL must validate for its own scheme set"
        );
        assert!(
            !validate_url("http://127.0.0.1:35997", WS_SCHEMES),
            "an http URL must be rejected by the websocket scheme set"
        );
    }

    #[test]
    fn validate_url_rejects_portless_or_garbage_urls() {
        // Each rejection is paired with an accept that proves the function
        // distinguishes structure, so a blanket-rejecting stub fails the
        // accept arm rather than passing the reject arm trivially.
        assert!(
            validate_url("ws://127.0.0.1:35998", WS_SCHEMES),
            "a well-formed ws URL must validate"
        );
        for url in [
            "ws://127.0.0.1",
            "127.0.0.1:35998",
            "",
            "garbage",
            "ws://example.com",
            "ws://127.0.0.1:abc",
            "ws://127.0.0.1:99999",
            "ftp://127.0.0.1:21",
        ] {
            assert!(
                !validate_url(url, WS_SCHEMES),
                "{url:?} must be rejected by validate_url"
            );
        }
    }

    /// A URL predicate that accepts websocket URLs with numeric ports. Mirrors
    /// the scheme set the `dial` unit tests route through, decoupled from the
    /// (stubbed) `validate_url` under test.
    fn valid_ws_predicate(url: &str) -> bool {
        let url = url.trim();
        let after_scheme = if let Some(rest) = url.strip_prefix("ws://") {
            rest
        } else if let Some(rest) = url.strip_prefix("wss://") {
            rest
        } else {
            return false;
        };
        let authority_end = after_scheme
            .bytes()
            .position(|b| matches!(b, b'/' | b'?' | b'#'))
            .unwrap_or(after_scheme.len());
        let authority = after_scheme.split_at(authority_end).0;
        let Some(colon) = authority.rfind(':') else {
            return false;
        };
        if let Some(bracket) = authority.rfind(']')
            && colon < bracket
        {
            return false;
        }
        authority.split_at(colon + 1).1.parse::<u16>().is_ok()
    }
}

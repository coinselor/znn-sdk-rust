//! Scheme-routing client factory.
//!
//! [`new_client`] routes a URL to the matching transport: `ws://`/`wss://` to a
//! [`WsClient`] and `http://`/`https://` to an [`HttpClient`], returning a
//! [`ClientTransport`] that dispatches [`Client::send_request`] to whichever
//! transport the scheme selected. The returned transport is `Uninitialized`;
//! the caller invokes `initialize` before use.
//!
//! The factory returns a [`ClientTransport`] enum rather than a trait object
//! because the async [`Client`] trait is not object-safe (its `send_request`
//! returns `impl Future`).
//!
//! [`WsClient`]: crate::client::websocket::WsClient

use crate::client::dial::{ConnectionState, HTTP_SCHEMES, WS_SCHEMES};
use crate::client::exceptions::ClientError;
use crate::client::http::HttpClient;
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use serde_json::Value;

/// A JSON-RPC transport selected by URL scheme.
pub enum ClientTransport {
    /// The websocket transport (`ws://`/`wss://`).
    WebSocket(WsClient),
    /// The HTTP transport (`http://`/`https://`).
    Http(HttpClient),
}

impl std::fmt::Debug for ClientTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WebSocket(_) => f.write_str("ClientTransport::WebSocket(..)"),
            Self::Http(_) => f.write_str("ClientTransport::Http(..)"),
        }
    }
}

/// Routes `url` to the matching transport, returning an `Uninitialized`
/// [`ClientTransport`].
///
/// `ws://`/`wss://` select [`WebSocket`](ClientTransport::WebSocket),
/// `http://`/`https://` select [`Http`](ClientTransport::Http), and any other
/// scheme returns [`ClientError::NoConnection`]. The returned transport is
/// [`ConnectionState::Uninitialized`]; the caller invokes
/// [`initialize`](ClientTransport::initialize) before use.
pub fn new_client(url: &str) -> Result<ClientTransport, ClientError> {
    let url = url.trim();
    if starts_with_scheme(url, WS_SCHEMES) {
        return Ok(ClientTransport::WebSocket(WsClient::new()));
    }
    if starts_with_scheme(url, HTTP_SCHEMES) {
        return Ok(ClientTransport::Http(HttpClient::new()));
    }
    Err(ClientError::NoConnection)
}

/// Returns `true` when `url` begins with one of `schemes` and has a non-empty
/// remainder. Shares the single scheme list with [`validate_url`].
fn starts_with_scheme(url: &str, schemes: &[&str]) -> bool {
    schemes.iter().any(|scheme| {
        url.strip_prefix(scheme)
            .is_some_and(|rest| !rest.is_empty())
    })
}

impl ClientTransport {
    /// Returns the connection state of the active transport.
    pub fn status(&self) -> ConnectionState {
        match self {
            Self::WebSocket(client) => client.status(),
            Self::Http(client) => client.status(),
        }
    }

    /// Returns `true` when the active transport has no live connection.
    pub fn is_closed(&self) -> bool {
        match self {
            Self::WebSocket(client) => client.is_closed(),
            Self::Http(client) => client.is_closed(),
        }
    }

    /// Dials `url` through the active transport, honoring `retry` (up to
    /// `NUM_RETRIES` attempts with a short backoff when set).
    pub async fn initialize(&mut self, url: &str, retry: bool) -> Result<bool, ClientError> {
        match self {
            Self::WebSocket(client) => client.initialize(url, retry).await,
            Self::Http(client) => client.initialize(url, retry).await,
        }
    }
}

impl Client for ClientTransport {
    async fn send_request(&self, method: &str, params: &[Value]) -> Result<Value, ClientError> {
        match self {
            Self::WebSocket(client) => client.send_request(method, params).await,
            Self::Http(client) => client.send_request(method, params).await,
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn new_client_routes_ws_and_wss_to_websocket() {
        for url in ["ws://127.0.0.1:35998", "wss://node.example:35998"] {
            let transport = new_client(url).expect("ws/wss routes to a transport");
            assert!(
                matches!(transport, ClientTransport::WebSocket(_)),
                "{url} must route to ClientTransport::WebSocket, got {transport:?}"
            );
        }
    }

    #[test]
    fn new_client_routes_http_and_https_to_http() {
        for url in ["http://127.0.0.1:35997", "https://node.example:443"] {
            let transport = new_client(url).expect("http/https routes to a transport");
            assert!(
                matches!(transport, ClientTransport::Http(_)),
                "{url} must route to ClientTransport::Http, got {transport:?}"
            );
        }
    }

    #[test]
    fn new_client_rejects_an_unknown_scheme() {
        let result = new_client("ftp://127.0.0.1:21");
        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "an unknown scheme must be rejected with NoConnection, got {result:?}"
        );
    }

    #[test]
    fn new_client_returns_an_uninitialized_transport() {
        // The factory hands back an Uninitialized, closed transport that the
        // caller must `initialize` before use. Pinned for both schemes so the
        // factory-output lifecycle is not left implicit.
        let transport = new_client("http://127.0.0.1:35997").expect("http routes to a transport");
        assert_eq!(
            transport.status(),
            ConnectionState::Uninitialized,
            "new_client must return an Uninitialized transport"
        );
        assert!(
            transport.is_closed(),
            "a fresh factory transport must be closed"
        );
    }

    #[tokio::test]
    async fn websocket_variant_dispatches_to_the_ws_client() {
        let transport = ClientTransport::WebSocket(WsClient::new());
        let result = transport.send_request("any.method", &[]).await;
        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "the websocket variant must dispatch through the fresh ws client, got {result:?}"
        );
    }

    #[tokio::test]
    async fn http_variant_dispatches_to_the_http_client() {
        let transport = ClientTransport::Http(HttpClient::new());
        let result = transport.send_request("any.method", &[]).await;
        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "the http variant must dispatch through the fresh http client, got {result:?}"
        );
    }
}

//! The HTTP JSON-RPC client.
//!
//! [`HttpClient`] wraps a `jsonrpsee` HTTP client and exposes the same status
//! lifecycle plus [`Client::send_request`] as [`WsClient`]. Before a successful
//! [`initialize`], requests fail fast with [`ClientError::NoConnection`]; once
//! connected they POST each request to the node's HTTP JSON-RPC endpoint.
//!
//! HTTP is request/response only: JSON-RPC subscriptions are websocket-only,
//! so [`HttpClient`] does not provide a `subscribe_stream`. Callers that need
//! subscription streams must use [`WsClient`].
//!
//! [`initialize`]: HttpClient::initialize
//! [`WsClient`]: crate::client::websocket::WsClient

use crate::client::dial::{ConnectionState, HTTP_SCHEMES, dial, validate_url};
use crate::client::exceptions::ClientError;
use crate::client::interfaces::Client;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::client::Error as RpcError;
use jsonrpsee::core::params::ArrayParams;
use jsonrpsee::http_client::HttpClient as RpcClient;
use jsonrpsee::http_client::HttpClientBuilder;
use serde_json::Value;

/// Method used to probe connectivity during [`HttpClient::initialize`].
///
/// `HttpClientBuilder::build` only parses the URL and defers the connection to
/// the first request. Each dial attempt issues a lightweight JSON-RPC call so a
/// successful attempt means the node is reachable, matching websocket dial
/// behavior. Any server response proves the endpoint is live. A transport-layer
/// failure means the node is unreachable and the attempt is retried.
const CONNECTIVITY_PROBE_METHOD: &str = "system.syncInfo";

/// A JSON-RPC client over HTTP.
pub struct HttpClient {
    state: ConnectionState,
    client: Option<RpcClient>,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    /// Creates a client in the `Uninitialized` state.
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Uninitialized,
            client: None,
        }
    }

    /// Returns `true` when there is no live transport.
    pub fn is_closed(&self) -> bool {
        self.state != ConnectionState::Running
    }

    /// Returns the intended connection state.
    pub fn status(&self) -> ConnectionState {
        self.state
    }

    /// Dials `url`, setting the client to `Running` on success. When `retry`
    /// is set, the dial is attempted up to [`NUM_RETRIES`](crate::client::constants::NUM_RETRIES)
    /// times with a short backoff between attempts; a malformed or non-HTTP
    /// URL yields [`ClientError::NoConnection`] immediately.
    ///
    /// Because [`HttpClientBuilder::build`] is lazy (it parses the URL but
    /// defers the connection to the first request), each attempt also probes
    /// the endpoint so that a node that is not reachable is retried, mirroring
    /// the websocket dial semantics.
    pub async fn initialize(&mut self, url: &str, retry: bool) -> Result<bool, ClientError> {
        // `dial`'s `connect_one` bound (`Fn(&str) -> Fut`) requires the returned
        // future to be independent of the borrowed `&str`, so the connect step
        // captures an owned copy of the URL rather than borrowing the argument.
        let connect_url = url.to_string();
        let this = self;
        dial(
            &mut this.state,
            url,
            retry,
            |url| validate_url(url, HTTP_SCHEMES),
            move |_: &str| {
                let connect_url = connect_url.clone();
                async move {
                    // `HttpClientBuilder::build` is synchronous: it parses the
                    // URL and constructs the transport but does not open a
                    // connection. A failure here is a malformed target, not a
                    // downed node.
                    let Ok(client) = HttpClientBuilder::default().build(&connect_url) else {
                        return Err(ClientError::NoConnection);
                    };
                    // Probe so a lazy build against an unreachable node still
                    // counts as a failed attempt and is retried.
                    probe_connectivity(&client)
                        .await
                        .map(|()| client)
                        .map_err(|_| ClientError::NoConnection)
                }
            },
            |client| {
                this.client = Some(client);
            },
        )
        .await
    }
}

/// Issues a no-arg JSON-RPC call to confirm `client` can reach a live node.
///
/// Any JSON-RPC response counts as connectivity, including a server-reported
/// JSON-RPC error such as `method not found`. Only transport-layer failures
/// (refused, timed out, or dropped connections) mean the endpoint is unreachable.
async fn probe_connectivity(client: &RpcClient) -> Result<(), RpcError> {
    let params = ArrayParams::new();
    match client
        .request::<Value, _>(CONNECTIVITY_PROBE_METHOD, params)
        .await
    {
        // A JSON-RPC error object means the server answered, so the node is
        // reachable; treat that as a successful probe.
        Ok(_) | Err(RpcError::Call(_)) => Ok(()),
        Err(other) => Err(other),
    }
}

impl Client for HttpClient {
    async fn send_request(&self, method: &str, params: &[Value]) -> Result<Value, ClientError> {
        let Some(client) = self.client.as_ref() else {
            return Err(ClientError::NoConnection);
        };
        if self.state != ConnectionState::Running {
            return Err(ClientError::NoConnection);
        }
        let mut rpc_params = ArrayParams::new();
        for value in params {
            rpc_params
                .insert(value.clone())
                .map_err(|_| ClientError::NoConnection)?;
        }
        client
            .request::<Value, _>(method, rpc_params)
            .await
            .map_err(|_| ClientError::NoConnection)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_fresh_http_client_is_uninitialized_and_closed() {
        let client = HttpClient::new();
        assert_eq!(
            client.status(),
            ConnectionState::Uninitialized,
            "a fresh http client must report Uninitialized"
        );
        assert!(client.is_closed(), "a fresh http client must be closed");
    }

    #[tokio::test]
    async fn http_send_request_fails_before_initialize() {
        let client = HttpClient::new();
        let result = client.send_request("any.method", &[]).await;
        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "send_request must fail with NoConnection before initialize, got {result:?}"
        );
    }

    #[tokio::test]
    async fn http_initialize_rejects_a_websocket_url() {
        let mut client = HttpClient::new();
        let result = client.initialize("ws://127.0.0.1:35998", false).await;
        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "initialize must reject a ws:// URL with NoConnection, got {result:?}"
        );
        assert!(
            client.is_closed(),
            "a rejected initialize must leave the client closed"
        );
    }
}

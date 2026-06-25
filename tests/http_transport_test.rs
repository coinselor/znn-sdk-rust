//! HTTP JSON-RPC transport integration tests.
//!
//! Covers `HttpClient`'s connected round-trip over a mock HTTP JSON-RPC server
//! and the `new_client` scheme-routing factory. The mock server (jsonrpsee
//! `server` feature) answers JSON-RPC over HTTP POST on the same port it serves
//! WebSocket, so these tests dial it with an `http://` URL.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use serde_json::{Value, json};
use support::MockNode;
use znn_sdk_rust::client::ConnectionState;
use znn_sdk_rust::client::exceptions::ClientError;
use znn_sdk_rust::client::http::HttpClient;
use znn_sdk_rust::client::interfaces::Client;
use znn_sdk_rust::{ClientTransport, new_client};

/// A connected `HttpClient` reports `Running`, is not closed, and round-trips
/// the server's JSON result for the requested method.
#[tokio::test]
async fn http_client_round_trips_through_a_mock_server() {
    let node = MockNode::spawn(|module| {
        module
            .register_method("ledger.getNetworkInfo", |_params, _ctx, _ext| {
                let response: jsonrpsee::core::RpcResult<Value> = Ok(json!({ "ok": true }));
                response
            })
            .expect("mock method registers");
    })
    .await;
    // The jsonrpsee server serves JSON-RPC over HTTP at the same address; swap
    // the `ws://` scheme for `http://`.
    let http_url = node.url.replacen("ws://", "http://", 1);

    let mut client = HttpClient::new();
    let connected = client
        .initialize(&http_url, false)
        .await
        .expect("initialize connects to the mock http server");
    assert!(connected, "initialize must report a successful connection");
    assert_eq!(client.status(), ConnectionState::Running);
    assert!(!client.is_closed(), "a connected http client is not closed");

    let result = client
        .send_request("ledger.getNetworkInfo", &[])
        .await
        .expect("send_request round-trips the mock response");
    assert_eq!(result, json!({ "ok": true }));
}

/// A factory-built `ClientTransport` drives the full lifecycle without being
/// destructured: `initialize` reaches `Running`, it is no longer closed, and
/// `send_request` round-trips through the HTTP variant. Pins the
/// "`ClientTransport` exposes the transport lifecycle" requirement, which is
/// otherwise only asserted by inspection on the pre-connect path.
#[tokio::test]
async fn a_factory_built_transport_initializes_through_client_transport() {
    let node = MockNode::spawn(|module| {
        module
            .register_method("ledger.getNetworkInfo", |_params, _ctx, _ext| {
                let response: jsonrpsee::core::RpcResult<Value> = Ok(json!({ "ok": true }));
                response
            })
            .expect("mock method registers");
    })
    .await;
    let http_url = node.url.replacen("ws://", "http://", 1);

    let mut transport = new_client(&http_url).expect("http routes to a transport");
    assert!(
        matches!(transport, ClientTransport::Http(_)),
        "an http:// URL must route to the Http variant, got {transport:?}"
    );

    let connected = transport
        .initialize(&http_url, false)
        .await
        .expect("initialize connects through ClientTransport");
    assert!(connected, "initialize must report a successful connection");
    assert_eq!(transport.status(), ConnectionState::Running);
    assert!(
        !transport.is_closed(),
        "an initialized ClientTransport must not be closed"
    );

    let result = transport
        .send_request("ledger.getNetworkInfo", &[])
        .await
        .expect("send_request round-trips through ClientTransport");
    assert_eq!(result, json!({ "ok": true }));
}

/// `new_client` routes an `http://` URL to the HTTP transport variant.
#[test]
fn new_client_routes_http_urls_to_the_http_transport() {
    let transport = new_client("http://127.0.0.1:35997").expect("http routes to a transport");
    assert!(
        matches!(transport, ClientTransport::Http(_)),
        "an http:// URL must route to ClientTransport::Http"
    );
}

/// `new_client` routes a `ws://` URL to the websocket transport variant.
#[test]
fn new_client_routes_ws_urls_to_the_websocket_transport() {
    let transport = new_client("ws://127.0.0.1:35998").expect("ws routes to a transport");
    assert!(
        matches!(transport, ClientTransport::WebSocket(_)),
        "a ws:// URL must route to ClientTransport::WebSocket"
    );
}

/// `new_client` rejects an unknown scheme with `ClientError::NoConnection`.
#[test]
fn new_client_rejects_an_unknown_scheme() {
    let result = new_client("ftp://127.0.0.1:21");
    assert!(
        matches!(result, Err(ClientError::NoConnection)),
        "an unknown scheme must be rejected, got {result:?}"
    );
}

/// `initialize(url, true)` makes more than one dial attempt against a node
/// that accepts-and-drops connections, and ultimately fails with
/// `NoConnection`. Mirrors the WebSocket retry test
/// (`initialize_with_retry_makes_multiple_attempts`). Pins the retry branch of
/// `initialize`, the one piece of dial logic distinct from validation.
#[tokio::test]
async fn http_initialize_with_retry_makes_multiple_attempts() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Accept and drop each connection so the HTTP request fails and
    // `initialize` retries. Each accept records one dial attempt.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("binds a free port");
    let addr = listener.local_addr().expect("reports its address");
    let attempts = Arc::new(AtomicU32::new(0));
    let counter = attempts.clone();
    let server = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            counter.fetch_add(1, Ordering::Relaxed);
            drop(stream);
        }
    });

    let mut client = HttpClient::new();
    let result = client.initialize(&format!("http://{addr}"), true).await;
    server.abort();

    assert!(
        matches!(result, Err(ClientError::NoConnection)),
        "a retried dial that never connects must fail with NoConnection, got {result:?}"
    );
    assert!(
        attempts.load(Ordering::Relaxed) > 1,
        "retry=true must make more than one dial attempt, saw {}",
        attempts.load(Ordering::Relaxed)
    );
    assert!(
        client.is_closed(),
        "a failed dial leaves the http client closed"
    );
}

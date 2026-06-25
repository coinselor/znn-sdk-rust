//! Shared jsonrpsee mock-server harness for API integration tests.

#![allow(clippy::expect_used, clippy::unwrap_used, dead_code)]

use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use znn_sdk_rust::client::websocket::WsClient;

/// A running mock Zenon node: its WebSocket URL, the registered RPC module, and
/// the handle keeping the server alive.
pub struct MockNode {
    /// `ws://` URL of the mock server.
    pub url: String,
    /// Server handle; dropping it stops the server.
    pub _handle: ServerHandle,
}

impl MockNode {
    /// Builds a mock server bound to a random local port, lets `register` add
    /// methods to its [`RpcModule`], then starts it.
    pub async fn spawn<F>(register: F) -> Self
    where
        F: FnOnce(&mut RpcModule<()>),
    {
        let server = ServerBuilder::default()
            .build("127.0.0.1:0")
            .await
            .expect("mock server binds a random port");
        let addr = server
            .local_addr()
            .expect("mock server reports its address");
        let mut module: RpcModule<()> = RpcModule::new(());
        register(&mut module);
        let handle = server.start(module);
        Self {
            url: format!("ws://{addr}"),
            _handle: handle,
        }
    }
}

/// Connects a fresh [`WsClient`] to `url` and shares it via [`Arc`].
pub async fn connect(url: &str) -> Arc<WsClient> {
    let mut client = WsClient::new();
    client
        .initialize(url, false)
        .await
        .expect("connects to the mock server");
    Arc::new(client)
}

/// A shared cell recording the positional params dispatched to a mock method.
pub type ParamsCell = Arc<Mutex<Vec<Value>>>;

/// A shared cell recording every call's positional params.
pub type CallsCell = Arc<Mutex<Vec<Vec<Value>>>>;

/// Registers `method`, records its positional params, and returns `response`.
pub fn capture_method(
    module: &mut RpcModule<()>,
    method: &'static str,
    response: Value,
) -> ParamsCell {
    let cell: ParamsCell = Arc::new(Mutex::new(Vec::new()));
    let cell2 = cell.clone();
    module
        .register_method(method, move |params, _ctx, _ext| {
            let arr: Vec<Value> = params.parse().unwrap_or_default();
            *cell2.lock().expect("params cell") = arr;
            let value: jsonrpsee::core::RpcResult<Value> = Ok(response.clone());
            value
        })
        .expect("mock method registers");
    cell
}

/// Registers `method`, records every call's positional params, and returns
/// `response`.
pub fn capture_calls(
    module: &mut RpcModule<()>,
    method: &'static str,
    response: Value,
) -> CallsCell {
    let cell: CallsCell = Arc::new(Mutex::new(Vec::new()));
    let cell2 = cell.clone();
    module
        .register_method(method, move |params, _ctx, _ext| {
            let arr: Vec<Value> = params.parse().unwrap_or_default();
            cell2.lock().expect("calls cell").push(arr);
            let value: jsonrpsee::core::RpcResult<Value> = Ok(response.clone());
            value
        })
        .expect("mock method registers");
    cell
}

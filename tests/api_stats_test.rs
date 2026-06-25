//! Integration tests for the `stats.*` JSON-RPC namespace.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::struct_field_names)]

mod support;

use serde::Deserialize;
use serde_json::{Value, json};
use support::{MockNode, capture_method, connect};
use znn_sdk_rust::api::stats::StatsApi;
use znn_sdk_rust::client::exceptions::ClientError;
use znn_sdk_rust::client::websocket::WsClient;
use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::stats::{NetworkInfo, OsInfo, ProcessInfo, SyncInfo, SyncState};

#[derive(Deserialize)]
struct StatsConformance {
    os_info: Value,
    process_info: Value,
    network_info: Value,
}

const STATS_CONFORMANCE: &str = include_str!("conformance/stats/sync.json");

#[tokio::test]
async fn os_info_decodes_and_platform_version_mirrors_platform() {
    let fixture: StatsConformance =
        serde_json::from_str(STATS_CONFORMANCE).expect("stats conformance parses");
    let expected = OsInfo::from_json(&fixture.os_info).expect("os info parses");
    let response = fixture.os_info;
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        captured = Some(capture_method(module, "stats.osInfo", response));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = StatsApi::new(connect(&node.url).await);

    let result = api.os_info().await;

    assert_eq!(
        captured.lock().expect("params cell").clone(),
        Vec::<Value>::new()
    );
    assert_eq!(result, Ok(expected.clone()));
    assert_eq!(
        expected.platform_version(),
        expected.platform(),
        "platformVersion must mirror the platform key"
    );
}

#[tokio::test]
async fn process_info_decodes() {
    let fixture: StatsConformance =
        serde_json::from_str(STATS_CONFORMANCE).expect("stats conformance parses");
    let expected = ProcessInfo::from_json(&fixture.process_info).expect("process info parses");
    let response = fixture.process_info;
    let node = MockNode::spawn(|module| {
        capture_method(module, "stats.processInfo", response);
    })
    .await;
    let api = StatsApi::new(connect(&node.url).await);

    let result = api.process_info().await;

    assert_eq!(result, Ok(expected));
}

#[tokio::test]
async fn network_info_decodes_peer_list() {
    let fixture: StatsConformance =
        serde_json::from_str(STATS_CONFORMANCE).expect("stats conformance parses");
    let expected = NetworkInfo::from_json(&fixture.network_info).expect("network info parses");
    let response = fixture.network_info;
    let node = MockNode::spawn(|module| {
        capture_method(module, "stats.networkInfo", response);
    })
    .await;
    let api = StatsApi::new(connect(&node.url).await);

    let result = api.network_info().await;

    assert_eq!(result, Ok(expected));
}

#[tokio::test]
async fn sync_info_decodes_the_state_index() {
    let response = json!({ "state": 2, "currentHeight": 10, "targetHeight": 100 });
    let expected = SyncInfo::from_json(&response).expect("sync info parses");
    let node = MockNode::spawn(|module| {
        module
            .register_method("stats.syncInfo", move |_params, _ctx, _ext| {
                let value: jsonrpsee::core::RpcResult<Value> = Ok(response.clone());
                value
            })
            .expect("method registers");
    })
    .await;
    let api = StatsApi::new(connect(&node.url).await);

    let result = api.sync_info().await;

    assert_eq!(expected.state(), SyncState::SyncDone);
    assert_eq!(expected.current_height(), 10);
    assert_eq!(expected.target_height(), 100);
    assert_eq!(result, Ok(expected));
}

#[tokio::test]
async fn a_transport_error_surfaces_as_a_client_error() {
    let api = StatsApi::new(std::sync::Arc::new(WsClient::new()));

    let result = api.os_info().await;

    assert!(
        matches!(result, Err(Error::Client(ClientError::NoConnection))),
        "a transport error must surface as Error::Client(NoConnection), got {result:?}"
    );
}

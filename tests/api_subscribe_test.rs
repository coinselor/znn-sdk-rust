//! Integration tests for the `ledger.subscribe` API.
#![allow(clippy::expect_used, clippy::unwrap_used)]

mod support;

use serde_json::{Value, json};
use support::{MockNode, capture_method, connect};
use znn_sdk_rust::api::subscribe::SubscribeApi;
use znn_sdk_rust::client::exceptions::ClientError;
use znn_sdk_rust::client::websocket::WsClient;
use znn_sdk_rust::error::Error;
use znn_sdk_rust::primitives::address::Address;

#[tokio::test]
async fn momentums_subscription_returns_the_id_and_uses_the_momentums_topic() {
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        captured = Some(capture_method(module, "ledger.subscribe", json!("sub-1")));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = SubscribeApi::new(connect(&node.url).await);

    let result = api.to_momentums().await;

    assert_eq!(
        captured.lock().expect("params cell").clone(),
        vec![json!("momentums")]
    );
    assert_eq!(result, Ok(Some("sub-1".to_string())));
}

#[tokio::test]
async fn per_address_subscription_uses_the_canonical_address() {
    let address =
        Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").expect("address parses");
    let canonical = address.to_string();
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        captured = Some(capture_method(module, "ledger.subscribe", json!("sub-2")));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = SubscribeApi::new(connect(&node.url).await);

    let result = api.to_account_blocks_by_address(&address).await;

    assert_eq!(
        captured.lock().expect("params cell").clone(),
        vec![json!("accountBlocksByAddress"), json!(canonical)]
    );
    assert_eq!(result, Ok(Some("sub-2".to_string())));
}

#[tokio::test]
async fn unreceived_subscription_uses_the_canonical_address() {
    let address =
        Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").expect("address parses");
    let canonical = address.to_string();
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        captured = Some(capture_method(module, "ledger.subscribe", json!("sub-3")));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = SubscribeApi::new(connect(&node.url).await);

    let result = api.to_unreceived_account_blocks_by_address(&address).await;

    assert_eq!(
        captured.lock().expect("params cell").clone(),
        vec![json!("unreceivedAccountBlocksByAddress"), json!(canonical)]
    );
    assert_eq!(result, Ok(Some("sub-3".to_string())));
}

#[tokio::test]
async fn null_response_maps_to_none() {
    let node = MockNode::spawn(|module| {
        capture_method(module, "ledger.subscribe", Value::Null);
    })
    .await;
    let api = SubscribeApi::new(connect(&node.url).await);

    let result = api.to_all_account_blocks().await;

    assert_eq!(result, Ok(None));
}

#[tokio::test]
async fn a_transport_error_surfaces_as_a_client_error() {
    let api = SubscribeApi::new(std::sync::Arc::new(WsClient::new()));

    let result = api.to_momentums().await;

    assert!(
        matches!(result, Err(Error::Client(ClientError::NoConnection))),
        "a transport error must surface as Error::Client(NoConnection), got {result:?}"
    );
}

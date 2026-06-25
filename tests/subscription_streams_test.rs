//! Websocket subscription handle and notification routing tests.
//!
//! Stream-returning subscribe methods create handles that receive
//! id-scoped `ledger.subscription` notifications and close when the client
//! stops.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use jsonrpsee::server::RpcModule;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use support::{MockNode, capture_method, connect};
use znn_sdk_rust::api::subscribe::{SubscribeApi, Subscription};
use znn_sdk_rust::client::websocket::WsClient;

fn register_ledger_subscription(module: &mut RpcModule<()>) {
    module
        .register_subscription::<Result<(), jsonrpsee::core::SubscriptionError>, _, _>(
            "ledger.subscribe",
            "ledger.subscription",
            "ledger.unsubscribe",
            |params, pending, _, _| async move {
                let args = params.parse::<Vec<String>>().unwrap_or_default();
                let topic = args.first().map(String::as_str).unwrap_or_default();
                let payload = match topic {
                    "momentums" => json!({"topic": "momentums", "height": 2}),
                    "allAccountBlocks" => json!({"topic": "allAccountBlocks", "height": 3}),
                    other => json!({"topic": other}),
                };
                let sink = pending.accept().await?;
                let msg = serde_json::value::to_raw_value(&payload)
                    .expect("mock notification payload serializes");
                let _ = sink.send(msg).await;
                std::future::pending::<Result<(), jsonrpsee::core::SubscriptionError>>().await
            },
        )
        .expect("mock subscription registers");
}

#[tokio::test]
async fn matching_notification_is_delivered_to_the_handle() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let client = connect(&node.url).await;
    let api = SubscribeApi::new(client.clone());
    let mut handle = api
        .to_momentums_stream()
        .await
        .expect("subscribe returns a handle");
    assert!(
        !handle.id().is_empty(),
        "stream handles expose the server subscription id"
    );

    let got = handle.recv().await;
    assert_eq!(
        got,
        Ok(Some(json!({"topic": "momentums", "height": 2}))),
        "the handle must receive the matching notification payload"
    );
}

#[tokio::test]
async fn unrelated_notification_is_not_delivered_to_the_handle() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let client = connect(&node.url).await;
    let api = SubscribeApi::new(client.clone());
    let mut momentums = api
        .to_momentums_stream()
        .await
        .expect("momentums subscribe returns a handle");
    let first = momentums.recv().await;
    assert_eq!(
        first,
        Ok(Some(json!({"topic": "momentums", "height": 2}))),
        "the first handle receives its own notification"
    );

    let mut blocks = api
        .to_all_account_blocks_stream()
        .await
        .expect("account-block subscribe returns a handle");
    let second = blocks.recv().await;
    assert_eq!(
        second,
        Ok(Some(json!({"topic": "allAccountBlocks", "height": 3}))),
        "the second handle receives its own notification"
    );

    let stray = tokio::time::timeout(Duration::from_millis(100), momentums.recv()).await;
    assert!(
        stray.is_err(),
        "a notification for another subscription id must not reach the first handle"
    );
}

#[tokio::test]
async fn stopping_the_client_terminates_the_active_handle() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let mut owned = WsClient::new();
    owned
        .initialize(&node.url, false)
        .await
        .expect("connects to the mock server");
    let client = Arc::new(owned);
    let api = SubscribeApi::new(client.clone());
    let mut handle = api
        .to_momentums_stream()
        .await
        .expect("subscribe returns a handle");
    let first = handle.recv().await;
    assert_eq!(
        first,
        Ok(Some(json!({"topic": "momentums", "height": 2}))),
        "the active handle receives notifications before stop"
    );
    // Drop the API so the only remaining strong reference is `client`.
    drop(api);

    let mut client = Arc::try_unwrap(client)
        .ok()
        .expect("single remaining reference expected after dropping the API");
    client.stop();

    let got = handle.recv().await;
    assert!(
        matches!(got, Ok(None) | Err(_)),
        "a stopped client must close or error the active subscription handle, got {got:?}"
    );
}

/// Raw subscription id access must remain available for lower-level callers
/// that route notifications themselves.
#[tokio::test]
async fn raw_subscription_id_remains_available() {
    let node = MockNode::spawn(|module| {
        capture_method(module, "ledger.subscribe", json!("raw-id-7"));
    })
    .await;
    let api = SubscribeApi::new(connect(&node.url).await);

    let result = api.to_momentums().await;

    assert_eq!(
        result,
        Ok(Some("raw-id-7".to_string())),
        "the raw id method must return the exact server subscription id"
    );
}

/// Compile-time guard: the handle type is part of the public API.
#[test]
fn subscription_handle_type_is_public() {
    let _ = std::any::type_name::<Subscription>();
}

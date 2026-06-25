//! Subscription closure classification and opt-in resubscribe tests.
//!
//! These tests cover the resubscribe policy: observable `CloseReason` on
//! transport drop/reconnect, client stop, and server-initiated close; an opt-in
//! `ResubscribeManager`; and the no-duplicate-delivery guarantee. The raw
//! `recv` signature is preserved.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use jsonrpsee::server::RpcModule;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use support::{MockNode, connect};
use znn_sdk_rust::api::subscribe::{
    CloseReason, ResubscribeManager, Subscription, SubscriptionOutcome,
};
use znn_sdk_rust::client::websocket::WsClient;

/// Registers a `ledger.subscribe` that emits one notification then stays open
/// (pends forever), so the subscription only ends when the transport drops or
/// the client stops.
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

// NOTE: A `register_ledger_subscription_oneshot` helper and a matching
// `server_initiated_close_is_observable` test were removed because their premise
// (that a server ending a subscription produces a client-observable stream-end)
// is unachievable on jsonrpsee 0.26: when the server subscription callback
// returns `Ok(())`, the client's `next()` never resolves, `close_reason()` is
// `None`, and the WebSocket stays open. There is no wire-level signal.
// At the protocol level a server-initiated close is therefore indistinguishable
// from a transport reconnect, so the spec's `TransportReconnected` fallback
// covers it. The `ServerInitiated` variant is retained as a forward-compat
// placeholder for a future protocol/transport that does signal it.

/// Polls `recv_event` (with a safety timeout) skipping delivered items until the
/// handle reports a classified closure. Returns the [`CloseReason`], or `None`
/// if the handle closed without classification (the red-phase stub path) or the
/// poll timed out. Using a loop avoids racing the initial notification against
/// the closure event.
async fn drain_until_close(handle: &mut Subscription) -> Option<CloseReason> {
    loop {
        let polled = tokio::time::timeout(Duration::from_secs(2), handle.recv_event()).await;
        match polled {
            Ok(Ok(Some(SubscriptionOutcome::Closed { reason }))) => return Some(reason),
            // An item (or any future non-exhaustive item kind): keep polling.
            Ok(Ok(Some(_))) => {}
            // Closed without classification, an error, or a timeout: give up.
            Ok(Ok(None) | Err(_)) | Err(_) => return None,
        }
    }
}

/// Describes a resubscribe outcome without `Debug`-formatting the inner
/// `Subscription` (which is intentionally not `Debug`).
fn describe_resubscribe(result: &Result<Option<Subscription>, znn_sdk_rust::Error>) -> String {
    match result {
        Ok(Some(_)) => "Ok(Some(handle))".to_string(),
        Ok(None) => "Ok(None)".to_string(),
        Err(e) => format!("Err({e:?})"),
    }
}

/// Subscribes a plain (unmanaged) momentum handle against `node`, keeping the
/// owning client and api alive for the returned handle's lifetime.
async fn momentum_handle(node: &MockNode) -> (Arc<WsClient>, Subscription) {
    let mut owned = WsClient::new();
    owned
        .initialize(&node.url, false)
        .await
        .expect("connects to the mock server");
    let client = Arc::new(owned);
    let api = znn_sdk_rust::api::subscribe::SubscribeApi::new(client.clone());
    let handle = api
        .to_momentums_stream()
        .await
        .expect("subscribe returns a handle");
    // The api only needs to live long enough to open the subscription.
    drop(api);
    (client, handle)
}

/// Distinct `CloseReason` variants must compare unequal so callers can branch on
/// the cause of closure.
#[test]
fn close_reason_variants_are_distinct() {
    let variants = [
        CloseReason::ClientStopped,
        CloseReason::TransportReconnected,
        CloseReason::ServerInitiated,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "distinct CloseReason variants must not be equal");
            }
        }
    }
}

/// A client stop must surface `Closed { reason: ClientStopped }` via
/// `recv_event`, distinct from the unclassified `Ok(None)` of raw `recv`.
/// Red-phase stub returns `Ok(None)` on closure, so the variant assertion fails.
#[tokio::test]
async fn client_stop_is_observable_as_client_stopped() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let (client, mut handle) = momentum_handle(&node).await;
    // Drain the initial notification so the next poll observes closure.
    let _ = handle.recv().await;
    let mut client = Arc::try_unwrap(client)
        .ok()
        .expect("single remaining reference");
    client.stop();

    let outcome = handle.recv_event().await;
    assert!(
        matches!(
            outcome,
            Ok(Some(SubscriptionOutcome::Closed {
                reason: CloseReason::ClientStopped
            }))
        ),
        "stop must surface Closed {{ reason: ClientStopped }}, got {outcome:?}"
    );
}

/// A real transport drop (server gone) while the client stays `Running` must
/// surface `Closed { reason: TransportReconnected }`, then `Ok(None)` on the
/// next poll. This is the headline scenario of the change. Red-phase stub
/// returns `Ok(None)` instead of a classified `Closed`, so the assertion fails.
#[tokio::test]
async fn transport_drop_is_observable_as_transport_reconnected() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let (_client, mut handle) = momentum_handle(&node).await;
    // Drain the initial notification so the loop reaches closure quickly.
    let _ = handle.recv().await;

    // Real transport drop: the client is never stopped, so it stays Running and
    // the stream-end classifies as TransportReconnected.
    drop(node);

    let reason = drain_until_close(&mut handle).await;
    assert_eq!(
        reason,
        Some(CloseReason::TransportReconnected),
        "a transport drop while the client is Running must classify as \
         TransportReconnected, got {reason:?}"
    );
    let after = handle.recv_event().await;
    assert!(
        matches!(after, Ok(None)),
        "after a classified Closed, a subsequent recv_event must return Ok(None), got {after:?}"
    );
}

/// Raw `recv` must keep returning `Ok(None)` on any closure (signature
/// unchanged), so existing callers are unaffected.
#[tokio::test]
async fn raw_recv_still_returns_none_on_stop() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let (client, mut handle) = momentum_handle(&node).await;
    let _ = handle.recv().await;
    let mut client = Arc::try_unwrap(client)
        .ok()
        .expect("single remaining reference");
    client.stop();

    let got = handle.recv().await;
    assert!(
        matches!(got, Ok(None)),
        "raw recv must still return Ok(None) on stop, got {got:?}"
    );
}

/// After closure is observed, a manually recreated subscription must deliver a
/// fresh notification. The default handle must not auto-resubscribe.
#[tokio::test]
async fn manual_recreate_delivers_new_notifications() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let client = connect(&node.url).await;
    let api = znn_sdk_rust::api::subscribe::SubscribeApi::new(client.clone());
    let first = api
        .to_momentums_stream()
        .await
        .expect("first subscribe returns a handle");
    drop(first);

    let mut second = api
        .to_momentums_stream()
        .await
        .expect("recreated subscribe returns a handle");
    let got = tokio::time::timeout(Duration::from_millis(500), second.recv()).await;
    assert!(
        matches!(got, Ok(Ok(Some(_)))),
        "a recreated subscription must deliver a fresh notification, got {got:?}"
    );
}

/// A managed subscription must survive a real transport drop + reconnect and
/// keep delivering under a new server id. This drops the mock server (real
/// disconnect), re-spawns it and reconnects, then expects `resubscribe_if_needed`
/// to hand back a freshly bound handle. The red-phase stub returns `Ok(None)`,
/// so the `Ok(Some(_))` assertion fails.
#[tokio::test]
async fn managed_subscription_resubscribes_after_transport_reconnect() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let mut client = WsClient::new();
    client
        .initialize(&node.url, false)
        .await
        .expect("connects to the mock server");
    let mut manager = ResubscribeManager::new(client);
    let original = manager
        .subscribe_momentums()
        .await
        .expect("managed subscribe returns a handle");
    let original_id = original.id().to_string();
    assert!(!original_id.is_empty(), "managed handle exposes an id");
    // The old handle is dead after the reconnect; drop it so its id deregisters.
    drop(original);

    // Real transport drop: stop the server. The notification router ends and
    // clears the registry.
    drop(node);

    // Reconnect: spawn a fresh server and dial it through the manager.
    let node2 = MockNode::spawn(register_ledger_subscription).await;
    manager
        .reconnect(&node2.url)
        .await
        .expect("reconnect dials the fresh server");

    let rebound = manager.resubscribe_if_needed().await;
    let rebound_label = describe_resubscribe(&rebound);
    assert!(
        matches!(rebound, Ok(Some(_))),
        "after a real reconnect with a prior subscription, resubscribe_if_needed must \
         return Ok(Some(handle)) carrying a re-bound subscription, got {rebound_label}"
    );
    let mut handle = rebound
        .ok()
        .flatten()
        .expect("rebound handle present after the Ok(Some) assertion");

    // The rebound handle must carry a new server id.
    assert_ne!(
        handle.id(),
        original_id.as_str(),
        "the rebound handle must carry a new server id after reconnect"
    );
    let next = tokio::time::timeout(Duration::from_secs(2), handle.recv()).await;
    assert!(
        matches!(next, Ok(Ok(Some(_)))),
        "the managed handle must deliver a notification after resubscribe, got {next:?}"
    );
}

/// When there is no recorded subscription to rebind, `resubscribe_if_needed`
/// must report a no-op (`Ok(None)`), never a spurious handle.
#[tokio::test]
async fn resubscribe_with_no_prior_subscription_is_a_noop() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let mut client = WsClient::new();
    client
        .initialize(&node.url, false)
        .await
        .expect("connects to the mock server");
    let mut manager = ResubscribeManager::new(client);

    let result = manager.resubscribe_if_needed().await;
    let label = describe_resubscribe(&result);
    assert!(
        matches!(result, Ok(None)),
        "resubscribe with no prior subscription must report Ok(None) (no-op), got {label}"
    );
}

/// A failed re-subscribe RPC must surface an error rather than silently
/// succeeding. After a real transport drop, the manager reconnects to a server
/// that has NO `ledger.subscribe` method, so the re-subscribe RPC fails. The
/// red-phase stub returns `Ok(None)`, so the error assertion fails.
#[tokio::test]
async fn failed_resubscribe_after_reconnect_surfaces_an_error() {
    let node = MockNode::spawn(register_ledger_subscription).await;
    let mut client = WsClient::new();
    client
        .initialize(&node.url, false)
        .await
        .expect("connects to the mock server");
    let mut manager = ResubscribeManager::new(client);
    let _handle = manager
        .subscribe_momentums()
        .await
        .expect("managed subscribe returns a handle");

    // Real transport drop.
    drop(node);

    // Reconnect to a server with no subscription method: the re-subscribe RPC
    // must fail.
    let node2 = MockNode::spawn(|_module: &mut RpcModule<()>| {}).await;
    manager
        .reconnect(&node2.url)
        .await
        .expect("reconnect dials the fresh server");

    let resubscribe = manager.resubscribe_if_needed().await;
    let label = describe_resubscribe(&resubscribe);
    assert!(
        resubscribe.is_err(),
        "a resubscribe attempt against a server without ledger.subscribe must \
         surface an error, got {label}"
    );
}

/// Compile-time guard: the new public types are part of the public API.
#[test]
fn resubscribe_types_are_public() {
    let _ = std::any::type_name::<CloseReason>();
    let _ = std::any::type_name::<SubscriptionOutcome>();
    let _ = std::any::type_name::<ResubscribeManager>();
    let _ = std::any::type_name::<Subscription>();
}

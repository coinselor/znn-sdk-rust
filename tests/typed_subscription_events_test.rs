//! Typed subscription event decoding tests.
//!
//! Loads the envelope vectors and asserts each notification decodes to the
//! expected `SubscriptionEvent` variant. Also exercises item-level error
//! handling and the raw/typed parallel read.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use jsonrpsee::server::RpcModule;
use serde_json::{Value, json};
use support::{MockNode, connect};
use znn_sdk_rust::api::subscribe::{Subscription, SubscriptionEvent};

const EVENTS_JSON: &str = include_str!("vectors/subscription/events.json");

/// Loads the vector file.
fn load_cases() -> Vec<Value> {
    let doc: Value = serde_json::from_str(EVENTS_JSON).expect("vectors parse");
    doc.get("cases")
        .and_then(Value::as_array)
        .expect("cases array")
        .clone()
}

/// Decodes a single notification through the public `SubscriptionEvent::decode`
/// constructor and matches the result against the vector's expectation.
#[test]
fn each_vector_decodes_to_expected_event_variant() {
    for case in load_cases() {
        let name = case["name"].as_str().expect("case name").to_string();
        let topic = case["topic"].as_str().expect("case topic").to_string();
        let payload = case["payload"].clone();
        let expected = case["expected_event"].as_str().expect("expected_event");

        let result = SubscriptionEvent::decode(&topic, &payload);
        match expected {
            "Momentum" => {
                let event = result.expect("momentum decodes");
                assert!(
                    matches!(event, SubscriptionEvent::Momentum(_)),
                    "{name}: expected Momentum variant, got {event:?}"
                );
                if let SubscriptionEvent::Momentum(m) = event {
                    let expected_hash = case["expected_hash"].as_str().expect("expected_hash");
                    assert_eq!(
                        m.hash().to_string(),
                        expected_hash,
                        "{name}: decoded momentum hash must match"
                    );
                }
            }
            "AccountBlocks" => {
                let event = result.expect("account blocks decode");
                let expected_len = case["expected_len"].as_u64().expect("expected_len");
                assert!(
                    matches!(event, SubscriptionEvent::AccountBlocks(ref blocks) if blocks.len() as u64 == expected_len),
                    "{name}: expected AccountBlocks of len {expected_len}, got {event:?}"
                );
            }
            "UnreceivedAccountBlocks" => {
                let event = result.expect("unreceived decode");
                let expected_len = case["expected_len"].as_u64().expect("expected_len");
                assert!(
                    matches!(event, SubscriptionEvent::UnreceivedAccountBlocks(ref blocks) if blocks.len() as u64 == expected_len),
                    "{name}: expected UnreceivedAccountBlocks of len {expected_len}, got {event:?}"
                );
            }
            "Unknown" => {
                let event = result.expect("unknown decodes");
                let expected_topic = case["expected_topic"].as_str().expect("expected_topic");
                assert!(
                    matches!(event, SubscriptionEvent::Unknown { ref topic, ref payload } if topic == expected_topic && payload == &case["payload"]),
                    "{name}: expected Unknown {{ topic: {expected_topic}, .. }} preserving raw payload, got {event:?}"
                );
            }
            "Error" => {
                let expected_kind = case["expected_error_kind"]
                    .as_str()
                    .expect("expected_error_kind");
                let kind_matches = match expected_kind {
                    "InvalidInput" => {
                        matches!(result, Err(znn_sdk_rust::Error::InvalidInput(_)))
                    }
                    _ => false,
                };
                assert!(
                    kind_matches,
                    "{name}: expected {expected_kind} error, got {result:?}"
                );
            }
            _ => {
                assert!(
                    matches!(expected, "Momentum"),
                    "{name}: unhandled expected_event {expected}"
                );
            }
        }
    }
}

/// A malformed payload must not poison the subscription: after an item-level
/// decode error, a well-formed payload still decodes.
#[test]
fn malformed_payload_errors_but_keeps_decode_usable() {
    let bad = json!({ "version": 1, "hash": "not-a-full-momentum" });
    let good = json!({
        "version": 1,
        "chainIdentifier": 100,
        "hash": "c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9",
        "previousHash": "0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712",
        "height": 2,
        "timestamp": 1_000_000_010,
        "data": "",
        "content": [],
        "changesHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
        "publicKey": "SAPwVIVQma3zMak169crdLkcu2B2Gm3iBCdDgfQ6IxU=",
        "signature": "qvlKN6rTQgM11/FosNazpeReViuD1GH1tIww2F0zNfXruTp3g9ULhA1mYnRYAiPJyP2NlIGhENwhzBAHJ0dYBw==",
        "producer": "z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju"
    });

    let first = SubscriptionEvent::decode("momentums", &bad);
    assert!(
        matches!(first, Err(znn_sdk_rust::Error::InvalidInput(_))),
        "malformed payload must yield InvalidInput, got {first:?}"
    );
    let second = SubscriptionEvent::decode("momentums", &good);
    assert!(
        matches!(second, Ok(SubscriptionEvent::Momentum(_))),
        "a subsequent well-formed payload must still decode, got {second:?}"
    );
}

/// `recv_typed` and raw `recv` must read the same notification stream. The
/// red-phase stub returns the wrong variant, so the typed assertion fails.
#[tokio::test]
async fn recv_typed_returns_typed_variant_for_live_notification() {
    let node = MockNode::spawn(|module: &mut RpcModule<()>| {
        module
            .register_subscription::<Result<(), jsonrpsee::core::SubscriptionError>, _, _>(
                "ledger.subscribe",
                "ledger.subscription",
                "ledger.unsubscribe",
                |_params, pending, _, _| async move {
                    let payload = json!({
                        "version": 1,
                        "chainIdentifier": 100,
                        "hash": "c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9",
                        "previousHash": "0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712",
                        "height": 2,
                        "timestamp": 1_000_000_010,
                        "data": "",
                        "content": [],
                        "changesHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
                        "publicKey": "SAPwVIVQma3zMak169crdLkcu2B2Gm3iBCdDgfQ6IxU=",
                        "signature": "qvlKN6rTQgM11/FosNazpeReViuD1GH1tIww2F0zNfXruTp3g9ULhA1mYnRYAiPJyP2NlIGhENwhzBAHJ0dYBw==",
                        "producer": "z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju"
                    });
                    let sink = pending.accept().await.expect("accept");
                    let msg = serde_json::value::to_raw_value(&payload)
                        .expect("payload serializes");
                    let _ = sink.send(msg).await;
                    std::future::pending::<Result<(), jsonrpsee::core::SubscriptionError>>().await
                },
            )
            .expect("subscription registers");
    })
    .await;
    let client = connect(&node.url).await;
    let api = znn_sdk_rust::api::subscribe::SubscribeApi::new(client);
    let mut handle = api
        .to_momentums_stream()
        .await
        .expect("subscribe returns a handle");

    let typed = handle.recv_typed().await;
    assert!(
        matches!(typed, Ok(Some(SubscriptionEvent::Momentum(_)))),
        "recv_typed must return a Momentum variant for a momentum notification, got {typed:?}"
    );
}

/// Raw `recv` and typed `recv_typed` read the same notification stream: the raw
/// call returns the undecoded `Value`, and a subsequent typed call on an
/// equivalent notification returns the matching variant. Red-phase stub returns
/// `Unknown` from `recv_typed`, so the typed assertion fails while the raw
/// assertion (signature unchanged) passes.
#[tokio::test]
async fn raw_recv_and_recv_typed_read_same_stream() {
    let node = MockNode::spawn(|module: &mut RpcModule<()>| {
        module
            .register_subscription::<Result<(), jsonrpsee::core::SubscriptionError>, _, _>(
                "ledger.subscribe",
                "ledger.subscription",
                "ledger.unsubscribe",
                |_params, pending, _, _| async move {
                    let payload = json!({
                        "version": 1,
                        "chainIdentifier": 100,
                        "hash": "c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9",
                        "previousHash": "0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712",
                        "height": 2,
                        "timestamp": 1_000_000_010,
                        "data": "",
                        "content": [],
                        "changesHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
                        "publicKey": "SAPwVIVQma3zMak169crdLkcu2B2Gm3iBCdDgfQ6IxU=",
                        "signature": "qvlKN6rTQgM11/FosNazpeReViuD1GH1tIww2F0zNfXruTp3g9ULhA1mYnRYAiPJyP2NlIGhENwhzBAHJ0dYBw==",
                        "producer": "z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju"
                    });
                    let sink = pending.accept().await.expect("accept");
                    // Emit two equivalent notifications: one read raw, one typed.
                    for _ in 0..2 {
                        let msg = serde_json::value::to_raw_value(&payload)
                            .expect("payload serializes");
                        let _ = sink.send(msg).await;
                    }
                    std::future::pending::<Result<(), jsonrpsee::core::SubscriptionError>>().await
                },
            )
            .expect("subscription registers");
    })
    .await;
    let client = connect(&node.url).await;
    let api = znn_sdk_rust::api::subscribe::SubscribeApi::new(client);
    let mut handle = api
        .to_momentums_stream()
        .await
        .expect("subscribe returns a handle");

    let raw = handle.recv().await;
    assert!(
        matches!(
            &raw,
            Ok(Some(value))
                if value["hash"]
                    == json!("c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9")
        ),
        "raw recv must return the undecoded momentum value, got {raw:?}"
    );

    let typed = handle.recv_typed().await;
    assert!(
        matches!(typed, Ok(Some(SubscriptionEvent::Momentum(_)))),
        "recv_typed must return a Momentum variant derived from the same shape, got {typed:?}"
    );
}

/// Compile-time guard: the typed decode entry points are part of the public API.
#[test]
fn typed_event_types_are_public() {
    let _ = std::any::type_name::<SubscriptionEvent>();
    let _ = std::any::type_name::<Subscription>();
}

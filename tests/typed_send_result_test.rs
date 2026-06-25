//! Typed publish/send result tests.
//!
//! Covers `PublishResult`/`PublishError` and the typed `publish_transaction` /
//! `send_typed` paths, while asserting the raw `publish_raw_transaction` / `send`
//! signatures stay unchanged.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use num_bigint::BigUint;
use serde_json::{Value, json};
use support::{MockNode, capture_calls, capture_method, connect};
use znn_sdk_rust::api::ledger::{PublishError, PublishResult};
use znn_sdk_rust::model::nom::account_block_template::AccountBlockTemplate;
use znn_sdk_rust::primitives::address::Address;
use znn_sdk_rust::primitives::hash::Hash;
use znn_sdk_rust::primitives::token_standard::znn_token_standard;
use znn_sdk_rust::utils::block;
use znn_sdk_rust::wallet::keypair::KeyPair;
use znn_sdk_rust::zenon::Zenon;

const PUBLISH_CONFORMANCE: &str = include_str!("conformance/ledger/publish_response.json");

fn momentum_json() -> Value {
    json!({
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
    })
}

fn send_template() -> AccountBlockTemplate {
    AccountBlockTemplate::send(
        Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").expect("address parses"),
        znn_token_standard(),
        BigUint::from(1u32),
        None,
    )
}

/// The conformance file pins the accepted (`null`) and rejected responses.
#[test]
fn conformance_pins_accepted_and_rejected_responses() {
    let doc: Value = serde_json::from_str(PUBLISH_CONFORMANCE).expect("conformance parses");
    assert!(doc["accepted"].is_null(), "accepted response is null");
    assert!(
        doc["rejected"]["response"].is_string(),
        "rejected response is a string"
    );
    assert!(
        doc["unexpected"]["response"].is_object(),
        "unexpected response is an object"
    );
}

/// `PublishError::from_response` decodes a string to `Rejected` and an unknown
/// object to `Unexpected`. The red-phase stub maps everything to `Unexpected`,
/// so the `Rejected` assertion fails.
#[test]
fn publish_error_decodes_non_null_responses() {
    let doc: Value = serde_json::from_str(PUBLISH_CONFORMANCE).expect("conformance parses");
    let rejected = doc["rejected"]["response"].clone();
    let unexpected = doc["unexpected"]["response"].clone();

    let rejected_err = PublishError::from_response(&rejected);
    let rejected_msg = rejected.as_str().expect("rejected response is a string");
    assert!(
        matches!(rejected_err, PublishError::Rejected { ref message } if message == rejected_msg),
        "a non-null string maps to Rejected, got {rejected_err:?}"
    );

    let unexpected_err = PublishError::from_response(&unexpected);
    assert!(
        matches!(unexpected_err, PublishError::Unexpected(ref rendered) if rendered == &unexpected.to_string()),
        "an unknown object maps to Unexpected(rendered), got {unexpected_err:?}"
    );
}

/// `publish_transaction` on a `null` response returns `PublishResult` with the
/// computed hash and prepared template.
#[tokio::test]
async fn publish_transaction_accepts_on_null_response() {
    let node = MockNode::spawn(|module| {
        capture_method(module, "ledger.publishRawTransaction", Value::Null);
    })
    .await;
    let ledger = znn_sdk_rust::api::ledger::LedgerApi::new(connect(&node.url).await);
    let template = send_template();

    let result = ledger
        .publish_transaction(&template)
        .await
        .expect("publish_transaction accepts on null");

    assert_eq!(
        result.template(),
        &template,
        "the result carries the prepared template"
    );
    // The hash matches the transaction hash computed for the template.
    let expected_hash = block::get_transaction_hash(&template).expect("hash computes");
    assert_eq!(
        result.hash(),
        &expected_hash,
        "the result hash matches the computed transaction hash"
    );
}

/// `publish_transaction` surfaces the typed error through `Error::Publish` on a
/// non-null rejection string. This proves the typed detail round-trips through
/// the public API, rather than only proving that an error occurred.
#[tokio::test]
async fn publish_transaction_surfaces_rejected_via_error_publish() {
    let node = MockNode::spawn(|module| {
        capture_method(
            module,
            "ledger.publishRawTransaction",
            json!("AccountBlock is already published"),
        );
    })
    .await;
    let ledger = znn_sdk_rust::api::ledger::LedgerApi::new(connect(&node.url).await);
    let template = send_template();

    let result = ledger.publish_transaction(&template).await;
    assert!(
        matches!(
            result,
            Err(znn_sdk_rust::Error::Publish(PublishError::Rejected { ref message }))
                if message == "AccountBlock is already published"
        ),
        "must surface Err(Error::Publish(Rejected {{ .. }})) with the node message, got {result:?}"
    );
}

/// `publish_transaction` surfaces `Error::Publish(PublishError::Unexpected(_))`
/// on an unknown non-null object.
#[tokio::test]
async fn publish_transaction_surfaces_unexpected_via_error_publish() {
    let payload = json!({ "error": "unknown", "code": -32000 });
    let node = MockNode::spawn(|module| {
        capture_method(module, "ledger.publishRawTransaction", payload.clone());
    })
    .await;
    let ledger = znn_sdk_rust::api::ledger::LedgerApi::new(connect(&node.url).await);
    let template = send_template();

    let result = ledger.publish_transaction(&template).await;
    assert!(
        matches!(
            result,
            Err(znn_sdk_rust::Error::Publish(PublishError::Unexpected(_)))
        ),
        "must surface Err(Error::Publish(Unexpected(_))), got {result:?}"
    );
}

/// `publish_raw_transaction` returns the raw node value unchanged (`null`).
#[tokio::test]
async fn publish_raw_transaction_returns_raw_value_unchanged() {
    let node = MockNode::spawn(|module| {
        capture_method(module, "ledger.publishRawTransaction", Value::Null);
    })
    .await;
    let ledger = znn_sdk_rust::api::ledger::LedgerApi::new(connect(&node.url).await);
    let template = send_template();

    let result = ledger
        .publish_raw_transaction(&template)
        .await
        .expect("raw publish returns the node value");
    assert_eq!(result, Value::Null, "raw publish returns the node null");
}

/// `send_typed` returns acceptance on a `null` response with the computed hash.
#[tokio::test]
async fn send_typed_accepts_on_null_response() {
    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 0, "basePlasma": 21_000, "requiredDifficulty": 0}),
        );
        capture_calls(module, "ledger.publishRawTransaction", Value::Null);
    })
    .await;
    let zenon = Zenon::from_client(connect(&node.url).await);
    let keypair = KeyPair::from_private_key([7u8; 32]);

    let result = zenon
        .send_typed(&send_template(), Some(&keypair))
        .await
        .expect("send_typed accepts on null");

    let prepared = zenon
        .prepare_block(&send_template(), Some(&keypair))
        .await
        .expect("prepare_block returns a template");
    assert_eq!(
        result.hash(),
        prepared.hash(),
        "send_typed result hash matches the prepared template hash"
    );
}

/// Raw `send` and typed `send_typed` must publish the exact template produced by
/// `prepare_block`, and `send_typed` must agree on its hash. This exercises both
/// paths and compares each captured publish payload against the prepared
/// template, so a divergence in either path is caught.
#[tokio::test]
async fn send_and_send_typed_publish_the_same_template() {
    let mut publish_calls = None;
    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 0, "basePlasma": 21_000, "requiredDifficulty": 0}),
        );
        publish_calls = Some(capture_calls(
            module,
            "ledger.publishRawTransaction",
            Value::Null,
        ));
    })
    .await;
    let zenon = Zenon::from_client(connect(&node.url).await);
    let keypair = KeyPair::from_private_key([7u8; 32]);
    let template = send_template();

    // Preparation is deterministic for this fixture (required difficulty 0), so
    // an independent prepare matches what each send path publishes.
    let prepared = zenon
        .prepare_block(&template, Some(&keypair))
        .await
        .expect("prepare_block returns a template");

    // Raw send first, then typed send: both hit ledger.publishRawTransaction.
    let _ = zenon
        .send(&template, Some(&keypair))
        .await
        .expect("raw send publishes");
    let typed = zenon
        .send_typed(&template, Some(&keypair))
        .await
        .expect("send_typed publishes");

    let calls = publish_calls
        .expect("publish capture")
        .lock()
        .expect("calls")
        .clone();
    assert_eq!(
        calls.len(),
        2,
        "both send and send_typed must publish exactly once each"
    );
    let raw_published = calls[0][0].clone();
    let typed_published = calls[1][0].clone();
    assert_eq!(
        raw_published,
        prepared.to_json(),
        "raw send must publish the exact template produced by prepare_block"
    );
    assert_eq!(
        typed_published,
        prepared.to_json(),
        "send_typed must publish the exact template produced by prepare_block"
    );
    assert_eq!(
        typed.hash(),
        prepared.hash(),
        "send_typed result hash must match the published template hash"
    );
}

/// `send` (raw) signature/behavior is unchanged: returns the raw `Value`.
#[tokio::test]
async fn raw_send_returns_raw_value_unchanged() {
    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 0, "basePlasma": 21_000, "requiredDifficulty": 0}),
        );
        capture_calls(module, "ledger.publishRawTransaction", Value::Null);
    })
    .await;
    let zenon = Zenon::from_client(connect(&node.url).await);
    let keypair = KeyPair::from_private_key([7u8; 32]);

    let result = zenon
        .send(&send_template(), Some(&keypair))
        .await
        .expect("raw send returns the node value");
    assert_eq!(result, Value::Null, "raw send returns the node null");
}

/// Compile-time guard: the typed result types are public.
#[test]
fn typed_send_types_are_public() {
    let _ = std::any::type_name::<PublishResult>();
    let _ = std::any::type_name::<PublishError>();
    let _ = std::any::type_name::<Hash>();
}

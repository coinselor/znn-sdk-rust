//! Integration tests for the high-level Zenon SDK entry point.
#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use num_bigint::BigUint;
use serde_json::{Value, json};
use support::{MockNode, capture_calls, connect};
use znn_sdk_rust::api::PageQuery;
use znn_sdk_rust::client::ConnectionState;
use znn_sdk_rust::client::exceptions::ClientError;
use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::nom::account_block_template::{AccountBlockTemplate, BlockType};
use znn_sdk_rust::primitives::address::Address;
use znn_sdk_rust::primitives::token_standard::znn_token_standard;
use znn_sdk_rust::utils::block;
use znn_sdk_rust::wallet::keypair::KeyPair;
use znn_sdk_rust::zenon::Zenon;

const STATS: &str = include_str!("conformance/stats/sync.json");
const PILLAR: &str = include_str!("conformance/embedded/pillar.json");

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

fn fixture(source: &str, key: &str) -> Value {
    serde_json::from_str::<Value>(source).expect("fixture parses")[key].clone()
}

#[tokio::test]
async fn every_api_root_shares_one_connection() {
    let mut ledger_calls = None;
    let mut stats_calls = None;
    let mut pillar_calls = None;
    let node = MockNode::spawn(|module| {
        ledger_calls = Some(capture_calls(
            module,
            "ledger.getFrontierMomentum",
            momentum_json(),
        ));
        stats_calls = Some(capture_calls(
            module,
            "stats.osInfo",
            fixture(STATS, "os_info"),
        ));
        pillar_calls = Some(capture_calls(
            module,
            "embedded.pillar.getAll",
            fixture(PILLAR, "pillar_info_list"),
        ));
    })
    .await;
    let zenon = Zenon::from_client(connect(&node.url).await);

    let _ = zenon.ledger.get_frontier_momentum().await;
    let _ = zenon.stats.os_info().await;
    let _ = zenon.embedded.pillar.get_all(PageQuery::default()).await;

    assert_eq!(
        ledger_calls
            .expect("ledger calls")
            .lock()
            .expect("calls")
            .len(),
        1
    );
    assert_eq!(
        stats_calls
            .expect("stats calls")
            .lock()
            .expect("calls")
            .len(),
        1
    );
    assert_eq!(
        pillar_calls
            .expect("pillar calls")
            .lock()
            .expect("calls")
            .len(),
        1
    );
}

#[tokio::test]
async fn connect_initializes_before_sharing() {
    let mut ledger_calls = None;
    let node = MockNode::spawn(|module| {
        ledger_calls = Some(capture_calls(
            module,
            "ledger.getFrontierMomentum",
            momentum_json(),
        ));
    })
    .await;
    let zenon = Zenon::connect(&node.url, false)
        .await
        .expect("connect succeeds against mock server");

    assert_eq!(zenon.client().status(), ConnectionState::Running);
    let _ = zenon.ledger.get_frontier_momentum().await;
    assert_eq!(
        ledger_calls
            .expect("ledger calls")
            .lock()
            .expect("calls")
            .len(),
        1
    );
}

#[tokio::test]
async fn fresh_sdk_entry_point_is_unconnected() {
    let zenon = Zenon::new();

    assert_eq!(zenon.client().status(), ConnectionState::Uninitialized);
    let result = zenon.ledger.get_frontier_momentum().await;
    assert!(
        matches!(result, Err(Error::Client(ClientError::NoConnection))),
        "unconnected SDK entry point must surface ClientError::NoConnection, got {result:?}"
    );
}

#[tokio::test]
async fn embedded_aggregator_wires_a_sub_api() {
    let mut calls = None;
    let node = MockNode::spawn(|module| {
        calls = Some(capture_calls(
            module,
            "embedded.pillar.getAll",
            fixture(PILLAR, "pillar_info_list"),
        ));
    })
    .await;
    let zenon = Zenon::from_client(connect(&node.url).await);

    let _ = zenon.embedded.pillar.get_all(PageQuery::default()).await;

    assert_eq!(calls.expect("calls").lock().expect("calls").len(), 1);
}

#[tokio::test]
async fn send_without_a_key_pair_is_rejected() {
    let zenon = Zenon::new();
    let template = AccountBlockTemplate::new(BlockType::UserSend);

    let result = zenon.send(&template, None).await;

    assert!(
        matches!(result, Err(Error::Generic(ref message)) if message == "No default wallet account selected"),
        "send without a selected key pair must return the exact no-keypair error, got {result:?}"
    );
}

#[tokio::test]
async fn send_with_explicit_key_pair_completes_signs_and_publishes() {
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
    let template = AccountBlockTemplate::send(
        Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").expect("address parses"),
        znn_token_standard(),
        BigUint::from(1u32),
        None,
    );

    let result = zenon.send(&template, Some(&keypair)).await;

    assert_eq!(result, Ok(Value::Null));
    let calls = publish_calls
        .expect("publish calls")
        .lock()
        .expect("calls")
        .clone();
    assert_eq!(calls.len(), 1);
    let published = calls[0][0].clone();
    let signed = AccountBlockTemplate::from_json(&published).expect("published template parses");
    assert_eq!(
        signed.address(),
        &keypair.address().expect("address derives")
    );
    assert_eq!(signed.height(), 1);
    assert_eq!(signed.momentum_acknowledged().height(), 2);
    assert_eq!(signed.fused_plasma(), 21_000);
    assert_eq!(signed.difficulty(), 0);
    assert_eq!(signed.nonce(), "0000000000000000");
    assert_eq!(signed.public_key(), keypair.public_key());
    let expected_hash = block::get_transaction_hash(&signed).expect("hash computes");
    assert_eq!(signed.hash(), &expected_hash);
    let signature: [u8; 64] = signed.signature().try_into().expect("64-byte signature");
    assert!(keypair.verify(&signature, expected_hash.bytes()));
    assert_eq!(
        STANDARD
            .decode(published["publicKey"].as_str().expect("public key string"))
            .expect("base64 decodes"),
        keypair.public_key()
    );
}

#[tokio::test]
async fn send_uses_the_default_key_pair_when_none_is_passed() {
    let mut publish_calls = None;
    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 0, "basePlasma": 1, "requiredDifficulty": 0}),
        );
        publish_calls = Some(capture_calls(
            module,
            "ledger.publishRawTransaction",
            Value::Null,
        ));
    })
    .await;
    let mut zenon = Zenon::from_client(connect(&node.url).await);
    let template = AccountBlockTemplate::new(BlockType::UserSend);
    zenon.set_default_key_pair(KeyPair::from_private_key([7u8; 32]));

    let result = zenon.send(&template, None).await;

    assert_eq!(result, Ok(Value::Null));
    assert_eq!(
        publish_calls
            .expect("publish calls")
            .lock()
            .expect("calls")
            .len(),
        1
    );
}

#[tokio::test]
async fn requires_pow_queries_plasma_requirement() {
    let node = MockNode::spawn(|module| {
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 0, "basePlasma": 0, "requiredDifficulty": 7}),
        );
    })
    .await;
    let zenon = Zenon::from_client(connect(&node.url).await);
    let template = AccountBlockTemplate::new(BlockType::UserSend);
    let keypair = KeyPair::from_private_key([7u8; 32]);

    let result = zenon.requires_pow(&template, &keypair).await;

    assert_eq!(result, Ok(true));
}

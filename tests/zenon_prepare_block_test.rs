//! Integration tests for `Zenon::prepare_block`.
//!
//! Preparation completes, signs, and returns an account-block template without
//! publishing it. `send` uses the same preparation path before publishing.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use num_bigint::BigUint;
use serde_json::{Value, json};
use support::{MockNode, capture_calls, connect};
use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::nom::account_block_template::{AccountBlockTemplate, BlockType};
use znn_sdk_rust::primitives::address::Address;
use znn_sdk_rust::primitives::hash::Hash;
use znn_sdk_rust::primitives::token_standard::znn_token_standard;
use znn_sdk_rust::utils::block;
use znn_sdk_rust::wallet::keypair::KeyPair;
use znn_sdk_rust::zenon::Zenon;

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

#[tokio::test]
async fn prepare_block_completes_signs_and_returns_without_publishing() {
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

    let prepared = zenon
        .prepare_block(&send_template(), Some(&keypair))
        .await
        .expect("prepare_block returns a template");

    // Preparation must never publish.
    assert_eq!(
        publish_calls
            .expect("publish capture")
            .lock()
            .expect("calls")
            .len(),
        0,
        "prepare_block must not call ledger.publishRawTransaction"
    );

    // The prepared template must carry the completed fields and a verifying
    // signature, matching the documented prepare contract.
    assert_eq!(
        prepared.address(),
        &keypair.address().expect("address derives"),
        "prepared template must be addressed to the signer"
    );
    assert_eq!(prepared.height(), 1, "first account block has height 1");
    assert_eq!(
        prepared.previous_hash(),
        &Hash::empty(),
        "first account block has an empty previous hash"
    );
    assert_eq!(
        prepared.momentum_acknowledged().height(),
        2,
        "momentum acknowledgement tracks the frontier momentum"
    );
    assert_eq!(
        prepared.fused_plasma(),
        21_000,
        "zero difficulty uses the base plasma"
    );
    assert_eq!(prepared.difficulty(), 0, "zero difficulty is recorded");
    assert_eq!(
        prepared.nonce(),
        "0000000000000000",
        "zero difficulty yields the canonical zero nonce"
    );
    assert_eq!(
        prepared.public_key(),
        keypair.public_key(),
        "prepared template carries the signer public key"
    );
    let expected_hash = block::get_transaction_hash(&prepared).expect("hash computes");
    assert_eq!(
        prepared.hash(),
        &expected_hash,
        "prepared hash matches the transaction hash"
    );
    let signature: [u8; 64] = prepared.signature().try_into().expect("64-byte signature");
    assert!(
        keypair.verify(&signature, expected_hash.bytes()),
        "prepared signature must verify against the hash"
    );
}

#[tokio::test]
async fn send_publishes_the_same_template_as_prepare_block() {
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

    // `send` must publish the same template that `prepare_block` produces.
    let prepared = zenon
        .prepare_block(&template, Some(&keypair))
        .await
        .expect("prepare_block returns a template");
    let _ = zenon
        .send(&template, Some(&keypair))
        .await
        .expect("send publishes");

    let publish_calls = publish_calls.expect("publish capture");
    let calls = publish_calls.lock().expect("calls").clone();
    let published = calls[0][0].clone();

    // `send` delegates to `prepare_block`, so the published template must match
    // the separately prepared template exactly.
    assert_eq!(
        prepared.to_json(),
        published,
        "send must publish the exact template produced by prepare_block"
    );
}

#[tokio::test]
async fn prepare_block_without_a_key_pair_is_rejected() {
    let zenon = Zenon::new();
    let template = AccountBlockTemplate::new(BlockType::UserSend);

    let result = zenon.prepare_block(&template, None).await;

    assert!(
        matches!(result, Err(Error::Generic(ref message)) if message == "No default wallet account selected"),
        "prepare_block without a selected key pair must return the exact no-keypair error, got {result:?}"
    );
}

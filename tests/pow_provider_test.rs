//! Integration tests for the pluggable `PoW` provider.
//!
//! Non-zero difficulty is routed through the configured provider, zero
//! difficulty bypasses it, and provider output is verified before signing.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod support;

use num_bigint::BigUint;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use support::{MockNode, capture_calls, connect};
use znn_sdk_rust::client::websocket::WsClient;
use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::nom::account_block_template::AccountBlockTemplate;
use znn_sdk_rust::pow;
use znn_sdk_rust::pow::provider::{PowFuture, PowProvider};
use znn_sdk_rust::primitives::address::Address;
use znn_sdk_rust::primitives::hash::Hash;
use znn_sdk_rust::primitives::token_standard::znn_token_standard;
use znn_sdk_rust::wallet::keypair::KeyPair;
use znn_sdk_rust::zenon::Zenon;

#[test]
fn zenon_with_ws_client_remains_send_and_sync() {
    fn assert_send<T: Send>(_: T) {}
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Zenon<WsClient>>();

    let zenon = Zenon::new();
    let template = send_template();
    let keypair = KeyPair::from_private_key([7u8; 32]);
    assert_send(zenon.prepare_block(&template, Some(&keypair)));
}

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

/// A provider that always returns a fixed nonce, ignoring the request.
struct FixedNonceProvider([u8; 8]);

impl PowProvider for FixedNonceProvider {
    fn generate_pow<'a>(&'a self, _data_hash: &'a Hash, _difficulty: u64) -> PowFuture<'a> {
        let nonce = self.0;
        Box::pin(async move { Ok(nonce) })
    }
}

/// A provider that records how many times it was asked for a nonce.
struct CountingProvider {
    calls: Arc<AtomicU32>,
}

impl CountingProvider {
    fn new() -> (Self, Arc<AtomicU32>) {
        let calls = Arc::new(AtomicU32::new(0));
        (
            Self {
                calls: calls.clone(),
            },
            calls,
        )
    }
}

impl PowProvider for CountingProvider {
    fn generate_pow<'a>(&'a self, _data_hash: &'a Hash, _difficulty: u64) -> PowFuture<'a> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Box::pin(async move { Ok([0u8; 8]) })
    }
}

#[tokio::test]
async fn non_zero_difficulty_uses_the_configured_provider() {
    let keypair = KeyPair::from_private_key([7u8; 32]);
    let address = keypair.address().expect("address derives");
    // The canonical account-block PoW data hash is deterministic for a known
    // signer and an empty previous hash (no frontier account block).
    let data_hash = pow::account_block_data_hash(&address, &Hash::empty());
    let nonce = pow::generate_pow(&data_hash, 100);
    let expected_nonce = const_hex::encode(nonce);

    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 8400, "basePlasma": 21_000, "requiredDifficulty": 100}),
        );
    })
    .await;
    let mut zenon = Zenon::from_client(connect(&node.url).await);
    zenon.set_pow_provider(Box::new(FixedNonceProvider(nonce)));

    let prepared = zenon
        .prepare_block(&send_template(), Some(&keypair))
        .await
        .expect("prepare_block returns a template");

    assert_eq!(
        prepared.difficulty(),
        100,
        "non-zero difficulty must be recorded from the plasma response"
    );
    assert_eq!(
        prepared.fused_plasma(),
        8400,
        "non-zero difficulty uses the available plasma"
    );
    assert_eq!(
        prepared.nonce(),
        expected_nonce,
        "the configured provider nonce must be placed on the template"
    );
}

#[tokio::test]
async fn zero_difficulty_bypasses_the_provider() {
    let (provider, calls) = CountingProvider::new();
    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 0, "basePlasma": 21_000, "requiredDifficulty": 0}),
        );
    })
    .await;
    let mut zenon = Zenon::from_client(connect(&node.url).await);
    zenon.set_pow_provider(Box::new(provider));
    let keypair = KeyPair::from_private_key([7u8; 32]);

    let prepared = zenon
        .prepare_block(&send_template(), Some(&keypair))
        .await
        .expect("prepare_block returns a template");

    assert_eq!(
        prepared.nonce(),
        "0000000000000000",
        "zero difficulty yields the canonical zero nonce"
    );
    assert_eq!(
        prepared.difficulty(),
        0,
        "zero difficulty is recorded as zero"
    );
    assert_eq!(
        calls.load(Ordering::Relaxed),
        0,
        "zero difficulty must not invoke the provider"
    );
}

#[tokio::test]
async fn an_invalid_provider_nonce_is_rejected() {
    let keypair = KeyPair::from_private_key([7u8; 32]);
    let address = keypair.address().expect("address derives");
    let data_hash = pow::account_block_data_hash(&address, &Hash::empty());
    let invalid_nonce = [0u8; 8];
    // Deterministically pick the smallest power-of-two difficulty at which the
    // fixed nonce fails verification for this data hash.
    let mut difficulty = 1u64;
    while pow::verify_pow(&data_hash, &invalid_nonce, difficulty) {
        difficulty = difficulty.checked_mul(2).expect("difficulty fits u64");
    }

    let mut publish_calls = None;
    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 8400, "basePlasma": 21_000, "requiredDifficulty": difficulty}),
        );
        publish_calls = Some(capture_calls(
            module,
            "ledger.publishRawTransaction",
            Value::Null,
        ));
    })
    .await;
    let mut zenon = Zenon::from_client(connect(&node.url).await);
    zenon.set_pow_provider(Box::new(FixedNonceProvider(invalid_nonce)));

    let result = zenon.prepare_block(&send_template(), Some(&keypair)).await;

    assert!(
        matches!(result, Err(Error::InvalidInput(_))),
        "an invalid provider nonce must be rejected with Error::InvalidInput, got {result:?}"
    );
    assert_eq!(
        publish_calls
            .expect("publish capture")
            .lock()
            .expect("calls")
            .len(),
        0,
        "an invalid provider nonce must not publish a signed template"
    );
}

/// With no provider configured and `native-pow` enabled, nonce resolution falls
/// back to the default factory's native provider and produces a nonce that
/// satisfies the requested difficulty. The nonce is verified against the
/// canonical account-block proof-of-work data hash, so a wrong or zero nonce
/// fails the assertion.
#[cfg(feature = "native-pow")]
#[tokio::test]
async fn no_configured_provider_falls_back_to_the_native_default() {
    let keypair = KeyPair::from_private_key([7u8; 32]);
    let address = keypair.address().expect("address derives");
    let data_hash = pow::account_block_data_hash(&address, &Hash::empty());
    let difficulty: u32 = 100;

    let node = MockNode::spawn(|module| {
        capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
        capture_calls(module, "ledger.getFrontierMomentum", momentum_json());
        capture_calls(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            json!({"availablePlasma": 8400, "basePlasma": 21_000, "requiredDifficulty": difficulty}),
        );
    })
    .await;
    // No `set_pow_provider`: resolution must go through the default factory.
    let zenon = Zenon::from_client(connect(&node.url).await);

    let prepared = zenon
        .prepare_block(&send_template(), Some(&keypair))
        .await
        .expect("prepare_block returns a template via the default provider");

    assert_eq!(
        prepared.difficulty(),
        difficulty,
        "non-zero difficulty must be recorded from the plasma response"
    );
    let nonce_bytes: [u8; 8] = const_hex::decode(prepared.nonce())
        .expect("nonce is hex")
        .try_into()
        .expect("nonce is 8 bytes");
    assert!(
        pow::verify_pow(&data_hash, &nonce_bytes, u64::from(difficulty)),
        "the default provider's nonce must satisfy the requested difficulty"
    );
}

//! Opt-in live testnet smoke tests.
//!
//! These tests are ignored by default. See docs/live-testnet.md.
#![allow(clippy::expect_used)]

use num_bigint::BigUint;
use znn_sdk_rust::model::nom::account_block_template::AccountBlockTemplate;
use znn_sdk_rust::primitives::token_standard::znn_token_standard;
use znn_sdk_rust::wallet::keystore::KeyStore;
use znn_sdk_rust::zenon::Zenon;

fn testnet_url() -> Option<String> {
    std::env::var("ZNN_TESTNET_NODE_URL").ok()
}

fn testnet_mnemonic() -> Option<String> {
    std::env::var("ZNN_TESTNET_MNEMONIC").ok()
}

#[tokio::test]
#[ignore = "requires ZNN_TESTNET_NODE_URL and a reachable testnet node"]
async fn live_frontier_momentum_on_testnet() {
    let Some(url) = testnet_url() else {
        eprintln!("skipping: set ZNN_TESTNET_NODE_URL");
        return;
    };

    let zenon = Zenon::connect(&url, false)
        .await
        .expect("connects to testnet node");
    let frontier = zenon
        .ledger
        .get_frontier_momentum()
        .await
        .expect("frontier momentum is returned");

    println!(
        "frontier height={} hash={}",
        frontier.height(),
        frontier.hash()
    );
}

#[tokio::test]
#[ignore = "requires ZNN_TESTNET_NODE_URL, ZNN_TESTNET_MNEMONIC, ZNN_TESTNET_SEND=1, and funded testnet account"]
async fn live_send_self_on_testnet() {
    if std::env::var("ZNN_TESTNET_SEND").ok().as_deref() != Some("1") {
        eprintln!("skipping: set ZNN_TESTNET_SEND=1 to publish a real testnet block");
        return;
    }
    let Some(url) = testnet_url() else {
        eprintln!("skipping: set ZNN_TESTNET_NODE_URL");
        return;
    };
    let Some(mnemonic) = testnet_mnemonic() else {
        eprintln!("skipping: set ZNN_TESTNET_MNEMONIC");
        return;
    };

    let zenon = Zenon::connect(&url, false)
        .await
        .expect("connects to testnet node");
    let store = KeyStore::from_mnemonic(&mnemonic).expect("mnemonic opens");
    let key_pair = store.get_key_pair(0).expect("account derives");
    let address = key_pair.address().expect("address derives");
    let template = AccountBlockTemplate::send(
        address.clone(),
        znn_token_standard(),
        BigUint::from(1u32),
        None,
    );

    println!("publishing one smallest-unit ZNN self-send from {address}");
    let response = zenon
        .send(&template, Some(&key_pair))
        .await
        .expect("testnet publish succeeds");
    println!("publish response={response}");
}

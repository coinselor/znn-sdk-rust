//! Build and optionally publish a simple ZNN transfer.
//!
//! By default this example only builds the template and prints the sender/receiver
//! information. Set `ZNN_NODE_URL` and `ZNN_MNEMONIC`, then pass `--publish` to
//! publish on a testnet/devnet node.

use num_bigint::BigUint;
use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::nom::account_block_template::AccountBlockTemplate;
use znn_sdk_rust::primitives::address::Address;
use znn_sdk_rust::primitives::token_standard::znn_token_standard;
use znn_sdk_rust::wallet::keystore::KeyStore;
use znn_sdk_rust::zenon::Zenon;

/// Smallest ZNN unit sent by the example.
pub const EXAMPLE_AMOUNT: u32 = 1;

/// Builds a self-send template for `mnemonic` account 0.
pub fn build_self_send_template(mnemonic: &str) -> Result<AccountBlockTemplate, Error> {
    let store = KeyStore::from_mnemonic(mnemonic)?;
    let key_pair = store.get_key_pair(0)?;
    let to = key_pair.address()?;
    Ok(AccountBlockTemplate::send(
        to,
        znn_token_standard(),
        BigUint::from(EXAMPLE_AMOUNT),
        None,
    ))
}

/// Publishes a one-unit self-send using the given node URL and mnemonic.
#[allow(dead_code)]
pub async fn publish_self_send(url: &str, mnemonic: &str) -> Result<serde_json::Value, Error> {
    let zenon = Zenon::connect(url, false).await.map_err(Error::from)?;
    let store = KeyStore::from_mnemonic(mnemonic)?;
    let key_pair = store.get_key_pair(0)?;
    let template = AccountBlockTemplate::send(
        key_pair.address()?,
        znn_token_standard(),
        BigUint::from(EXAMPLE_AMOUNT),
        None,
    );
    zenon.send(&template, Some(&key_pair)).await
}

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = std::env::var("ZNN_MNEMONIC").unwrap_or_else(|_| {
        "route become dream access impulse price inform obtain engage ski believe awful absent pig thing vibrant possible exotic flee pepper marble rural fire fancy".to_string()
    });
    let store = KeyStore::from_mnemonic(&mnemonic)?;
    let key_pair = store.get_key_pair(0)?;
    let to = std::env::var("ZNN_TO_ADDRESS")
        .ok()
        .map(|value| Address::parse(&value))
        .transpose()?
        .unwrap_or(key_pair.address()?);
    let template = AccountBlockTemplate::send(
        to,
        znn_token_standard(),
        BigUint::from(EXAMPLE_AMOUNT),
        None,
    );

    println!("from={}", key_pair.address()?);
    println!("to={}", template.to_address());
    println!("amount={} smallest ZNN unit(s)", template.amount());

    if std::env::args().any(|arg| arg == "--publish") {
        let url = std::env::var("ZNN_NODE_URL")?;
        let zenon = Zenon::connect(&url, false).await?;
        let response = zenon.send(&template, Some(&key_pair)).await?;
        println!("publish response={response}");
    } else {
        println!(
            "dry run only; pass --publish with ZNN_NODE_URL and a funded testnet ZNN_MNEMONIC to publish"
        );
    }

    Ok(())
}

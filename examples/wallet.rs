//! Wallet key-derivation example.

use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::nom::account_block_template::DEFAULT_CHAIN_IDENTIFIER;
use znn_sdk_rust::wallet::keystore::KeyStore;

/// Canonical example mnemonic used by the reference SDK compatibility vectors.
pub const MNEMONIC: &str = "route become dream access impulse price inform obtain engage ski believe awful absent pig thing vibrant possible exotic flee pepper marble rural fire fancy";

/// Output produced by the wallet example.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletReport {
    /// BIP-39 entropy in hex.
    pub entropy: String,
    /// Private key in hex.
    pub private_key: String,
    /// Public key in hex.
    pub public_key: String,
    /// Zenon address.
    pub address: String,
    /// Address core bytes in hex.
    pub address_core: String,
    /// Chain identifier.
    pub chain_identifier: u32,
}

/// Derives the canonical example account.
pub fn derive_example_wallet() -> Result<WalletReport, Error> {
    let store = KeyStore::from_mnemonic(MNEMONIC)?;
    let key_pair = store.get_key_pair(0)?;
    let address = key_pair.address()?;

    Ok(WalletReport {
        entropy: store.entropy().to_string(),
        private_key: const_hex::encode(key_pair.private_key()),
        public_key: const_hex::encode(key_pair.public_key()),
        address: address.to_string(),
        address_core: const_hex::encode(address.core()),
        chain_identifier: DEFAULT_CHAIN_IDENTIFIER,
    })
}

#[allow(dead_code)]
fn main() -> Result<(), Error> {
    let report = derive_example_wallet()?;
    println!("entropy={}", report.entropy);
    println!("private_key={}", report.private_key);
    println!("public_key={}", report.public_key);
    println!("address={}", report.address);
    println!("address_core={}", report.address_core);
    println!("chain_identifier={}", report.chain_identifier);
    Ok(())
}

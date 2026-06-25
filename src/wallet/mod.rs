//! Wallet: mnemonic, keystore, keypair, encrypted files.
//!
//! Mnemonic, entropy/seed, BIP-44 key derivation, and account key pairs are
//! always available as deterministic core crypto. Filesystem keystore
//! persistence (the encrypted-file format and the keystore manager) is gated
//! behind the `wallet-fs` feature.

pub mod constants;
pub mod derivation;
pub mod exceptions;
pub mod interfaces;
pub mod keypair;
pub mod keystore;
pub mod mnemonic;

#[cfg(feature = "wallet-fs")]
pub mod encrypted_file;

#[cfg(feature = "wallet-fs")]
pub mod manager;

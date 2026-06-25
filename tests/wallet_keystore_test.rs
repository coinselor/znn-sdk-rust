//! Integration tests for the keystore: mnemonic/entropy/seed derivation and
//! account key-pair derivation.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

use znn_sdk_rust::Error;
use znn_sdk_rust::wallet::interfaces::WalletDefinition;
use znn_sdk_rust::wallet::keystore::{KeyStore, KeyStoreDefinition};

/// A known-valid 24-word English BIP-39 mnemonic (256-bit entropy).
const VALID_MNEMONIC: &str = "route become dream access impulse price inform obtain engage ski \
believe awful absent pig thing vibrant possible exotic flee pepper marble rural fire fancy";

/// Valid 256-bit BIP-39 entropy in hex (mixed case, to test lowercasing).
const VALID_ENTROPY_UPPER: &str =
    "000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F";

fn is_lower_hex(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[test]
fn from_mnemonic_derives_entropy_and_seed() {
    let ks = KeyStore::from_mnemonic(VALID_MNEMONIC).expect("valid mnemonic");
    assert_eq!(ks.mnemonic(), Some(VALID_MNEMONIC), "normalized mnemonic");
    assert!(
        is_lower_hex(ks.entropy()),
        "entropy must be lowercase hex, got {:?}",
        ks.entropy()
    );
    let seed = ks.seed().expect("seed is set");
    assert!(is_lower_hex(seed), "seed must be lowercase hex");
    assert_eq!(seed.len(), 128, "a 64-byte seed is 128 hex characters");
}

#[test]
fn from_mnemonic_rejects_an_invalid_mnemonic() {
    let result = KeyStore::from_mnemonic("this is not a valid bip39 mnemonic");
    assert!(
        matches!(result, Err(Error::InvalidInput(_))),
        "an invalid mnemonic must be rejected with InvalidInput, got {result:?}"
    );
}

#[test]
fn from_entropy_lowercases_and_derives() {
    let ks = KeyStore::from_entropy(VALID_ENTROPY_UPPER).expect("valid entropy");
    assert_eq!(
        ks.entropy(),
        VALID_ENTROPY_UPPER.to_lowercase(),
        "entropy is stored lowercase"
    );
    assert!(
        ks.mnemonic().is_some_and(|m| !m.is_empty()),
        "a mnemonic is derived from entropy"
    );
    let seed = ks.seed().expect("seed is set");
    assert_eq!(seed.len(), 128, "a 64-byte seed is 128 hex characters");
}

#[test]
fn from_entropy_rejects_invalid_hex_and_length() {
    let non_hex = KeyStore::from_entropy("zzzz");
    assert!(
        matches!(non_hex, Err(Error::InvalidInput(_))),
        "non-hex entropy must be rejected with InvalidInput, got {non_hex:?}"
    );
    let bad_length = KeyStore::from_entropy("00");
    assert!(
        matches!(bad_length, Err(Error::InvalidInput(_))),
        "8-bit entropy is an invalid length and must return InvalidInput, got {bad_length:?}"
    );
}

#[test]
fn from_seed_stores_the_seed() {
    let seed = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let ks = KeyStore::from_seed(seed).expect("valid seed");
    assert_eq!(ks.seed(), Some(seed), "from_seed stores the given seed");
}

#[test]
fn from_seed_rejects_invalid_seed_input() {
    let non_hex = KeyStore::from_seed("not-hex");
    assert!(
        matches!(non_hex, Err(Error::InvalidInput(_))),
        "non-hex seeds must be rejected with InvalidInput, got {non_hex:?}"
    );
    let empty = KeyStore::from_seed("");
    assert!(
        matches!(empty, Err(Error::InvalidInput(_))),
        "empty seeds must be rejected with InvalidInput, got {empty:?}"
    );
}

#[test]
fn get_key_pair_is_deterministic_and_index_dependent() {
    let ks = KeyStore::from_mnemonic(VALID_MNEMONIC).expect("valid mnemonic");
    let a = ks.get_key_pair(0).expect("index 0");
    let b = ks.get_key_pair(0).expect("index 0 again");
    let c = ks.get_key_pair(1).expect("index 1");
    assert_eq!(
        a.private_key(),
        b.private_key(),
        "the same index yields the same key"
    );
    assert_ne!(
        a.private_key(),
        c.private_key(),
        "different indices yield different keys"
    );
}

#[test]
fn derive_addresses_by_range_returns_one_per_index() {
    let ks = KeyStore::from_mnemonic(VALID_MNEMONIC).expect("valid mnemonic");
    let addresses = ks.derive_addresses_by_range(0, 3).expect("range derives");
    assert_eq!(addresses.len(), 3, "one address per index in [0, 3)");
}

#[test]
fn find_address_locates_a_derived_account() {
    let ks = KeyStore::from_mnemonic(VALID_MNEMONIC).expect("valid mnemonic");
    let address = ks
        .get_key_pair(0)
        .expect("index 0")
        .address()
        .expect("address");
    let found = ks.find_address(&address, 5).expect("search runs");
    assert_eq!(found, Some(0), "the index-0 address is found at index 0");
}

#[test]
fn keystore_definition_exposes_id_and_name() {
    let dir = std::env::temp_dir().join("znn_keystore_def_test");
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("my_wallet");
    fs::write(&path, b"{}").expect("write keystore file");

    let def = KeyStoreDefinition::new(path.clone()).expect("existing file");
    assert_eq!(
        def.wallet_id(),
        path.to_string_lossy(),
        "wallet id is the file path"
    );
    assert_eq!(
        def.wallet_name(),
        "my_wallet",
        "wallet name is the base name"
    );

    fs::remove_file(&path).ok();
}

#[test]
fn keystore_definition_rejects_a_missing_file() {
    let missing = PathBuf::from("/this/path/does/not/exist/znn_missing_keystore");
    assert!(
        KeyStoreDefinition::new(missing).is_err(),
        "a non-existent keystore file must be rejected"
    );
}

//! Compatibility tests for wallet derivation, keystore parsing, and encrypted
//! file decryption against the bundled Syrius fixture.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use znn_sdk_rust::wallet::encrypted_file::EncryptedFile;
use znn_sdk_rust::wallet::keystore::KeyStore;
use znn_sdk_rust::wallet::manager::KeyStoreManager;

const FIXTURE_JSON: &str = include_str!("vectors/wallet/syrius/compat.json");
const UPSTREAM_REPO: &str = "https://github.com/zenon-network/znn_sdk_dart";
const UPSTREAM_COMMIT: &str = "b21f0f866f4a0d5ae15d86c76b7ae414ca0b973a";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    provenance: Provenance,
    mnemonic: String,
    entropy: String,
    seed: String,
    accounts: Vec<AccountVector>,
    encrypted_keystore: EncryptedKeystoreVector,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Provenance {
    upstream_repo: String,
    upstream_commit: String,
    upstream_branch: String,
    generator: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountVector {
    index: u32,
    derivation_path: String,
    private_key: String,
    public_key: String,
    address: String,
    address_core: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptedKeystoreVector {
    password: String,
    wallet_name: String,
    expected_entropy: String,
    json: Value,
}

fn fixture() -> Fixture {
    serde_json::from_str(FIXTURE_JSON).expect("valid Syrius compatibility fixture")
}

fn assert_provenance(provenance: &Provenance) {
    assert_eq!(
        provenance.upstream_repo, UPSTREAM_REPO,
        "fixture must come from the expected upstream repository"
    );
    assert_eq!(
        provenance.upstream_commit, UPSTREAM_COMMIT,
        "fixture must be pinned to the expected upstream commit"
    );
    assert_eq!(provenance.upstream_branch, "master", "upstream branch");
    assert_eq!(provenance.generator, "dart run generate.dart", "generator");
}

fn temp_wallet_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("znn_syrius_compat_{tag}_{}", std::process::id()));
    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir).expect("create temp wallet dir");
    dir
}

#[test]
fn mnemonic_derivation_matches_fixture_vectors() {
    let fixture = fixture();
    assert_provenance(&fixture.provenance);

    let store = KeyStore::from_mnemonic(&fixture.mnemonic).expect("fixture mnemonic is valid");
    assert_eq!(store.entropy(), fixture.entropy, "mnemonic entropy");
    assert_eq!(store.seed(), Some(fixture.seed.as_str()), "mnemonic seed");
    assert_eq!(
        fixture.accounts.len(),
        3,
        "fixture covers the first three account indices"
    );

    for account in &fixture.accounts {
        let key_pair = store
            .get_key_pair(account.index)
            .expect("account key derives");
        let address = key_pair.address().expect("address derives");
        assert_eq!(
            account.derivation_path,
            format!("m/44'/73404'/{}'", account.index),
            "derivation path for index {}",
            account.index
        );
        assert_eq!(
            const_hex::encode(key_pair.private_key()),
            account.private_key,
            "private key for index {}",
            account.index
        );
        assert_eq!(
            const_hex::encode(key_pair.public_key()),
            account.public_key,
            "public key for index {}",
            account.index
        );
        assert_eq!(
            address.to_string(),
            account.address,
            "address for index {}",
            account.index
        );
        assert_eq!(
            const_hex::encode(address.core()),
            account.address_core,
            "address core for index {}",
            account.index
        );
    }
}

#[test]
fn encrypted_keystore_decrypts_with_rust_crypto() {
    let fixture = fixture();
    assert_provenance(&fixture.provenance);
    let encrypted_json = serde_json::to_string(&fixture.encrypted_keystore.json)
        .expect("encrypted keystore json serializes");
    let encrypted = EncryptedFile::from_json(&encrypted_json).expect("encrypted keystore parses");
    let entropy = encrypted
        .decrypt(&fixture.encrypted_keystore.password)
        .expect("fixture keystore decrypts");
    assert_eq!(
        const_hex::encode(entropy),
        fixture.encrypted_keystore.expected_entropy,
        "fixture keystore decrypts to the expected entropy"
    );
}

#[test]
fn manager_reads_fixture_keystore() {
    let fixture = fixture();
    assert_provenance(&fixture.provenance);
    let dir = temp_wallet_dir("manager_read");
    let path = dir.join(&fixture.encrypted_keystore.wallet_name);
    let encrypted_json = serde_json::to_string(&fixture.encrypted_keystore.json)
        .expect("encrypted keystore json serializes");
    fs::write(&path, encrypted_json).expect("write fixture keystore");

    let manager = KeyStoreManager::new(dir.clone());
    let store = manager
        .read_key_store(&fixture.encrypted_keystore.password, &path)
        .expect("manager reads fixture keystore");

    assert_eq!(
        store.entropy(),
        fixture.encrypted_keystore.expected_entropy,
        "manager reconstructs the keystore entropy"
    );
    assert_eq!(
        store
            .get_key_pair(0)
            .expect("index 0")
            .address()
            .unwrap()
            .to_string(),
        fixture
            .accounts
            .first()
            .expect("fixture has an index-0 account")
            .address,
        "manager reconstructs the index-0 address"
    );

    fs::remove_dir_all(&dir).ok();
}

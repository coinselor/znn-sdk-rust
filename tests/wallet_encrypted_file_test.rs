//! Integration tests for the encrypted file format: JSON round-trip, metadata
//! extraction, field parsing, and encrypt/decrypt behavior.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use serde_json::{Value, json};

use znn_sdk_rust::wallet::encrypted_file::EncryptedFile;
use znn_sdk_rust::wallet::exceptions::WalletError;

const NO_META: &str = r#"{
  "crypto": {
    "argon2Params": { "salt": "0x5b85100f186953332faddeaf2b6d68de" },
    "cipherData": "0xcfe2e1aa498229aaf78ab9634592a19c1f45ad5178ed4d530fdeef8fe528e4b78955a389e0d1e832fd0c21346b3a45cd",
    "cipherName": "aes-256-gcm",
    "kdf": "argon2.IDKey",
    "nonce": "0x86d6b27ba67a72238af12958"
  },
  "timestamp": 1707418422,
  "version": 1
}"#;

const WITH_META: &str = r#"{
  "baseAddress": "z1qqjnwjjpnue8xmmpanz6csze6tcmtzzdtfsww7",
  "walletType": "keystore",
  "crypto": {
    "argon2Params": { "salt": "0x5b85100f186953332faddeaf2b6d68de" },
    "cipherData": "0xcfe2e1aa498229aaf78ab9634592a19c1f45ad5178ed4d530fdeef8fe528e4b78955a389e0d1e832fd0c21346b3a45cd",
    "cipherName": "aes-256-gcm",
    "kdf": "argon2.IDKey",
    "nonce": "0x86d6b27ba67a72238af12958"
  },
  "timestamp": 1707418422,
  "version": 1
}"#;

fn parsed(json: &str) -> Value {
    serde_json::from_str(json).expect("vector is valid json")
}

#[test]
fn round_trips_a_file_without_metadata() {
    let file = EncryptedFile::from_json(NO_META).expect("parses");
    assert_eq!(
        file.to_json_value().expect("serializes"),
        parsed(NO_META),
        "to_json_value must reproduce the input json"
    );
    let from_string: Value = serde_json::from_str(&file.to_json().expect("serializes to a string"))
        .expect("to_json output is valid json");
    assert_eq!(
        from_string,
        parsed(NO_META),
        "to_json string must reproduce the input json"
    );
}

#[test]
fn file_without_metadata_has_no_metadata() {
    let file = EncryptedFile::from_json(NO_META).expect("parses");
    assert_eq!(file.metadata(), None, "no top-level metadata keys");
}

#[test]
fn round_trips_a_file_with_metadata() {
    let file = EncryptedFile::from_json(WITH_META).expect("parses");
    assert_eq!(
        file.to_json_value().expect("serializes"),
        parsed(WITH_META),
        "to_json_value must reproduce the input json including metadata"
    );
    let from_string: Value = serde_json::from_str(&file.to_json().expect("serializes to a string"))
        .expect("to_json output is valid json");
    assert_eq!(
        from_string,
        parsed(WITH_META),
        "to_json string must reproduce the input json including metadata"
    );
}

#[test]
fn file_with_metadata_exposes_its_metadata() {
    let file = EncryptedFile::from_json(WITH_META).expect("parses");
    let expected = json!({
        "baseAddress": "z1qqjnwjjpnue8xmmpanz6csze6tcmtzzdtfsww7",
        "walletType": "keystore"
    });
    assert_eq!(
        file.metadata().cloned().map(Value::Object),
        Some(expected),
        "metadata must be the non-crypto top-level keys"
    );
}

#[test]
fn parses_the_crypto_section_and_header() {
    let file = EncryptedFile::from_json(NO_META).expect("parses");
    assert_eq!(file.version, Some(1), "version");
    assert_eq!(file.timestamp, Some(1_707_418_422), "timestamp");
    let crypto = file.crypto.expect("crypto section is present");
    assert_eq!(crypto.cipher_name, "aes-256-gcm", "cipher name");
    assert_eq!(crypto.kdf, "argon2.IDKey", "kdf");
    assert_eq!(crypto.nonce.len(), 12, "12-byte nonce");
    assert_eq!(
        crypto.argon2_params.expect("argon2 params").salt.len(),
        16,
        "16-byte salt"
    );
    assert!(
        !crypto.cipher_data.is_empty(),
        "cipher data must be decoded from hex"
    );
}

#[test]
fn encrypt_then_decrypt_round_trips() {
    let plaintext = b"a wallet entropy payload";
    let file = EncryptedFile::encrypt(plaintext, "correct-password", None).expect("encrypts");
    let decrypted = file.decrypt("correct-password").expect("decrypts");
    assert_eq!(
        decrypted.as_slice(),
        plaintext,
        "decrypt must reproduce the encrypted plaintext"
    );
}

#[test]
fn decrypt_with_wrong_password_reports_incorrect_password() {
    let file = EncryptedFile::from_json(NO_META).expect("parses");
    let result = file.decrypt("definitely-the-wrong-password");
    assert!(
        matches!(result, Err(WalletError::IncorrectPassword)),
        "a wrong password must yield IncorrectPassword, got {result:?}"
    );
}

#[test]
fn decrypt_rejects_an_unsupported_cipher_name() {
    let json = NO_META.replace("\"aes-256-gcm\"", "\"aes-128-gcm\"");
    let file = EncryptedFile::from_json(&json).expect("parses");
    let result = file.decrypt("correct-password");
    assert!(
        matches!(result, Err(WalletError::Wallet(ref message)) if message.contains("unsupported cipher")),
        "an unsupported cipher must be rejected, got {result:?}"
    );
}

#[test]
fn decrypt_rejects_an_unsupported_kdf() {
    let json = NO_META.replace("\"argon2.IDKey\"", "\"pbkdf2\"");
    let file = EncryptedFile::from_json(&json).expect("parses");
    let result = file.decrypt("correct-password");
    assert!(
        matches!(result, Err(WalletError::Wallet(ref message)) if message.contains("unsupported kdf")),
        "an unsupported kdf must be rejected, got {result:?}"
    );
}

const UNSUPPORTED_CIPHER: &str = r#"{
  "crypto": {
    "argon2Params": { "salt": "0x5b85100f186953332faddeaf2b6d68de" },
    "cipherData": "0xcfe2e1aa498229aaf78ab9634592a19c1f45ad5178ed4d530fdeef8fe528e4b78955a389e0d1e832fd0c21346b3a45cd",
    "cipherName": "chacha20-poly1305",
    "kdf": "argon2.IDKey",
    "nonce": "0x86d6b27ba67a72238af12958"
  },
  "timestamp": 1707418422,
  "version": 1
}"#;

#[test]
fn decrypt_rejects_an_unsupported_cipher() {
    let file = EncryptedFile::from_json(UNSUPPORTED_CIPHER).expect("parses");
    let result = file.decrypt("any-password");
    assert!(
        matches!(result, Err(WalletError::Wallet(_))),
        "an unsupported cipher must be reported distinctly from a wrong password, got {result:?}"
    );
}

const UNSUPPORTED_VERSION: &str = r#"{
  "crypto": {
    "argon2Params": { "salt": "0x5b85100f186953332faddeaf2b6d68de" },
    "cipherData": "0xcfe2e1aa498229aaf78ab9634592a19c1f45ad5178ed4d530fdeef8fe528e4b78955a389e0d1e832fd0c21346b3a45cd",
    "cipherName": "aes-256-gcm",
    "kdf": "argon2.IDKey",
    "nonce": "0x86d6b27ba67a72238af12958"
  },
  "timestamp": 1707418422,
  "version": 2
}"#;

const SHORT_SALT: &str = r#"{
  "crypto": {
    "argon2Params": { "salt": "0x5b85100f18695333" },
    "cipherData": "0xcfe2e1aa498229aaf78ab9634592a19c1f45ad5178ed4d530fdeef8fe528e4b78955a389e0d1e832fd0c21346b3a45cd",
    "cipherName": "aes-256-gcm",
    "kdf": "argon2.IDKey",
    "nonce": "0x86d6b27ba67a72238af12958"
  },
  "timestamp": 1707418422,
  "version": 1
}"#;

const SHORT_CIPHER_DATA: &str = r#"{
  "crypto": {
    "argon2Params": { "salt": "0x5b85100f186953332faddeaf2b6d68de" },
    "cipherData": "0xcfe2e1aa498229aa",
    "cipherName": "aes-256-gcm",
    "kdf": "argon2.IDKey",
    "nonce": "0x86d6b27ba67a72238af12958"
  },
  "timestamp": 1707418422,
  "version": 1
}"#;

#[test]
fn decrypt_rejects_an_unsupported_version() {
    let file = EncryptedFile::from_json(UNSUPPORTED_VERSION).expect("parses");
    let result = file.decrypt("any-password");
    assert!(
        matches!(result, Err(WalletError::Wallet(_))),
        "an unsupported version must be rejected, got {result:?}"
    );
}

#[test]
fn decrypt_rejects_a_bad_salt_length() {
    let file = EncryptedFile::from_json(SHORT_SALT).expect("parses");
    let result = file.decrypt("any-password");
    assert!(
        matches!(result, Err(WalletError::Wallet(_))),
        "a salt that is not 16 bytes must be rejected, got {result:?}"
    );
}

#[test]
fn decrypt_rejects_truncated_cipher_data_as_malformed() {
    let file = EncryptedFile::from_json(SHORT_CIPHER_DATA).expect("parses");
    let result = file.decrypt("any-password");
    assert!(
        matches!(result, Err(WalletError::Wallet(ref message)) if message.contains("authentication tag")),
        "truncated cipherData must be reported as malformed, got {result:?}"
    );
}

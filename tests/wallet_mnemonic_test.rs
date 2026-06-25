//! Integration tests for English BIP-39 mnemonic helpers.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use znn_sdk_rust::Error;
use znn_sdk_rust::wallet::mnemonic::{
    VALID_STRENGTHS, generate_mnemonic, is_valid_word, validate_mnemonic,
};

/// A known-valid 24-word English BIP-39 mnemonic (256-bit entropy).
const VALID_MNEMONIC: &str = "route become dream access impulse price inform obtain engage ski \
believe awful absent pig thing vibrant possible exotic flee pepper marble rural fire fancy";

fn words(sentence: &str) -> Vec<&str> {
    sentence.split_whitespace().collect()
}

#[test]
fn generate_mnemonic_returns_valid_mnemonic_for_supported_strengths() {
    for strength in VALID_STRENGTHS {
        let sentence = generate_mnemonic(strength).expect("supported strength generates");
        let words = words(&sentence);
        let expected_len = (strength / 32 * 3) as usize;
        assert_eq!(
            words.len(),
            expected_len,
            "mnemonic for {strength}-bit entropy must have {expected_len} words"
        );
        assert!(
            validate_mnemonic(&words),
            "generated mnemonic for {strength} must validate"
        );
        for w in &words {
            assert!(is_valid_word(w), "generated word not in wordlist: {w}");
        }
    }
}

#[test]
fn generate_mnemonic_rejects_unsupported_strength() {
    let result = generate_mnemonic(100);
    assert!(
        matches!(result, Err(Error::InvalidInput(_))),
        "100 must be rejected with InvalidInput, got {result:?}"
    );
}

#[test]
fn validate_mnemonic_accepts_a_known_valid_mnemonic() {
    assert!(
        validate_mnemonic(&words(VALID_MNEMONIC)),
        "the canonical 24-word mnemonic must validate"
    );
}

#[test]
fn validate_mnemonic_rejects_unknown_words() {
    assert!(
        !validate_mnemonic(&["notaword", "alsofake"]),
        "words outside the wordlist must not validate"
    );
}

#[test]
fn validate_mnemonic_rejects_valid_words_with_a_bad_checksum() {
    // Every word is in the English wordlist, but the BIP-39 checksum is wrong:
    // the valid all-`abandon` 12-word mnemonic ends in `about`, so twelve
    // `abandon` words have a correct word set and an incorrect checksum. A
    // validator that only checks word membership would wrongly accept this, so
    // this case pins checksum verification.
    let all_abandon = ["abandon"; 12];
    assert!(
        !validate_mnemonic(&all_abandon),
        "valid words with a bad checksum must not validate"
    );
}

#[test]
fn is_valid_word_accepts_wordlist_endpoints() {
    // Pins membership across both ends of the English BIP-39 wordlist: `abandon`
    // is the first word and `zoo` the last. Backed by the `bip39` crate's list,
    // not a vendored copy, so these words must keep validating after the reback.
    assert!(is_valid_word("abandon"), "abandon is in the wordlist");
    assert!(is_valid_word("zoo"), "zoo is in the wordlist");
}

#[test]
fn is_valid_word_rejects_non_words() {
    assert!(!is_valid_word("abandon!"), "abandon! is not a word");
    assert!(!is_valid_word(""), "the empty string is not a word");
}

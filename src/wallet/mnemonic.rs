//! BIP-39 mnemonic helpers (English wordlist only).

use crate::error::Error;
use bip39::{Language, Mnemonic};

/// Valid BIP-39 entropy strengths, in bits.
pub const VALID_STRENGTHS: [u32; 5] = [128, 160, 192, 224, 256];

/// Generates a new English BIP-39 mnemonic for the given entropy `strength`.
///
/// `strength` must be one of [`VALID_STRENGTHS`]; any other value returns
/// [`Error::InvalidInput`]. The returned sentence has `strength / 32 * 3`
/// space-separated words.
pub fn generate_mnemonic(strength: u32) -> Result<String, Error> {
    if !VALID_STRENGTHS.contains(&strength) {
        return Err(Error::InvalidInput(format!(
            "invalid mnemonic strength: {strength}"
        )));
    }
    let word_count = (strength / 32 * 3) as usize;
    let mnemonic = Mnemonic::generate_in(Language::English, word_count)
        .map_err(|e| Error::InvalidInput(format!("mnemonic generation failed: {e}")))?;
    Ok(mnemonic.to_string())
}

/// Validates an English BIP-39 mnemonic given as a list of words.
///
/// Returns `true` only when the words are all in the English wordlist and the
/// BIP-39 checksum is correct.
pub fn validate_mnemonic(words: &[&str]) -> bool {
    let sentence = words.join(" ");
    Mnemonic::parse_in_normalized(Language::English, &sentence).is_ok()
}

/// Returns `true` if `word` is in the English BIP-39 wordlist.
pub fn is_valid_word(word: &str) -> bool {
    Language::English.word_list().contains(&word)
}

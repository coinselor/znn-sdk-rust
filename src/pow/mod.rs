//! Account-block proof-of-work generation.
//!
//! The nonce is found when the little-endian `u64` read from the first 8 bytes of
//! `sha3-256(nonce_le || data_hash)` is at least the difficulty threshold.

pub mod provider;

use crate::crypto::crypto;
use crate::primitives::address::Address;
use crate::primitives::hash::{Hash, LENGTH as HASH_LENGTH};
use crate::utils::bytes::merge;

/// Proof-of-work generation progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowStatus {
    /// The nonce search is in progress.
    Generating,
    /// The nonce search has finished.
    Done,
}

/// Returns the proof-of-work threshold for `difficulty`:
/// `2^64 - floor(2^64 / difficulty)`, and `0` for a zero difficulty.
#[allow(clippy::cast_possible_truncation)]
pub fn threshold(difficulty: u64) -> u64 {
    if difficulty == 0 {
        return 0;
    }
    let two_pow_64: u128 = 1u128 << 64;
    let dividend = u128::from(difficulty);
    // `two_pow_64 - two_pow_64 / difficulty` is always in `[0, 2^64)` because the
    // divisor is at least `1`, so the cast back to `u64` is lossless.
    (two_pow_64 - two_pow_64 / dividend) as u64
}

/// Returns the proof-of-work data hash for an account block: `sha3-256` of the address
/// core followed by the previous hash.
pub fn account_block_data_hash(address: &Address, previous_hash: &Hash) -> Hash {
    let data = merge(&[address.core(), previous_hash.bytes()]);
    digest_to_hash(&data)
}

/// Verifies that `nonce` satisfies the proof-of-work `difficulty` for `data_hash`.
///
/// A nonce qualifies when the little-endian `u64` read from the first 8 bytes
/// of `sha3-256(nonce_le || data_hash)` is at least `threshold(difficulty)`; a
/// zero threshold (difficulty `0` or `1`) accepts every nonce.
pub fn verify_pow(data_hash: &Hash, nonce: &[u8; 8], difficulty: u64) -> bool {
    let target = threshold(difficulty);
    if target == 0 {
        return true;
    }
    let mut input = [0u8; 8 + HASH_LENGTH];
    input[..8].copy_from_slice(nonce);
    input[8..].copy_from_slice(data_hash.bytes());
    let hash = crypto::digest(&input);
    digest_prefix_u64(&hash) >= target
}

/// Searches for an 8-byte nonce satisfying the proof-of-work `difficulty` for
/// `data_hash`.
///
/// The search starts from a random nonce and increments it in little-endian
/// order until a qualifying nonce is found.
///
/// This randomized, in-process search is gated behind the `native-pow` feature.
/// Hosts without a native RNG (for example browser/WASM targets) should inject a
/// [`crate::pow::provider::PowProvider`] instead.
#[cfg(feature = "native-pow")]
pub fn generate_pow(data_hash: &Hash, difficulty: u64) -> [u8; 8] {
    let mut nonce = random_nonce();
    while !verify_pow(data_hash, &nonce, difficulty) {
        quick_inc(&mut nonce);
    }
    nonce
}

/// Searches for a nonce over the empty hash.
#[cfg(feature = "native-pow")]
pub fn benchmark_pow(difficulty: u64) -> [u8; 8] {
    generate_pow(&Hash::empty(), difficulty)
}

#[allow(clippy::expect_used)]
fn digest_to_hash(data: &[u8]) -> Hash {
    Hash::from_bytes(&crypto::digest(data)).expect("sha3-256 yields 32 bytes")
}

#[allow(clippy::expect_used)]
fn digest_prefix_u64(hash: &[u8; HASH_LENGTH]) -> u64 {
    let prefix: &[u8; 8] = hash
        .first_chunk::<8>()
        .expect("sha3-256 yields at least 8 bytes");
    u64::from_le_bytes(*prefix)
}

/// Little-endian increment-with-carry.
#[cfg(feature = "native-pow")]
fn quick_inc(nonce: &mut [u8; 8]) {
    for byte in nonce.iter_mut() {
        *byte = byte.wrapping_add(1);
        if *byte != 0 {
            return;
        }
    }
}

/// Returns a random starting nonce, falling back to all zeros if the platform
/// entropy source is unavailable (the incrementing search still converges).
#[cfg(feature = "native-pow")]
fn random_nonce() -> [u8; 8] {
    let mut nonce = [0u8; 8];
    let _ = getrandom::getrandom(&mut nonce);
    nonce
}

#[allow(dead_code)]
const _HASH_LEN_OK: () = assert!(HASH_LENGTH == 32);

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_possible_truncation
)]
mod tests {
    use super::*;

    fn expected_threshold(difficulty: u64) -> u64 {
        if difficulty == 0 {
            return 0;
        }
        let two64: u128 = 1u128 << 64;
        (two64 - two64 / u128::from(difficulty)) as u64
    }

    #[test]
    fn pow_status_variants_are_distinct() {
        assert_ne!(PowStatus::Generating, PowStatus::Done);
    }

    #[test]
    fn threshold_matches_the_formula_at_canonical_difficulties() {
        for difficulty in [0_u64, 1, 2, 100, 1000, 10_000_000, u64::MAX] {
            assert_eq!(
                threshold(difficulty),
                expected_threshold(difficulty),
                "threshold({difficulty})"
            );
        }
    }

    #[test]
    fn account_block_data_hash_is_sha3_of_core_then_previous_hash() {
        let address =
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").expect("address parses");
        let previous =
            Hash::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .expect("previous hash parses");
        let expected = digest_to_hash(&merge(&[address.core(), previous.bytes()]));
        assert_eq!(
            account_block_data_hash(&address, &previous),
            expected,
            "data hash must be sha3-256(core || previous_hash)"
        );
    }

    #[test]
    fn verify_pow_accepts_any_nonce_at_difficulty_one() {
        let data = Hash::empty();
        assert!(
            verify_pow(&data, &[1, 2, 3, 4, 5, 6, 7, 8], 1),
            "difficulty 1 has a zero threshold, so any nonce verifies"
        );
    }

    #[cfg(feature = "native-pow")]
    #[test]
    fn a_generated_nonce_verifies_at_moderate_difficulty() {
        let data = Hash::parse("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210")
            .expect("data hash parses");
        let nonce = generate_pow(&data, 100);
        assert!(
            verify_pow(&data, &nonce, 100),
            "a generated nonce must verify at the same difficulty"
        );
    }

    #[cfg(feature = "native-pow")]
    #[test]
    fn benchmark_pow_verifies_against_the_empty_hash() {
        let nonce = benchmark_pow(100);
        assert!(
            verify_pow(&Hash::empty(), &nonce, 100),
            "benchmark_pow must verify against the empty hash"
        );
    }
}

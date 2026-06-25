//! Argon2id key derivation.
//!
//! Provides the Argon2id hash (version 0x13) and the fixed-parameter key
//! derivation the wallet uses to turn a password into a 32-byte AES key.

use crate::error::Error;
use argon2::{Algorithm, Argon2, Params, Version};

/// Argon2 version number (1.3).
pub const VERSION: u8 = 0x13;

/// Wallet KDF memory cost, in kibibytes (64 MiB).
pub const WALLET_MEMORY_KIB: u32 = 65536;

/// Wallet KDF iteration (time) cost.
pub const WALLET_ITERATIONS: u32 = 1;

/// Wallet KDF degree of parallelism (lanes).
pub const WALLET_PARALLELISM: u32 = 4;

/// Wallet KDF output length, in bytes.
pub const WALLET_TAG_LENGTH: usize = 32;

/// Argon2id parameters.
#[derive(Debug, Clone, Copy)]
pub struct Argon2idParams {
    /// Memory cost, in kibibytes.
    pub memory_kib: u32,
    /// Iteration (time) cost.
    pub iterations: u32,
    /// Degree of parallelism (lanes).
    pub parallelism: u32,
    /// Output tag length, in bytes.
    pub tag_length: usize,
}

/// The fixed wallet Argon2id parameters.
pub const WALLET_PARAMS: Argon2idParams = Argon2idParams {
    memory_kib: WALLET_MEMORY_KIB,
    iterations: WALLET_ITERATIONS,
    parallelism: WALLET_PARALLELISM,
    tag_length: WALLET_TAG_LENGTH,
};

/// Computes the Argon2id (version 0x13) tag of `password` with `salt` and `params`.
pub fn hash_id(password: &[u8], salt: &[u8], params: &Argon2idParams) -> Result<Vec<u8>, Error> {
    let cost = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(params.tag_length),
    )
    .map_err(|e| Error::InvalidInput(format!("invalid argon2 parameters: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, cost);
    let mut out = vec![0u8; params.tag_length];
    argon2
        .hash_password_into(password, salt, &mut out)
        .map_err(|e| Error::InvalidInput(format!("argon2 hashing failed: {e}")))?;
    Ok(out)
}

/// Derives a 32-byte key from `password` and `salt` using the wallet parameters.
pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32], Error> {
    let tag = zeroize::Zeroizing::new(hash_id(password, salt, &WALLET_PARAMS)?);
    <[u8; 32]>::try_from(tag.as_slice())
        .map_err(|_| Error::generic("argon2 wallet tag was not 32 bytes"))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Argon2idVector {
        password: String,
        salt: String,
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
        tag_length: usize,
        tag: String,
    }

    const VECTOR: &str = include_str!("../../tests/vectors/crypto/argon2/argon2id.json");

    fn vector() -> Argon2idVector {
        serde_json::from_str(VECTOR).expect("valid argon2id vector")
    }

    fn bytes(hex: &str) -> Vec<u8> {
        const_hex::decode(hex).expect("hex decodes")
    }

    #[test]
    fn hash_id_matches_the_known_answer_vector() {
        let v = vector();
        let params = Argon2idParams {
            memory_kib: v.memory_kib,
            iterations: v.iterations,
            parallelism: v.parallelism,
            tag_length: v.tag_length,
        };
        let tag = hash_id(&bytes(&v.password), &bytes(&v.salt), &params).expect("hash");
        assert_eq!(
            tag,
            bytes(&v.tag),
            "Argon2id tag must match the known-answer vector"
        );
    }

    #[test]
    fn wallet_parameters_match_expected_values() {
        assert_eq!(VERSION, 0x13, "Argon2 version 1.3");
        assert_eq!(WALLET_MEMORY_KIB, 65536, "64 MiB memory cost");
        assert_eq!(WALLET_ITERATIONS, 1, "one iteration");
        assert_eq!(WALLET_PARALLELISM, 4, "four lanes");
        assert_eq!(WALLET_TAG_LENGTH, 32, "32-byte key");
    }

    #[test]
    fn derive_key_uses_the_wallet_parameters() {
        let password = b"correct horse battery staple";
        let salt = [0x5bu8; 16];
        let direct = hash_id(password, &salt, &WALLET_PARAMS).expect("hash");
        let derived = derive_key(password, &salt).expect("derive");
        assert_eq!(
            derived.as_slice(),
            direct.as_slice(),
            "derive_key must equal hash_id with the wallet parameters"
        );
    }
}

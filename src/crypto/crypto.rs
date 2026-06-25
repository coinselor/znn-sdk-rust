//! SDK cryptographic facade.
//!
//! Thin wrappers over [`crate::crypto::ed25519`] for signing, verification, and
//! SLIP-0010 key derivation, plus the two digests the protocol uses: SHA3-256
//! ([`digest`]) and SHA-256 ([`sha256_bytes`]).

use crate::crypto::ed25519;
use crate::error::Error;
use sha2::{Digest, Sha256};
use sha3::Sha3_256;

/// Returns the 32-byte Ed25519 public key for a 32-byte private key.
pub fn get_public_key(private_key: &[u8; 32]) -> [u8; 32] {
    ed25519::public_key(private_key)
}

/// Returns the 64-byte RFC 8032 Ed25519 signature of `message`.
///
/// `public_key` must correspond to `private_key`; a mismatched key pair returns
/// [`Error::InvalidInput`] rather than a signature.
pub fn sign(
    message: &[u8],
    private_key: &[u8; 32],
    public_key: &[u8; 32],
) -> Result<[u8; 64], Error> {
    ed25519::sign(message, private_key, public_key)
}

/// Verifies a 64-byte Ed25519 signature, returning `true` when it is valid.
pub fn verify(signature: &[u8; 64], message: &[u8], public_key: &[u8; 32]) -> bool {
    ed25519::verify(signature, message, public_key)
}

/// Returns the 32-byte SLIP-0010 ed25519 private key for a hardened `path`,
/// propagating [`Error::InvalidInput`] for a malformed path or seed.
pub fn derive_key(path: &str, seed_hex: &str) -> Result<[u8; 32], Error> {
    Ok(*ed25519::derive_path(path, seed_hex)?.key())
}

/// Returns the 32-byte SHA3-256 (NIST FIPS 202) digest of `data`.
pub fn digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

/// Returns the 32-byte SHA-256 (NIST FIPS 180-4) digest of `data`.
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct SignatureVectors {
        vectors: Vec<SignatureVector>,
    }

    #[derive(Deserialize)]
    struct SignatureVector {
        name: String,
        secret_key: String,
        public_key: String,
        message: String,
        signature: String,
    }

    #[derive(Deserialize)]
    struct Slip10Vectors {
        seed: String,
        nodes: Vec<Slip10Node>,
    }

    #[derive(Deserialize)]
    struct Slip10Node {
        path: String,
        private_key: String,
    }

    #[derive(Deserialize)]
    struct DigestVectors {
        vectors: Vec<DigestVector>,
    }

    #[derive(Deserialize)]
    struct DigestVector {
        name: String,
        input: String,
        digest: String,
    }

    const SIGNATURES: &str = include_str!("../../tests/vectors/crypto/ed25519/signatures.json");
    const SLIP10: &str = include_str!("../../tests/vectors/crypto/ed25519/slip10.json");
    const DIGEST: &str = include_str!("../../tests/vectors/crypto/core/digest.json");
    const SHA256: &str = include_str!("../../tests/vectors/crypto/core/sha256.json");

    fn signatures() -> Vec<SignatureVector> {
        serde_json::from_str::<SignatureVectors>(SIGNATURES)
            .expect("valid signature vectors")
            .vectors
    }

    fn slip10() -> Slip10Vectors {
        serde_json::from_str::<Slip10Vectors>(SLIP10).expect("valid slip10 vectors")
    }

    fn digests(json: &str) -> Vec<DigestVector> {
        serde_json::from_str::<DigestVectors>(json)
            .expect("valid digest vectors")
            .vectors
    }

    fn arr32(hex: &str) -> [u8; 32] {
        const_hex::decode(hex)
            .expect("hex decodes")
            .try_into()
            .expect("32 bytes")
    }

    fn arr64(hex: &str) -> [u8; 64] {
        const_hex::decode(hex)
            .expect("hex decodes")
            .try_into()
            .expect("64 bytes")
    }

    // Ed25519 facade.

    #[test]
    fn get_public_key_matches_rfc8032_vectors() {
        for v in signatures() {
            assert_eq!(
                get_public_key(&arr32(&v.secret_key)),
                arr32(&v.public_key),
                "public key for {}",
                v.name
            );
        }
    }

    #[test]
    fn sign_matches_rfc8032_vectors() {
        for v in signatures() {
            let msg = const_hex::decode(&v.message).expect("message hex decodes");
            assert_eq!(
                sign(&msg, &arr32(&v.secret_key), &arr32(&v.public_key))
                    .expect("matching key pair signs"),
                arr64(&v.signature),
                "signature for {}",
                v.name
            );
        }
    }

    #[test]
    fn sign_rejects_a_mismatched_key_pair() {
        let find = |name: &str| {
            signatures()
                .into_iter()
                .find(|v| v.name == name)
                .expect("named vector present")
        };
        let a = find("rfc8032_test1_empty_message");
        let b = find("rfc8032_test2_one_byte");
        let result = sign(b"message", &arr32(&a.secret_key), &arr32(&b.public_key));
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a non-corresponding public key must be rejected, got {result:?}"
        );
    }

    #[test]
    fn verify_accepts_a_valid_signature() {
        let v = signatures()
            .into_iter()
            .find(|v| v.name == "rfc8032_test2_one_byte")
            .expect("test2 vector present");
        let msg = const_hex::decode(&v.message).expect("message hex decodes");
        assert!(
            verify(&arr64(&v.signature), &msg, &arr32(&v.public_key)),
            "a valid signature must verify"
        );
    }

    #[test]
    fn verify_rejects_a_tampered_signature() {
        let v = signatures()
            .into_iter()
            .find(|v| v.name == "rfc8032_test2_one_byte")
            .expect("test2 vector present");
        let msg = const_hex::decode(&v.message).expect("message hex decodes");
        let mut sig = arr64(&v.signature);
        sig[0] ^= 0x01;
        assert!(
            !verify(&sig, &msg, &arr32(&v.public_key)),
            "a tampered signature must not verify"
        );
    }

    #[test]
    fn derive_key_matches_slip10_private_keys() {
        let data = slip10();
        for node in &data.nodes {
            let key = derive_key(&node.path, &data.seed).expect("derive_key returns a key");
            assert_eq!(
                key,
                arr32(&node.private_key),
                "derived private key for {}",
                node.path
            );
        }
    }

    #[test]
    fn derive_key_propagates_invalid_path() {
        let result = derive_key("m/0", &slip10().seed);
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a malformed path must propagate InvalidInput, got {result:?}"
        );
    }

    #[test]
    fn derive_key_propagates_invalid_seed() {
        let result = derive_key("m/0'", "xyz");
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a non-hex seed must propagate InvalidInput, got {result:?}"
        );
    }

    // Digests.

    #[test]
    fn digest_matches_sha3_256_vectors() {
        for v in digests(DIGEST) {
            let input = const_hex::decode(&v.input).expect("input hex decodes");
            assert_eq!(
                digest(&input),
                arr32(&v.digest),
                "sha3-256 digest for {}",
                v.name
            );
        }
    }

    #[test]
    fn digest_of_empty_input_is_sha3_256_constant() {
        assert_eq!(
            digest(&[]),
            arr32("a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a")
        );
    }

    #[test]
    fn sha256_bytes_matches_sha_256_vectors() {
        for v in digests(SHA256) {
            let input = const_hex::decode(&v.input).expect("input hex decodes");
            assert_eq!(
                sha256_bytes(&input),
                arr32(&v.digest),
                "sha-256 digest for {}",
                v.name
            );
        }
    }

    #[test]
    fn sha256_bytes_of_empty_input_is_sha_256_constant() {
        assert_eq!(
            sha256_bytes(&[]),
            arr32("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
    }
}

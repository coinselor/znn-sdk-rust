//! Ed25519 signing and SLIP-0010 hierarchical key derivation.
//!
//! This module exposes the RFC 8032 Ed25519 signature scheme over 32-byte
//! private keys and the hardened-only SLIP-0010 derivation that turns a binary
//! seed into a key/chain-code pair for a BIP32-style path.

use crate::error::Error;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use sha2::Sha512;

type HmacSha512 = Hmac<Sha512>;

/// HMAC key for SLIP-0010 ed25519 master-key generation.
const ED25519_CURVE: &str = "ed25519 seed";

/// Offset added to every derivation index. SLIP-0010 ed25519 derivation is
/// hardened-only, so each path index is derived at `value + HARDENED_OFFSET`.
pub const HARDENED_OFFSET: u32 = 0x8000_0000;

/// A derived private key and its chain code, each 32 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyData {
    key: [u8; 32],
    chain_code: [u8; 32],
}

impl KeyData {
    /// Returns the 32-byte private key.
    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }

    /// Returns the 32-byte chain code.
    pub fn chain_code(&self) -> &[u8; 32] {
        &self.chain_code
    }
}

/// Derives the 32-byte Ed25519 public key for a 32-byte private key.
pub fn public_key(private_key: &[u8; 32]) -> [u8; 32] {
    SigningKey::from_bytes(private_key)
        .verifying_key()
        .to_bytes()
}

/// Produces the 64-byte RFC 8032 Ed25519 signature of `message`.
///
/// The signature is determined by `private_key` and `message`. `public_key`
/// must be the public key corresponding to `private_key`; if it is not, this
/// returns [`Error::InvalidInput`] so a mismatched key pair cannot produce a
/// signature.
pub fn sign(
    message: &[u8],
    private_key: &[u8; 32],
    public_key: &[u8; 32],
) -> Result<[u8; 64], Error> {
    let signing_key = SigningKey::from_bytes(private_key);
    if &signing_key.verifying_key().to_bytes() != public_key {
        return Err(Error::InvalidInput(
            "public key does not correspond to the private key".to_string(),
        ));
    }
    Ok(signing_key.sign(message).to_bytes())
}

/// Verifies a 64-byte Ed25519 signature, returning `true` when it is valid.
///
/// A public key that does not decode to a curve point, or a signature that
/// does not satisfy the verification equation, yields `false`.
pub fn verify(signature: &[u8; 64], message: &[u8], public_key: &[u8; 32]) -> bool {
    let signature = Signature::from_bytes(signature);
    match VerifyingKey::from_bytes(public_key) {
        Ok(verifying_key) => verifying_key.verify(message, &signature).is_ok(),
        Err(_) => false,
    }
}

/// Computes the SLIP-0010 ed25519 master key and chain code from a
/// hex-encoded seed.
pub fn get_master_key_from_seed(seed_hex: &str) -> Result<KeyData, Error> {
    let seed = const_hex::decode(seed_hex)
        .map_err(|e| Error::InvalidInput(format!("invalid seed hex: {e}")))?;
    let i = hmac_sha512(ED25519_CURVE.as_bytes(), &seed)?;
    Ok(split_key_data(&i))
}

/// Derives the SLIP-0010 ed25519 key and chain code for a hardened
/// BIP32-style `path` from a hex-encoded seed.
pub fn derive_path(path: &str, seed_hex: &str) -> Result<KeyData, Error> {
    let indices = parse_path(path)?;
    let mut data = get_master_key_from_seed(seed_hex)?;
    for index in indices {
        data = derive_child(&data, index)?;
    }
    Ok(data)
}

/// Parses a BIP32-style path into hardened derivation indices.
///
/// The path must begin with the `m` root, optionally followed by `/`-separated
/// indices, each a run of decimal digits suffixed with `'`. Each index must be
/// strictly less than [`HARDENED_OFFSET`]; the returned values already include
/// the offset.
fn parse_path(path: &str) -> Result<Vec<u32>, Error> {
    let mut segments = path.split('/');
    if segments.next() != Some("m") {
        return Err(Error::InvalidInput(format!(
            "derivation path must begin with the 'm' root: {path:?}"
        )));
    }
    let mut indices = Vec::new();
    for segment in segments {
        let digits = segment.strip_suffix('\'').ok_or_else(|| {
            Error::InvalidInput(format!(
                "derivation index must be hardened (suffixed with '): {segment:?}"
            ))
        })?;
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Error::InvalidInput(format!(
                "derivation index must be decimal digits: {segment:?}"
            )));
        }
        let value: u32 = digits.parse().map_err(|_| {
            Error::InvalidInput(format!("derivation index out of range: {segment:?}"))
        })?;
        if value >= HARDENED_OFFSET {
            return Err(Error::InvalidInput(format!(
                "derivation index must be less than 2^31: {segment:?}"
            )));
        }
        indices.push(value + HARDENED_OFFSET);
    }
    Ok(indices)
}

/// SLIP-0010 hardened child key derivation: `HMAC-SHA512(chain_code, 0x00 ||
/// key || index_be)`.
fn derive_child(parent: &KeyData, index: u32) -> Result<KeyData, Error> {
    let mut data = Vec::with_capacity(37);
    data.push(0x00);
    data.extend_from_slice(&parent.key);
    data.extend_from_slice(&index.to_be_bytes());
    let i = hmac_sha512(&parent.chain_code, &data)?;
    Ok(split_key_data(&i))
}

/// Splits a 64-byte HMAC output into the 32-byte key (left) and 32-byte chain
/// code (right).
fn split_key_data(i: &[u8; 64]) -> KeyData {
    let (left, right) = i.split_at(32);
    let mut key = [0u8; 32];
    let mut chain_code = [0u8; 32];
    key.copy_from_slice(left);
    chain_code.copy_from_slice(right);
    KeyData { key, chain_code }
}

/// Computes `HMAC-SHA512(key, data)`. HMAC accepts a key of any length, so the
/// construction never fails for the keys used here.
fn hmac_sha512(key: &[u8], data: &[u8]) -> Result<[u8; 64], Error> {
    let mut mac = HmacSha512::new_from_slice(key)
        .map_err(|e| Error::generic(format!("HMAC-SHA512 key rejected: {e}")))?;
    mac.update(data);
    let bytes = mac.finalize().into_bytes();
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    Ok(out)
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
        chain_code: String,
        private_key: String,
        public_key: String,
    }

    const SIGNATURES: &str = include_str!("../../tests/vectors/crypto/ed25519/signatures.json");
    const SLIP10: &str = include_str!("../../tests/vectors/crypto/ed25519/slip10.json");

    fn signatures() -> Vec<SignatureVector> {
        serde_json::from_str::<SignatureVectors>(SIGNATURES)
            .expect("valid signature vectors")
            .vectors
    }

    fn slip10() -> Slip10Vectors {
        serde_json::from_str::<Slip10Vectors>(SLIP10).expect("valid slip10 vectors")
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

    fn vector(name: &str) -> SignatureVector {
        signatures()
            .into_iter()
            .find(|v| v.name == name)
            .expect("named signature vector present")
    }

    // Signing.

    #[test]
    fn public_key_matches_rfc8032_vectors() {
        for v in signatures() {
            let sk = arr32(&v.secret_key);
            assert_eq!(
                public_key(&sk),
                arr32(&v.public_key),
                "public key for {}",
                v.name
            );
        }
    }

    #[test]
    fn sign_matches_rfc8032_vectors() {
        for v in signatures() {
            let sk = arr32(&v.secret_key);
            let pk = arr32(&v.public_key);
            let msg = const_hex::decode(&v.message).expect("message hex decodes");
            assert_eq!(
                sign(&msg, &sk, &pk).expect("matching key pair signs"),
                arr64(&v.signature),
                "signature for {}",
                v.name
            );
        }
    }

    #[test]
    fn sign_rejects_a_mismatched_key_pair() {
        let a = vector("rfc8032_test1_empty_message");
        let b = vector("rfc8032_test2_one_byte");
        let result = sign(b"message", &arr32(&a.secret_key), &arr32(&b.public_key));
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a public key that does not match the private key must be rejected, got {result:?}"
        );
    }

    #[test]
    fn verify_accepts_valid_signatures() {
        for v in signatures() {
            let pk = arr32(&v.public_key);
            let msg = const_hex::decode(&v.message).expect("message hex decodes");
            let sig = arr64(&v.signature);
            assert!(
                verify(&sig, &msg, &pk),
                "valid signature for {} must verify",
                v.name
            );
        }
    }

    #[test]
    fn verify_rejects_a_tampered_signature() {
        let v = vector("rfc8032_test2_one_byte");
        let pk = arr32(&v.public_key);
        let msg = const_hex::decode(&v.message).expect("message hex decodes");
        let mut sig = arr64(&v.signature);
        sig[0] ^= 0x01;
        assert!(
            !verify(&sig, &msg, &pk),
            "a signature with a flipped byte must not verify"
        );
    }

    #[test]
    fn verify_rejects_a_modified_message() {
        let v = vector("rfc8032_test3_two_bytes");
        let pk = arr32(&v.public_key);
        let sig = arr64(&v.signature);
        assert!(
            !verify(&sig, b"not the signed message", &pk),
            "a signature over a different message must not verify"
        );
    }

    #[test]
    fn sign_then_verify_round_trips() {
        let v = vector("rfc8032_test2_one_byte");
        let sk = arr32(&v.secret_key);
        let pk = arr32(&v.public_key);
        let msg = const_hex::decode(&v.message).expect("message hex decodes");
        let sig = sign(&msg, &sk, &pk).expect("matching key pair signs");
        assert!(
            verify(&sig, &msg, &pk),
            "a freshly produced signature must verify"
        );
    }

    // SLIP-0010 derivation.

    #[test]
    fn master_key_matches_slip10_vector() {
        let data = slip10();
        let master = data
            .nodes
            .iter()
            .find(|n| n.path == "m")
            .expect("master node present");
        let key = get_master_key_from_seed(&data.seed).expect("master derives");
        assert_eq!(key.key(), &arr32(&master.private_key), "master private key");
        assert_eq!(
            key.chain_code(),
            &arr32(&master.chain_code),
            "master chain code"
        );
    }

    #[test]
    fn derive_path_matches_slip10_vectors() {
        let data = slip10();
        for node in &data.nodes {
            let key = derive_path(&node.path, &data.seed).expect("derive_path returns a key");
            assert_eq!(
                key.key(),
                &arr32(&node.private_key),
                "private key for {}",
                node.path
            );
            assert_eq!(
                key.chain_code(),
                &arr32(&node.chain_code),
                "chain code for {}",
                node.path
            );
        }
    }

    #[test]
    fn derived_master_key_yields_slip10_public_key() {
        let data = slip10();
        let master = data
            .nodes
            .iter()
            .find(|n| n.path == "m")
            .expect("master node present");
        let key = derive_path("m", &data.seed).expect("derive master");
        assert_eq!(
            public_key(key.key()),
            arr32(&master.public_key),
            "public key from the derived master private key"
        );
    }

    #[test]
    fn key_data_equality_and_inequality() {
        let data = slip10();
        let a = derive_path("m/0'", &data.seed).expect("derive m/0'");
        let b = derive_path("m/0'", &data.seed).expect("derive m/0'");
        let c = derive_path("m/0'/1'", &data.seed).expect("derive m/0'/1'");
        assert_eq!(a, b, "identical derivations are equal");
        assert_ne!(a, c, "different derivations are not equal");
    }

    // Path and seed validation.

    #[test]
    fn get_master_key_rejects_non_hex_seed() {
        let result = get_master_key_from_seed("xyz");
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "non-hex seed must be rejected with InvalidInput, got {result:?}"
        );
    }

    #[test]
    fn derive_path_rejects_an_empty_path() {
        let result = derive_path("", &slip10().seed);
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "empty path must be rejected with InvalidInput, got {result:?}"
        );
    }

    #[test]
    fn derive_path_rejects_a_non_hardened_index() {
        let result = derive_path("m/0", &slip10().seed);
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "non-hardened index must be rejected with InvalidInput, got {result:?}"
        );
    }

    #[test]
    fn derive_path_rejects_a_non_numeric_index() {
        let result = derive_path("m/abc'", &slip10().seed);
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "non-numeric index must be rejected with InvalidInput, got {result:?}"
        );
    }

    #[test]
    fn derive_path_rejects_a_missing_master_root() {
        let result = derive_path("0'/1'", &slip10().seed);
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a path without the m root must be rejected with InvalidInput, got {result:?}"
        );
    }

    // Constant.

    #[test]
    fn hardened_offset_is_two_pow_31() {
        assert_eq!(HARDENED_OFFSET, 0x8000_0000);
    }
}

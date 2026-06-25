//! Account key pair: private key, derived public key, address, and signing.
//!
//! A [`KeyPair`] wraps a 32-byte Ed25519 private key and derives the public key
//! and Zenon address on demand. The address is `z` plus a 20-byte core formed
//! from the user byte and the first 19 bytes of the SHA3-256 digest of the
//! public key.

use crate::crypto::crypto;
use crate::error::Error;
use crate::primitives::address::{Address, CORE_SIZE, PREFIX, USER_BYTE};

/// An Ed25519 account key pair.
#[derive(Clone, zeroize::ZeroizeOnDrop)]
pub struct KeyPair {
    private_key: [u8; 32],
}

impl core::fmt::Debug for KeyPair {
    /// Redacts the private key; it is not printed.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KeyPair").finish_non_exhaustive()
    }
}

impl KeyPair {
    /// Creates a key pair from a 32-byte private key.
    pub fn from_private_key(private_key: [u8; 32]) -> Self {
        Self { private_key }
    }

    /// Returns the 32-byte private key.
    pub fn private_key(&self) -> &[u8; 32] {
        &self.private_key
    }

    /// Returns the 32-byte Ed25519 public key for this key pair.
    pub fn public_key(&self) -> [u8; 32] {
        crypto::get_public_key(&self.private_key)
    }

    /// Returns the Zenon address for this key pair.
    ///
    /// The 20-byte core is the user byte followed by the first 19 bytes of the
    /// SHA3-256 digest of the public key.
    pub fn address(&self) -> Result<Address, Error> {
        let digest = crypto::digest(&self.public_key());
        let mut core = [0u8; CORE_SIZE];
        core[0] = USER_BYTE;
        core[1..].copy_from_slice(&digest[..CORE_SIZE - 1]);
        Address::new(PREFIX, &core)
    }

    /// Signs `message`, returning the 64-byte Ed25519 signature.
    pub fn sign(&self, message: &[u8]) -> Result<[u8; 64], Error> {
        crypto::sign(message, &self.private_key, &self.public_key())
    }

    /// Verifies a 64-byte signature of `message` under this key pair's public key.
    pub fn verify(&self, signature: &[u8; 64], message: &[u8]) -> bool {
        crypto::verify(signature, message, &self.public_key())
    }
}

impl crate::wallet::interfaces::WalletAccount for KeyPair {
    fn get_public_key(&self) -> Result<[u8; 32], Error> {
        Ok(self.public_key())
    }

    fn get_address(&self) -> Result<Address, Error> {
        self.address()
    }

    fn sign(&self, message: &[u8]) -> Result<[u8; 64], Error> {
        self.sign(message)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct KeyPairVectors {
        vectors: Vec<KeyPairVector>,
    }

    #[derive(Deserialize)]
    struct KeyPairVector {
        name: String,
        secret_key: String,
        public_key: String,
        address: String,
        message: String,
        signature: String,
    }

    const VECTORS: &str = include_str!("../../tests/vectors/wallet/keypair/keypair.json");

    fn vectors() -> Vec<KeyPairVector> {
        serde_json::from_str::<KeyPairVectors>(VECTORS)
            .expect("valid keypair vectors")
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

    #[test]
    fn public_key_matches_vectors() {
        for v in vectors() {
            let kp = KeyPair::from_private_key(arr32(&v.secret_key));
            assert_eq!(
                kp.public_key(),
                arr32(&v.public_key),
                "public key for {}",
                v.name
            );
        }
    }

    #[test]
    fn address_matches_vectors() {
        for v in vectors() {
            let kp = KeyPair::from_private_key(arr32(&v.secret_key));
            assert_eq!(
                kp.address().expect("address derives").to_string(),
                v.address,
                "address for {}",
                v.name
            );
        }
    }

    #[test]
    fn sign_matches_vectors() {
        for v in vectors() {
            let kp = KeyPair::from_private_key(arr32(&v.secret_key));
            let message = const_hex::decode(&v.message).expect("message hex decodes");
            assert_eq!(
                kp.sign(&message).expect("signs"),
                arr64(&v.signature),
                "signature for {}",
                v.name
            );
        }
    }

    #[test]
    fn verify_accepts_valid_signatures() {
        for v in vectors() {
            let kp = KeyPair::from_private_key(arr32(&v.secret_key));
            let message = const_hex::decode(&v.message).expect("message hex decodes");
            assert!(
                kp.verify(&arr64(&v.signature), &message),
                "valid signature for {} must verify",
                v.name
            );
        }
    }

    #[test]
    fn verify_rejects_a_tampered_signature() {
        let v = vectors()
            .into_iter()
            .find(|v| v.name == "rfc8032_test2_one_byte")
            .expect("test2 vector present");
        let kp = KeyPair::from_private_key(arr32(&v.secret_key));
        let message = const_hex::decode(&v.message).expect("message hex decodes");
        let mut sig = arr64(&v.signature);
        sig[0] ^= 0x01;
        assert!(
            !kp.verify(&sig, &message),
            "a tampered signature must not verify"
        );
    }

    #[test]
    fn sign_then_verify_round_trips() {
        let v = vectors()
            .into_iter()
            .find(|v| v.name == "rfc8032_test2_one_byte")
            .expect("test2 vector present");
        let kp = KeyPair::from_private_key(arr32(&v.secret_key));
        let message = b"a fresh message";
        let signature = kp.sign(message).expect("signs");
        assert!(
            kp.verify(&signature, message),
            "a freshly produced signature must verify"
        );
    }
}

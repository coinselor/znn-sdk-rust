//! Cryptographic primitives: Ed25519, SLIP-0010 derivation, hashing, Argon2id.

pub mod argon2;
// The facade module is intentionally named `crypto`, so its path is
// `crate::crypto::crypto`; the repeated segment is deliberate.
#[allow(clippy::module_inception)]
pub mod crypto;
pub mod ed25519;

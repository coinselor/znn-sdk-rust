//! Keystore: BIP-39 mnemonic/entropy/seed wallet and its on-disk definition.
//!
//! A [`KeyStore`] holds a normalized mnemonic, its hex entropy, and its hex
//! seed, and derives account [`KeyPair`]s along the Zenon BIP-44 path. A
//! [`KeyStoreDefinition`] names a keystore file on disk.

use crate::crypto::crypto;
use crate::error::Error;
use crate::primitives::address::Address;
use crate::wallet::derivation::get_derivation_account;
use crate::wallet::interfaces::WalletDefinition;
use crate::wallet::keypair::KeyPair;
use bip39::{Language, Mnemonic};
use std::path::{Path, PathBuf};

/// A BIP-39 wallet holding a mnemonic, its entropy, and its seed.
#[derive(Clone, zeroize::ZeroizeOnDrop)]
pub struct KeyStore {
    mnemonic: Option<String>,
    entropy: String,
    seed: Option<String>,
}

impl core::fmt::Debug for KeyStore {
    /// Redacts the secret material; the mnemonic, entropy, and seed are not printed.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KeyStore").finish_non_exhaustive()
    }
}

impl KeyStore {
    /// Builds a keystore from a BIP-39 mnemonic, deriving entropy and seed.
    ///
    /// Returns [`Error::InvalidInput`] if the mnemonic is not a valid,
    /// normalized English BIP-39 sentence.
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self, Error> {
        let parsed = Mnemonic::parse_in_normalized(Language::English, mnemonic)
            .map_err(|e| Error::InvalidInput(format!("invalid mnemonic: {e}")))?;
        Ok(Self {
            mnemonic: Some(parsed.to_string()),
            entropy: const_hex::encode(parsed.to_entropy()),
            seed: Some(const_hex::encode(parsed.to_seed_normalized(""))),
        })
    }

    /// Builds a keystore from hex `entropy`, deriving the mnemonic and seed.
    ///
    /// The stored entropy is lowercased. Returns [`Error::InvalidInput`] for
    /// non-hex input or an invalid entropy length.
    pub fn from_entropy(entropy_hex: &str) -> Result<Self, Error> {
        let bytes = const_hex::decode(entropy_hex)
            .map_err(|e| Error::InvalidInput(format!("invalid entropy hex: {e}")))?;
        let parsed = Mnemonic::from_entropy_in(Language::English, &bytes)
            .map_err(|e| Error::InvalidInput(format!("invalid entropy: {e}")))?;
        Ok(Self {
            mnemonic: Some(parsed.to_string()),
            entropy: entropy_hex.to_lowercase(),
            seed: Some(const_hex::encode(parsed.to_seed_normalized(""))),
        })
    }

    /// Builds a keystore from a hex seed.
    pub fn from_seed(seed: &str) -> Result<Self, Error> {
        let bytes = const_hex::decode(seed)
            .map_err(|e| Error::InvalidInput(format!("invalid seed hex: {e}")))?;
        if bytes.is_empty() {
            return Err(Error::InvalidInput("seed must not be empty".to_string()));
        }
        Ok(Self {
            mnemonic: None,
            entropy: String::new(),
            seed: Some(seed.to_string()),
        })
    }

    /// Builds a keystore from fresh random entropy.
    pub fn new_random() -> Result<Self, Error> {
        let mnemonic = Mnemonic::generate_in(Language::English, 24)
            .map_err(|e| Error::InvalidInput(format!("entropy generation failed: {e}")))?;
        Self::from_entropy(&const_hex::encode(mnemonic.to_entropy()))
    }

    /// Returns the normalized mnemonic, if set.
    pub fn mnemonic(&self) -> Option<&str> {
        self.mnemonic.as_deref()
    }

    /// Returns the hex entropy.
    pub fn entropy(&self) -> &str {
        &self.entropy
    }

    /// Returns the hex seed, if set.
    pub fn seed(&self) -> Option<&str> {
        self.seed.as_deref()
    }

    /// Returns the account key pair at `index` along the Zenon BIP-44 path.
    ///
    /// Returns [`Error::InvalidInput`] if the keystore has no seed.
    pub fn get_key_pair(&self, index: u32) -> Result<KeyPair, Error> {
        let seed = self
            .seed
            .as_deref()
            .ok_or_else(|| Error::InvalidInput("keystore has no seed".to_string()))?;
        let key = crypto::derive_key(&get_derivation_account(index), seed)?;
        Ok(KeyPair::from_private_key(key))
    }

    /// Returns the addresses for indices in `[left, right)`.
    pub fn derive_addresses_by_range(&self, left: u32, right: u32) -> Result<Vec<Address>, Error> {
        (left..right)
            .map(|i| self.get_key_pair(i)?.address())
            .collect()
    }

    /// Returns the index of `address` among the first `count` accounts, if found.
    pub fn find_address(&self, address: &Address, count: u32) -> Result<Option<usize>, Error> {
        for i in 0..count {
            if &self.get_key_pair(i)?.address()? == address {
                return Ok(Some(i as usize));
            }
        }
        Ok(None)
    }
}

impl crate::wallet::interfaces::Wallet for KeyStore {
    fn get_account(
        &self,
        index: u32,
    ) -> Result<Box<dyn crate::wallet::interfaces::WalletAccount>, Error> {
        Ok(Box::new(self.get_key_pair(index)?))
    }
}

/// A keystore file on disk.
#[derive(Debug, Clone)]
pub struct KeyStoreDefinition {
    file: PathBuf,
}

impl KeyStoreDefinition {
    /// Creates a definition for an existing keystore `file`.
    ///
    /// Returns [`Error::InvalidInput`] if the file does not exist.
    pub fn new(file: PathBuf) -> Result<Self, Error> {
        if !file.exists() {
            return Err(Error::InvalidInput(format!(
                "keystore does not exist: {}",
                file.display()
            )));
        }
        Ok(Self { file })
    }

    /// Returns the keystore file path.
    pub fn file(&self) -> &Path {
        &self.file
    }
}

impl WalletDefinition for KeyStoreDefinition {
    fn wallet_id(&self) -> String {
        self.file.to_string_lossy().into_owned()
    }

    fn wallet_name(&self) -> String {
        self.file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

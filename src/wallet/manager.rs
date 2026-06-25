//! Keystore manager: lists, finds, saves, and reads keystore files in a wallet
//! directory.

use crate::error::Error;
use crate::wallet::constants::{BASE_ADDRESS_KEY, KEY_STORE_WALLET_TYPE, WALLET_TYPE_KEY};
use crate::wallet::encrypted_file::EncryptedFile;
use crate::wallet::exceptions::WalletError;
use crate::wallet::interfaces::{Wallet, WalletDefinition, WalletManager, WalletOptions};
use crate::wallet::keystore::{KeyStore, KeyStoreDefinition};
use serde_json::{Map, Value};
use std::path::{Component, Path, PathBuf};

/// Options for opening a keystore wallet.
#[derive(Debug, Clone)]
pub struct KeyStoreOptions {
    /// Password used to decrypt the keystore.
    pub decryption_password: String,
}

impl KeyStoreOptions {
    /// Creates options with the given decryption password.
    pub fn new(decryption_password: impl Into<String>) -> Self {
        Self {
            decryption_password: decryption_password.into(),
        }
    }
}

impl WalletOptions for KeyStoreOptions {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

/// Manages keystore files within a wallet directory.
#[derive(Debug, Clone)]
pub struct KeyStoreManager {
    wallet_path: PathBuf,
}

impl KeyStoreManager {
    /// Creates a manager rooted at `wallet_path`.
    pub fn new(wallet_path: PathBuf) -> Self {
        Self { wallet_path }
    }

    /// Returns the wallet directory.
    pub fn wallet_path(&self) -> &Path {
        &self.wallet_path
    }

    fn keystore_path_for_name(&self, name: &str) -> Result<PathBuf, Error> {
        let mut components = Path::new(name).components();
        match (components.next(), components.next()) {
            (Some(Component::Normal(_)), None) if !name.is_empty() => {
                Ok(self.wallet_path.join(name))
            }
            _ => Err(Error::InvalidInput(format!(
                "keystore name must be a single file name: {name:?}"
            ))),
        }
    }

    /// Returns a definition for every keystore file in the wallet directory.
    pub fn list_all_key_stores(&self) -> Result<Vec<KeyStoreDefinition>, Error> {
        let entries = std::fs::read_dir(&self.wallet_path)
            .map_err(|e| Error::generic(format!("cannot read wallet directory: {e}")))?;
        let mut stores = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| Error::generic(format!("cannot read entry: {e}")))?;
            let path = entry.path();
            if path.is_file() {
                stores.push(KeyStoreDefinition::new(path)?);
            }
        }
        Ok(stores)
    }

    /// Returns the definition of the keystore named `name`, if present.
    pub fn find_key_store(&self, name: &str) -> Result<Option<KeyStoreDefinition>, Error> {
        for store in self.list_all_key_stores()? {
            if store.wallet_name() == name {
                return Ok(Some(store));
            }
        }
        Ok(None)
    }

    /// Encrypts and writes `store` to disk, returning its definition.
    pub fn save_key_store(
        &self,
        store: &KeyStore,
        password: &str,
        name: Option<&str>,
    ) -> Result<KeyStoreDefinition, Error> {
        if store.entropy().is_empty() {
            return Err(Error::InvalidInput(
                "keystore has no entropy to persist; build it from a mnemonic or entropy"
                    .to_string(),
            ));
        }
        let base_address = store.get_key_pair(0)?.address()?.to_string();
        let name = match name {
            Some(n) => n.to_string(),
            None => base_address.clone(),
        };

        let mut metadata = Map::new();
        metadata.insert(BASE_ADDRESS_KEY.to_string(), Value::String(base_address));
        metadata.insert(
            WALLET_TYPE_KEY.to_string(),
            Value::String(KEY_STORE_WALLET_TYPE.to_string()),
        );

        let entropy = zeroize::Zeroizing::new(
            const_hex::decode(store.entropy())
                .map_err(|e| Error::InvalidInput(format!("invalid keystore entropy: {e}")))?,
        );
        let encrypted = EncryptedFile::encrypt(&entropy, password, Some(metadata))?;
        let json = encrypted.to_json()?;

        let path = self.keystore_path_for_name(&name)?;
        std::fs::write(&path, json)
            .map_err(|e| Error::generic(format!("cannot write keystore: {e}")))?;
        KeyStoreDefinition::new(path)
    }

    /// Reads and decrypts the keystore at `file`.
    ///
    /// Returns [`WalletError::IncorrectPassword`] for a wrong password and
    /// [`WalletError::UnsupportedWalletType`] for a non-keystore wallet type, so
    /// callers can distinguish those from other failures.
    pub fn read_key_store(&self, password: &str, file: &Path) -> Result<KeyStore, WalletError> {
        if !file.exists() {
            return Err(WalletError::wallet(format!(
                "keystore does not exist: {}",
                file.display()
            )));
        }
        let content = std::fs::read_to_string(file)
            .map_err(|e| WalletError::wallet(format!("cannot read keystore: {e}")))?;
        let encrypted =
            EncryptedFile::from_json(&content).map_err(|e| WalletError::wallet(e.to_string()))?;

        if let Some(metadata) = encrypted.metadata()
            && let Some(wallet_type) = metadata.get(WALLET_TYPE_KEY).and_then(Value::as_str)
            && wallet_type != KEY_STORE_WALLET_TYPE
        {
            return Err(WalletError::UnsupportedWalletType(wallet_type.to_string()));
        }

        let entropy = zeroize::Zeroizing::new(encrypted.decrypt(password)?);
        let entropy_hex = zeroize::Zeroizing::new(const_hex::encode(entropy.as_slice()));
        KeyStore::from_entropy(&entropy_hex).map_err(|e| WalletError::wallet(e.to_string()))
    }

    /// Creates a new random keystore and saves it.
    pub fn create_new(
        &self,
        passphrase: &str,
        name: Option<&str>,
    ) -> Result<KeyStoreDefinition, Error> {
        let store = KeyStore::new_random()?;
        self.save_key_store(&store, passphrase, name)
    }

    /// Creates a keystore from a mnemonic and saves it.
    pub fn create_from_mnemonic(
        &self,
        mnemonic: &str,
        passphrase: &str,
        name: Option<&str>,
    ) -> Result<KeyStoreDefinition, Error> {
        let store = KeyStore::from_mnemonic(mnemonic)?;
        self.save_key_store(&store, passphrase, name)
    }
}

impl WalletManager for KeyStoreManager {
    fn get_wallet_definitions(&self) -> Result<Vec<Box<dyn WalletDefinition>>, Error> {
        Ok(self
            .list_all_key_stores()?
            .into_iter()
            .map(|d| Box::new(d) as Box<dyn WalletDefinition>)
            .collect())
    }

    fn get_wallet(
        &self,
        definition: &dyn WalletDefinition,
        options: Option<&dyn WalletOptions>,
    ) -> Result<Box<dyn Wallet>, Error> {
        if !self.supports_wallet(definition)? {
            return Err(Error::generic(
                "wallet definition is not managed by this keystore manager",
            ));
        }
        let options = options
            .ok_or_else(|| Error::generic("keystore options are required to open a wallet"))?;
        let options = options
            .as_any()
            .downcast_ref::<KeyStoreOptions>()
            .ok_or_else(|| Error::generic("unsupported wallet options"))?;
        let path = PathBuf::from(definition.wallet_id());
        let store = self.read_key_store(&options.decryption_password, &path)?;
        Ok(Box::new(store) as Box<dyn Wallet>)
    }

    fn supports_wallet(&self, definition: &dyn WalletDefinition) -> Result<bool, Error> {
        for known in self.get_wallet_definitions()? {
            if known.wallet_id() == definition.wallet_id() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_wallet_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("znn_keystore_mgr_{tag}"));
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).expect("temp wallet dir");
        for name in ["wallet_a", "wallet_b"] {
            fs::write(dir.join(name), b"{}").expect("write keystore file");
        }
        dir
    }

    fn empty_temp_wallet_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("znn_keystore_mgr_{tag}"));
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).expect("temp wallet dir");
        dir
    }

    #[test]
    fn list_all_key_stores_returns_one_per_file() {
        let dir = temp_wallet_dir("list");
        let manager = KeyStoreManager::new(dir.clone());
        let stores = manager.list_all_key_stores().expect("lists");
        assert_eq!(stores.len(), 2, "two keystore files are present");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_key_store_locates_an_existing_file() {
        let dir = temp_wallet_dir("find");
        let manager = KeyStoreManager::new(dir.clone());
        let found = manager.find_key_store("wallet_a").expect("search runs");
        let def = found.expect("wallet_a is present");
        assert_eq!(
            def.wallet_name(),
            "wallet_a",
            "found definition names the file"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_key_store_returns_none_for_a_missing_file() {
        let dir = temp_wallet_dir("find_missing");
        let manager = KeyStoreManager::new(dir.clone());
        let found = manager.find_key_store("not_there").expect("search runs");
        assert!(found.is_none(), "a missing file yields None");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn get_wallet_definitions_lists_all_keystores() {
        let dir = temp_wallet_dir("defs");
        let manager = KeyStoreManager::new(dir.clone());
        let defs = manager.get_wallet_definitions().expect("lists");
        assert_eq!(defs.len(), 2, "trait listing returns one per file");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn supports_wallet_accepts_a_known_keystore() {
        let dir = temp_wallet_dir("supports");
        let manager = KeyStoreManager::new(dir.clone());
        let def = KeyStoreDefinition::new(dir.join("wallet_a")).expect("existing file");
        assert!(
            manager.supports_wallet(&def).expect("check runs"),
            "a keystore in the wallet directory must be supported"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_key_store_writes_a_named_keystore() {
        let dir = temp_wallet_dir("save");
        let manager = KeyStoreManager::new(dir.clone());
        let store = KeyStore::from_entropy("000102030405060708090a0b0c0d0e0f").expect("entropy");
        let def = manager
            .save_key_store(&store, "password", Some("saved_wallet"))
            .expect("save succeeds");
        assert!(
            def.file().exists(),
            "the saved keystore file must exist on disk"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_key_store_rejects_path_traversal_names() {
        let dir = temp_wallet_dir("save_traversal");
        let outside = std::env::temp_dir().join("znn_keystore_mgr_save_traversal_target");
        fs::remove_file(&outside).ok();
        let manager = KeyStoreManager::new(dir.clone());
        let store = KeyStore::from_entropy("000102030405060708090a0b0c0d0e0f").expect("entropy");

        let relative = manager.save_key_store(&store, "password", Some("../znn_keystore_escape"));
        assert!(
            matches!(relative, Err(Error::InvalidInput(_))),
            "relative traversal names must be rejected, got {relative:?}"
        );

        let absolute_name = outside.to_string_lossy();
        let absolute = manager.save_key_store(&store, "password", Some(&absolute_name));
        assert!(
            matches!(absolute, Err(Error::InvalidInput(_))),
            "absolute path names must be rejected, got {absolute:?}"
        );
        assert!(
            !outside.exists(),
            "save_key_store must not write outside the wallet directory"
        );

        fs::remove_dir_all(&dir).ok();
        fs::remove_file(&outside).ok();
    }

    #[test]
    fn save_then_read_round_trips_the_keystore() {
        let dir = temp_wallet_dir("roundtrip");
        let manager = KeyStoreManager::new(dir.clone());
        let store = KeyStore::from_entropy("000102030405060708090a0b0c0d0e0f").expect("entropy");
        let def = manager
            .save_key_store(&store, "password", Some("rt_wallet"))
            .expect("save succeeds");
        let reopened = manager
            .read_key_store("password", def.file())
            .expect("read succeeds");
        assert_eq!(
            reopened.entropy(),
            store.entropy(),
            "a saved keystore must read back with the same entropy"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_new_writes_a_random_keystore_that_reads_back() {
        let dir = empty_temp_wallet_dir("create_new");
        let manager = KeyStoreManager::new(dir.clone());
        let def = manager
            .create_new("password", Some("random_wallet"))
            .expect("create_new succeeds");

        assert!(
            def.file().exists(),
            "create_new must write the keystore file"
        );
        assert_eq!(def.wallet_name(), "random_wallet");

        let reopened = manager
            .read_key_store("password", def.file())
            .expect("created keystore reads back");
        assert!(
            reopened
                .mnemonic()
                .is_some_and(|mnemonic| !mnemonic.is_empty()),
            "random keystore must read back with a mnemonic"
        );
        assert!(
            !reopened.entropy().is_empty(),
            "random keystore must read back with persisted entropy"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_from_mnemonic_writes_a_keystore_that_reads_back() {
        const MNEMONIC: &str = "route become dream access impulse price inform obtain engage ski \
believe awful absent pig thing vibrant possible exotic flee pepper marble rural fire fancy";

        let dir = empty_temp_wallet_dir("create_from_mnemonic");
        let manager = KeyStoreManager::new(dir.clone());
        let expected = KeyStore::from_mnemonic(MNEMONIC).expect("valid mnemonic");
        let def = manager
            .create_from_mnemonic(MNEMONIC, "password", Some("mnemonic_wallet"))
            .expect("create_from_mnemonic succeeds");

        assert!(
            def.file().exists(),
            "create_from_mnemonic must write the keystore file"
        );
        assert_eq!(def.wallet_name(), "mnemonic_wallet");

        let reopened = manager
            .read_key_store("password", def.file())
            .expect("created keystore reads back");
        assert_eq!(
            reopened.mnemonic(),
            expected.mnemonic(),
            "mnemonic-created keystore must preserve the normalized mnemonic"
        );
        assert_eq!(
            reopened.entropy(),
            expected.entropy(),
            "mnemonic-created keystore must preserve entropy"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_rejects_a_seed_only_keystore() {
        let dir = temp_wallet_dir("seed_only");
        let manager = KeyStoreManager::new(dir.clone());
        let store =
            KeyStore::from_seed("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff")
                .expect("valid seed");
        let result = manager.save_key_store(&store, "password", Some("bad"));
        assert!(
            matches!(result, Err(Error::InvalidInput(_))),
            "a keystore with no entropy must not be persisted, got {result:?}"
        );
        assert!(
            !dir.join("bad").exists(),
            "no unreadable keystore file must be written"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_reports_a_wrong_password_distinctly() {
        let dir = temp_wallet_dir("wrongpw");
        let manager = KeyStoreManager::new(dir.clone());
        let store = KeyStore::from_entropy("000102030405060708090a0b0c0d0e0f").expect("entropy");
        let def = manager
            .save_key_store(&store, "correct-password", Some("pw_wallet"))
            .expect("save succeeds");
        let result = manager.read_key_store("wrong-password", def.file());
        assert!(
            matches!(result, Err(WalletError::IncorrectPassword)),
            "a wrong password must surface as IncorrectPassword, got {result:?}"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_rejects_an_unsupported_wallet_type() {
        let dir = temp_wallet_dir("unsupported_type");
        let path = dir.join("ledger_wallet");
        let manager = KeyStoreManager::new(dir.clone());
        let mut metadata = Map::new();
        metadata.insert(
            WALLET_TYPE_KEY.to_string(),
            Value::String("ledger".to_string()),
        );
        let encrypted = EncryptedFile::encrypt(
            b"000102030405060708090a0b0c0d0e0f",
            "password",
            Some(metadata),
        )
        .expect("encrypts");
        fs::write(&path, encrypted.to_json().expect("serializes")).expect("write file");

        let result = manager.read_key_store("password", &path);
        assert!(
            matches!(result, Err(WalletError::UnsupportedWalletType(ref kind)) if kind == "ledger"),
            "unsupported wallet types must be reported distinctly, got {result:?}"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn get_wallet_rejects_a_definition_outside_the_wallet_directory() {
        let dir = temp_wallet_dir("unmanaged");
        let outside = std::env::temp_dir().join("znn_keystore_outside");
        fs::create_dir_all(&outside).expect("outside dir");
        let outside_file = outside.join("intruder");
        fs::write(&outside_file, b"{}").expect("write outside file");

        let manager = KeyStoreManager::new(dir.clone());
        let foreign = KeyStoreDefinition::new(outside_file).expect("existing file");
        let options = KeyStoreOptions::new("password");
        let result = manager.get_wallet(&foreign, Some(&options));
        assert!(
            result.is_err(),
            "the manager must not open a definition outside its wallet directory"
        );

        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&outside).ok();
    }

    #[test]
    fn get_wallet_preserves_the_wrong_password_error() {
        let dir = temp_wallet_dir("getwallet_pw");
        let manager = KeyStoreManager::new(dir.clone());
        let store = KeyStore::from_entropy("000102030405060708090a0b0c0d0e0f").expect("entropy");
        let def = manager
            .save_key_store(&store, "correct-password", Some("gw_wallet"))
            .expect("save succeeds");
        let options = KeyStoreOptions::new("wrong-password");
        let is_incorrect_password = matches!(
            manager.get_wallet(&def, Some(&options)),
            Err(Error::IncorrectPassword)
        );
        assert!(
            is_incorrect_password,
            "get_wallet must surface a wrong password as Error::IncorrectPassword"
        );
        fs::remove_dir_all(&dir).ok();
    }
}

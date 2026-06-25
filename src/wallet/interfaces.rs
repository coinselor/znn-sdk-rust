//! Wallet abstraction traits.
//!
//! These traits define the contracts shared by wallet implementations: a wallet
//! definition (identity), wallet options, a wallet and its accounts, and a
//! wallet manager. Concrete implementations live in their own modules.

use crate::error::Error;
use crate::primitives::address::Address;

/// Identity of a wallet (for example, a keystore file).
pub trait WalletDefinition {
    /// Returns the wallet's id or path.
    fn wallet_id(&self) -> String;

    /// Returns the wallet's display name.
    fn wallet_name(&self) -> String;

    /// Returns `true` if this wallet's name equals `name`.
    fn is_named(&self, name: &str) -> bool {
        self.wallet_name() == name
    }
}

/// Options for opening a wallet (for example, a decryption password).
pub trait WalletOptions {
    /// Returns the options as [`Any`](core::any::Any) for downcasting to a
    /// concrete options type.
    fn as_any(&self) -> &dyn core::any::Any;
}

/// A wallet that yields accounts by index.
pub trait Wallet {
    /// Returns the wallet account at `index`.
    fn get_account(&self, index: u32) -> Result<Box<dyn WalletAccount>, Error>;
}

/// A single account of a wallet.
pub trait WalletAccount {
    /// Returns the account's 32-byte Ed25519 public key.
    fn get_public_key(&self) -> Result<[u8; 32], Error>;

    /// Returns the account's address.
    fn get_address(&self) -> Result<Address, Error>;

    /// Signs `message`, returning the 64-byte signature.
    fn sign(&self, message: &[u8]) -> Result<[u8; 64], Error>;

    /// Returns the account's address as a canonical string.
    fn address_string(&self) -> Result<String, Error> {
        Ok(self.get_address()?.to_string())
    }
}

/// Manages wallets of a particular kind.
pub trait WalletManager {
    /// Returns the definitions of all wallets this manager knows about.
    fn get_wallet_definitions(&self) -> Result<Vec<Box<dyn WalletDefinition>>, Error>;

    /// Opens the wallet identified by `definition`, using `options` if needed.
    fn get_wallet(
        &self,
        definition: &dyn WalletDefinition,
        options: Option<&dyn WalletOptions>,
    ) -> Result<Box<dyn Wallet>, Error>;

    /// Returns `true` if this manager supports `definition`.
    fn supports_wallet(&self, definition: &dyn WalletDefinition) -> Result<bool, Error>;
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    struct TestDefinition {
        id: String,
        name: String,
    }

    impl WalletDefinition for TestDefinition {
        fn wallet_id(&self) -> String {
            self.id.clone()
        }
        fn wallet_name(&self) -> String {
            self.name.clone()
        }
    }

    struct TestAccount {
        address: Address,
    }

    impl WalletAccount for TestAccount {
        fn get_public_key(&self) -> Result<[u8; 32], Error> {
            Ok([0u8; 32])
        }
        fn get_address(&self) -> Result<Address, Error> {
            Ok(self.address.clone())
        }
        fn sign(&self, _message: &[u8]) -> Result<[u8; 64], Error> {
            Ok([0u8; 64])
        }
    }

    fn empty_address() -> Address {
        Address::parse("z1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsggv2f").expect("empty address parses")
    }

    #[test]
    fn is_named_matches_the_wallet_name() {
        let def = TestDefinition {
            id: "/wallets/main".to_string(),
            name: "main".to_string(),
        };
        assert!(def.is_named("main"), "is_named must match the wallet name");
        assert!(
            !def.is_named("other"),
            "is_named must reject a different name"
        );
    }

    #[test]
    fn is_named_dispatches_through_a_trait_object() {
        let def: Box<dyn WalletDefinition> = Box::new(TestDefinition {
            id: "/wallets/main".to_string(),
            name: "main".to_string(),
        });
        assert!(def.is_named("main"));
    }

    #[test]
    fn address_string_renders_the_account_address() {
        let address = empty_address();
        let account = TestAccount {
            address: address.clone(),
        };
        assert_eq!(
            account.address_string().expect("address string"),
            address.to_string(),
            "address_string must render the account address"
        );
    }

    #[test]
    fn address_string_dispatches_through_a_trait_object() {
        let address = empty_address();
        let account: Box<dyn WalletAccount> = Box::new(TestAccount {
            address: address.clone(),
        });
        assert_eq!(
            account.address_string().expect("address string"),
            address.to_string()
        );
    }
}

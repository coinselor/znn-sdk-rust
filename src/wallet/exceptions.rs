//! Wallet error types.

use thiserror::Error;

/// Errors raised by wallet operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WalletError {
    /// A general wallet failure carrying a message.
    #[error("{0}")]
    Wallet(String),

    /// The supplied password failed to decrypt the keystore.
    #[error("Incorrect password")]
    IncorrectPassword,

    /// The wallet type recorded in a keystore is not supported.
    #[error("Wallet type ({0}) is not supported")]
    UnsupportedWalletType(String),
}

impl WalletError {
    /// Creates a general [`WalletError::Wallet`] from a message.
    pub fn wallet(message: impl Into<String>) -> Self {
        Self::Wallet(message.into())
    }
}

impl From<WalletError> for crate::error::Error {
    /// Maps a wallet error into the crate error, preserving the wrong-password
    /// signal as [`crate::error::Error::IncorrectPassword`].
    fn from(error: WalletError) -> Self {
        match error {
            WalletError::IncorrectPassword => crate::error::Error::IncorrectPassword,
            WalletError::UnsupportedWalletType(kind) => {
                crate::error::Error::InvalidInput(format!("wallet type ({kind}) is not supported"))
            }
            WalletError::Wallet(message) => crate::error::Error::Generic(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_displays_its_message() {
        assert_eq!(
            WalletError::wallet("Given keyStore does not exist").to_string(),
            "Given keyStore does not exist"
        );
    }

    #[test]
    fn incorrect_password_display() {
        assert_eq!(
            WalletError::IncorrectPassword.to_string(),
            "Incorrect password"
        );
    }

    #[test]
    fn unsupported_wallet_type_display() {
        assert_eq!(
            WalletError::UnsupportedWalletType("ledger".to_string()).to_string(),
            "Wallet type (ledger) is not supported"
        );
    }

    #[test]
    fn equal_variants_compare_equal() {
        assert_eq!(WalletError::wallet("x"), WalletError::wallet("x"));
    }

    #[test]
    fn distinct_variants_compare_unequal() {
        assert_ne!(WalletError::wallet("x"), WalletError::IncorrectPassword);
    }
}

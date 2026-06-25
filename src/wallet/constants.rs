//! Wallet metadata and keystore constants.

/// Metadata key for the wallet's base (index 0) address.
pub const BASE_ADDRESS_KEY: &str = "baseAddress";

/// Metadata key for the wallet type.
pub const WALLET_TYPE_KEY: &str = "walletType";

/// Wallet type value identifying a keystore wallet.
pub const KEY_STORE_WALLET_TYPE: &str = "keystore";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_address_key_value() {
        assert_eq!(BASE_ADDRESS_KEY, "baseAddress");
    }

    #[test]
    fn wallet_type_key_value() {
        assert_eq!(WALLET_TYPE_KEY, "walletType");
    }

    #[test]
    fn key_store_wallet_type_value() {
        assert_eq!(KEY_STORE_WALLET_TYPE, "keystore");
    }
}

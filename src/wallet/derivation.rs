//! BIP-44 derivation paths for Zenon accounts.
//!
//! Zenon uses the BIP-44 layout `m / purpose' / coin_type' / account'`.
//! Purpose is `44`; coin type is `73404`. Every level is hardened.

/// Zenon BIP-44 coin type.
pub const COIN_TYPE: &str = "73404";

/// Zenon BIP-44 path prefix: `m/44'/<COIN_TYPE>'`.
pub const DERIVATION_PATH: &str = "m/44'/73404'";

/// Returns the hardened derivation path for the given account index.
///
/// The result is `m/44'/73404'/<account>'`.
pub fn get_derivation_account(account: u32) -> String {
    format!("{DERIVATION_PATH}/{account}'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_type_value() {
        assert_eq!(COIN_TYPE, "73404");
    }

    #[test]
    fn derivation_path_prefix() {
        assert_eq!(DERIVATION_PATH, "m/44'/73404'");
    }

    #[test]
    fn account_zero_path() {
        assert_eq!(get_derivation_account(0), "m/44'/73404'/0'");
    }

    #[test]
    fn account_five_path() {
        assert_eq!(get_derivation_account(5), "m/44'/73404'/5'");
    }

    #[test]
    fn account_path_builds_on_prefix() {
        assert_eq!(
            get_derivation_account(7),
            format!("{DERIVATION_PATH}/7'"),
            "account path must extend the derivation prefix"
        );
    }
}

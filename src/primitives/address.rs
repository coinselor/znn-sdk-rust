//! Zenon `Address` type.
//!
//! An `Address` is a human-readable part (`z`) and a 20-byte core, encoded as a
//! canonical lowercase Bech32 string. This module builds on the Bech32 codec in
//! [`crate::primitives::bech32`] and adds parsing, validation, ordering, and the
//! embedded contract address set.

use crate::error::Error;
use crate::primitives::bech32::{decode_bech32_address, encode_bech32_address};
use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;
use std::sync::LazyLock;

/// Human-readable part of a Zenon address.
pub const PREFIX: &str = "z";
/// Length of a canonical Zenon address string.
pub const ADDRESS_LENGTH: usize = 40;
/// Leading core byte for user addresses.
pub const USER_BYTE: u8 = 0;
/// Leading core byte for embedded contract addresses.
pub const CONTRACT_BYTE: u8 = 1;
/// Byte length of an address core.
pub const CORE_SIZE: usize = 20;

/// Canonical strings of the embedded contract addresses.
const EMBEDDED_CONTRACT_STRINGS: [&str; 11] = [
    "z1qxemdeddedxplasmaxxxxxxxxxxxxxxxxsctrp",
    "z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg",
    "z1qxemdeddedxt0kenxxxxxxxxxxxxxxxxh9amk0",
    "z1qxemdeddedxsentynelxxxxxxxxxxxxxwy0r2r",
    "z1qxemdeddedxswapxxxxxxxxxxxxxxxxxxl4yww",
    "z1qxemdeddedxstakexxxxxxxxxxxxxxxxjv8v62",
    "z1qxemdeddedxaccelerat0rxxxxxxxxxxp4tk22",
    "z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d",
    "z1qxemdeddedxlyquydytyxxxxxxxxxxxxflaaae",
    "z1qxemdeddedxsp0rkxxxxxxxxxxxxxxxx956u48",
    "z1qxemdeddedxhtlcxxxxxxxxxxxxxxxxxygecvw",
];

static EMBEDDED_CONTRACT_ADDRESSES: LazyLock<Vec<Address>> = LazyLock::new(|| {
    // These strings are canonical, verified addresses; any that failed to parse
    // would be dropped here and caught by the `is_embedded` unit test.
    EMBEDDED_CONTRACT_STRINGS
        .iter()
        .filter_map(|s| Address::parse(s).ok())
        .collect()
});

/// A Zenon address: a human-readable part and a 20-byte core.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    hrp: String,
    core: [u8; CORE_SIZE],
}

impl Address {
    /// Creates an address from a human-readable part and a core, rejecting a
    /// core whose length is not [`CORE_SIZE`].
    pub fn new(hrp: &str, core: &[u8]) -> Result<Self, Error> {
        let core: [u8; CORE_SIZE] = core.try_into().map_err(|_| {
            Error::InvalidInput(format!(
                "address core must be {CORE_SIZE} bytes, got {}",
                core.len()
            ))
        })?;
        Ok(Self {
            hrp: hrp.to_string(),
            core,
        })
    }

    /// Parses a canonical Zenon Bech32 address.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let (hrp, core) = decode_bech32_address(s)?;
        Self::new(&hrp, &core)
    }

    /// Returns the human-readable part.
    pub fn hrp(&self) -> &str {
        &self.hrp
    }

    /// Returns the 20-byte core.
    pub fn core(&self) -> &[u8; CORE_SIZE] {
        &self.core
    }

    /// Returns the short form: first 7 characters, `...`, then the last 6.
    pub fn to_short_string(&self) -> String {
        let s = self.to_string();
        let head = s.get(..7).unwrap_or(&s);
        let tail = s.get(s.len().saturating_sub(6)..).unwrap_or("");
        format!("{head}...{tail}")
    }

    /// Returns `true` if `s` parses and re-encodes to exactly `s`.
    pub fn is_valid(s: &str) -> bool {
        Self::parse(s).is_ok_and(|address| address.to_string() == s)
    }

    /// Returns `true` if this address is in the embedded contract address set.
    pub fn is_embedded(&self) -> bool {
        EMBEDDED_CONTRACT_ADDRESSES.contains(self)
    }
}

/// Returns the embedded contract address set.
pub fn embedded_contract_addresses() -> Vec<Address> {
    EMBEDDED_CONTRACT_ADDRESSES.clone()
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = encode_bech32_address(&self.hrp, &self.core).map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct AddressVectors {
        addresses: Vec<AddressVector>,
    }

    #[derive(Deserialize)]
    struct AddressVector {
        address: String,
        core_hex: String,
        embedded: bool,
    }

    #[derive(Deserialize)]
    struct InvalidVectors {
        invalid: Vec<InvalidVector>,
    }

    #[derive(Deserialize)]
    struct InvalidVector {
        address: String,
    }

    const EMBEDDED: &str = include_str!("../../tests/vectors/primitives/address/embedded.json");
    const INVALID: &str = include_str!("../../tests/vectors/primitives/address/invalid.json");
    const EMPTY: &str = "z1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsggv2f";

    fn addresses() -> Vec<AddressVector> {
        serde_json::from_str::<AddressVectors>(EMBEDDED)
            .expect("valid address vectors")
            .addresses
    }

    fn invalid() -> Vec<InvalidVector> {
        serde_json::from_str::<InvalidVectors>(INVALID)
            .expect("valid invalid vectors")
            .invalid
    }

    #[test]
    fn constants_match_expected_values() {
        assert_eq!(PREFIX, "z");
        assert_eq!(ADDRESS_LENGTH, 40);
        assert_eq!(USER_BYTE, 0);
        assert_eq!(CONTRACT_BYTE, 1);
        assert_eq!(CORE_SIZE, 20);
    }

    #[test]
    fn new_accepts_20_byte_core() {
        let addr = Address::new("z", &[0u8; 20]).expect("20-byte core accepted");
        assert_eq!(addr.core(), &[0u8; 20]);
        assert_eq!(addr.hrp(), "z");
    }

    #[test]
    fn new_rejects_non_20_byte_core() {
        assert!(Address::new("z", &[0u8; 19]).is_err());
    }

    #[test]
    fn parse_empty_address() {
        let addr = Address::parse(EMPTY).expect("empty address parses");
        assert_eq!(addr.hrp(), "z");
        assert_eq!(addr.core(), &[0u8; 20]);
    }

    #[test]
    fn round_trip_known_vectors() {
        for v in addresses() {
            let core = const_hex::decode(&v.core_hex).expect("valid hex");
            let addr = Address::parse(&v.address).expect("vector parses");
            assert_eq!(
                addr.core().as_slice(),
                core.as_slice(),
                "core for {}",
                v.address
            );
            assert_eq!(
                addr.to_string(),
                v.address,
                "canonical string for {}",
                v.address
            );
        }
    }

    #[test]
    fn parse_rejects_invalid_string() {
        assert!(Address::parse("z1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsggv2g").is_err());
    }

    #[test]
    fn to_short_string_empty() {
        let addr = Address::parse(EMPTY).expect("empty address parses");
        assert_eq!(addr.to_short_string(), "z1qqqqq...sggv2f");
    }

    #[test]
    fn is_valid_true_for_canonical_vectors() {
        for v in addresses() {
            assert!(
                Address::is_valid(&v.address),
                "should be valid: {}",
                v.address
            );
        }
    }

    #[test]
    fn is_valid_false_for_invalid_vectors() {
        for v in invalid() {
            assert!(
                !Address::is_valid(&v.address),
                "should be invalid: {}",
                v.address
            );
        }
    }

    #[test]
    fn equal_addresses_compare_equal() {
        let a = Address::parse(EMPTY).expect("parses");
        let b = Address::parse(EMPTY).expect("parses");
        assert_eq!(a, b);
    }

    #[test]
    fn ordering_follows_canonical_string() {
        let a_str = EMPTY;
        let b_str = "z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg";
        let a = Address::parse(a_str).expect("parses");
        let b = Address::parse(b_str).expect("parses");
        assert_eq!(a.cmp(&b), a_str.cmp(b_str));
    }

    #[test]
    fn is_embedded_matches_flag() {
        for v in addresses() {
            let addr = Address::parse(&v.address).expect("parses");
            assert_eq!(
                addr.is_embedded(),
                v.embedded,
                "is_embedded for {}",
                v.address
            );
        }
    }

    #[test]
    fn embedded_set_is_complete() {
        assert_eq!(embedded_contract_addresses().len(), 11);
    }
}

//! Zenon `AccountHeader` ledger model.
//!
//! An [`AccountHeader`] pairs an [`Address`], a [`struct@Hash`], and a height. It is
//! the element type of [`crate::model::nom::momentum::Momentum`] content.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use serde_json::{Value, json};

/// A Zenon account header: an address, a hash, and a height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountHeader {
    address: Address,
    hash: Hash,
    height: u64,
}

impl AccountHeader {
    /// Creates an account header from an address, a hash, and a height.
    pub fn new(address: Address, hash: Hash, height: u64) -> Self {
        Self {
            address,
            hash,
            height,
        }
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the hash.
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    /// Returns the height.
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Serializes the header to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "address": self.address.to_string(),
            "hash": self.hash.to_string(),
            "height": self.height,
        })
    }

    /// Deserializes a header from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "account header")?;
        let address = Address::parse(required_str(object, "address")?)?;
        let hash = Hash::parse(required_str(object, "hash")?)?;
        let height = required_u64(object, "height")?;
        Ok(Self::new(address, hash, height))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct AccountHeaderConformance {
        #[allow(dead_code)]
        description: String,
        header: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/nom/account_header.json");

    fn header_value() -> Value {
        serde_json::from_str::<AccountHeaderConformance>(CONFORMANCE)
            .expect("valid account header conformance")
            .header
    }

    fn sample_header() -> AccountHeader {
        AccountHeader::new(
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            Hash::parse("3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73")
                .unwrap(),
            2,
        )
    }

    #[test]
    fn new_and_accessors_return_each_field() {
        let header = sample_header();
        assert_eq!(
            header.address().to_string(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(
            header.hash().to_string(),
            "3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73"
        );
        assert_eq!(header.height(), 2);
    }

    #[test]
    fn to_json_matches_the_conformance_header() {
        let header = AccountHeader::from_json(&header_value()).expect("conformance header parses");
        assert_eq!(header.to_json(), header_value());
    }

    #[test]
    fn from_json_reads_the_conformance_fields() {
        let header = AccountHeader::from_json(&header_value()).expect("conformance header parses");
        assert_eq!(
            header.address().to_string(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(
            header.hash().to_string(),
            "3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73"
        );
        assert_eq!(header.height(), 2);
    }

    #[test]
    fn round_trips_through_to_json_and_from_json() {
        // Anchor on a constructor-built header so a placeholder cannot satisfy
        // the round-trip.
        let original = sample_header();
        let round_trip = AccountHeader::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn rejects_a_malformed_header_object() {
        let mut missing = header_value();
        missing
            .as_object_mut()
            .expect("header is an object")
            .remove("hash");
        let result = AccountHeader::from_json(&missing);
        assert!(result.is_err(), "missing hash must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut bad_hash = header_value();
        bad_hash["hash"] = serde_json::json!("not-a-hash");
        let result = AccountHeader::from_json(&bad_hash);
        assert!(result.is_err(), "non-canonical hash must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn equal_when_all_fields_match() {
        assert_eq!(sample_header(), sample_header());
    }

    #[test]
    fn not_equal_when_heights_differ() {
        let other = AccountHeader {
            height: 3,
            ..sample_header()
        };
        assert_ne!(sample_header(), other);
    }

    #[test]
    fn not_equal_when_hashes_differ() {
        let other = AccountHeader {
            hash: Hash::parse("598fa623dd308bec7163bb375aa7546ec4aced3b71a1c9278709903e69280dbd")
                .unwrap(),
            ..sample_header()
        };
        assert_ne!(sample_header(), other);
    }
}

//! Stake contract models.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use num_bigint::BigUint;
use serde_json::{Value, json};

/// A staking entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeEntry {
    amount: BigUint,
    weighted_amount: BigUint,
    start_timestamp: u64,
    expiration_timestamp: u64,
    address: Address,
    id: Hash,
}

impl StakeEntry {
    /// Creates a stake entry.
    pub fn new(
        amount: BigUint,
        weighted_amount: BigUint,
        start_timestamp: u64,
        expiration_timestamp: u64,
        address: Address,
        id: Hash,
    ) -> Self {
        Self {
            amount,
            weighted_amount,
            start_timestamp,
            expiration_timestamp,
            address,
            id,
        }
    }

    /// Returns the amount.
    pub fn amount(&self) -> &BigUint {
        &self.amount
    }
    /// Returns the weighted amount.
    pub fn weighted_amount(&self) -> &BigUint {
        &self.weighted_amount
    }
    /// Returns the start timestamp.
    pub fn start_timestamp(&self) -> u64 {
        self.start_timestamp
    }
    /// Returns the expiration timestamp.
    pub fn expiration_timestamp(&self) -> u64 {
        self.expiration_timestamp
    }
    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }
    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "amount": self.amount.to_string(),
            "weightedAmount": self.weighted_amount.to_string(),
            "startTimestamp": self.start_timestamp,
            "expirationTimestamp": self.expiration_timestamp,
            "address": self.address.to_string(),
            "id": self.id.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "stake entry")?;
        Ok(Self::new(
            required_big_uint(object, "amount")?,
            required_big_uint(object, "weightedAmount")?,
            required_u64(object, "startTimestamp")?,
            required_u64(object, "expirationTimestamp")?,
            Address::parse(required_str(object, "address")?)?,
            Hash::parse(required_str(object, "id")?)?,
        ))
    }
}

/// A paged list of stake entries with totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeList {
    total_amount: BigUint,
    total_weighted_amount: BigUint,
    count: u64,
    list: Vec<StakeEntry>,
}

impl StakeList {
    /// Creates a stake list.
    pub fn new(
        total_amount: BigUint,
        total_weighted_amount: BigUint,
        count: u64,
        list: Vec<StakeEntry>,
    ) -> Self {
        Self {
            total_amount,
            total_weighted_amount,
            count,
            list,
        }
    }

    /// Returns the total amount.
    pub fn total_amount(&self) -> &BigUint {
        &self.total_amount
    }
    /// Returns the total weighted amount.
    pub fn total_weighted_amount(&self) -> &BigUint {
        &self.total_weighted_amount
    }
    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }
    /// Returns the list.
    pub fn list(&self) -> &[StakeEntry] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "totalAmount": self.total_amount.to_string(),
            "totalWeightedAmount": self.total_weighted_amount.to_string(),
            "count": self.count,
            "list": self.list.iter().map(StakeEntry::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "stake list")?;
        Ok(Self::new(
            required_big_uint(object, "totalAmount")?,
            required_big_uint(object, "totalWeightedAmount")?,
            required_u64(object, "count")?,
            required_array(object, "list", StakeEntry::from_json)?,
        ))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Conformance {
        #[allow(dead_code)]
        description: String,
        stake_entry: Value,
        stake_list: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/stake.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid stake conformance")
    }

    fn hash_value() -> &'static str {
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }

    #[test]
    fn stake_entry_round_trip() {
        let original = StakeEntry::new(
            BigUint::from(100_000_000_000u64),
            BigUint::from(150_000_000_000u64),
            1_700_000_000,
            1_730_000_000,
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            Hash::parse(hash_value()).unwrap(),
        );
        let round_trip = StakeEntry::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn stake_entry_from_json_reads_conformance() {
        let entry = StakeEntry::from_json(&conf().stake_entry).expect("parses");
        assert_eq!(*entry.amount(), BigUint::from(100_000_000_000u64));
        assert_eq!(entry.start_timestamp(), 1_700_000_000);
        assert_eq!(entry.expiration_timestamp(), 1_730_000_000);
        assert_eq!(entry.id().to_string(), hash_value());
    }

    #[test]
    fn stake_entry_rejects_malformed() {
        let mut bad = conf().stake_entry;
        bad["amount"] = json!("not-a-number");
        let result = StakeEntry::from_json(&bad);
        assert!(result.is_err(), "non-decimal amount must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn stake_list_round_trip() {
        let value = conf().stake_list;
        let list = StakeList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
        assert_eq!(list.count(), 1);
        assert_eq!(list.list().len(), 1);
        assert_eq!(*list.total_amount(), BigUint::from(100_000_000_000u64));
    }
}

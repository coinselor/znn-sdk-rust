//! HTLC contract model.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::TokenStandard;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use num_bigint::BigUint;
use serde_json::{Value, json};

/// An HTLC contract entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtlcInfo {
    id: Hash,
    time_locked: Address,
    hash_locked: Address,
    token_standard: TokenStandard,
    amount: BigUint,
    expiration_time: u64,
    hash_type: u64,
    key_max_size: u64,
    hash_lock: Vec<u8>,
}

impl HtlcInfo {
    /// Creates an HTLC info.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Hash,
        time_locked: Address,
        hash_locked: Address,
        token_standard: TokenStandard,
        amount: BigUint,
        expiration_time: u64,
        hash_type: u64,
        key_max_size: u64,
        hash_lock: Vec<u8>,
    ) -> Self {
        Self {
            id,
            time_locked,
            hash_locked,
            token_standard,
            amount,
            expiration_time,
            hash_type,
            key_max_size,
            hash_lock,
        }
    }

    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }
    /// Returns the time-locked address.
    pub fn time_locked(&self) -> &Address {
        &self.time_locked
    }
    /// Returns the hash-locked address.
    pub fn hash_locked(&self) -> &Address {
        &self.hash_locked
    }
    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }
    /// Returns the amount.
    pub fn amount(&self) -> &BigUint {
        &self.amount
    }
    /// Returns the expiration time.
    pub fn expiration_time(&self) -> u64 {
        self.expiration_time
    }
    /// Returns the hash type.
    pub fn hash_type(&self) -> u64 {
        self.hash_type
    }
    /// Returns the key max size.
    pub fn key_max_size(&self) -> u64 {
        self.key_max_size
    }
    /// Returns the hash lock bytes.
    pub fn hash_lock(&self) -> &[u8] {
        &self.hash_lock
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id.to_string(),
            "timeLocked": self.time_locked.to_string(),
            "hashLocked": self.hash_locked.to_string(),
            "tokenStandard": self.token_standard.to_string(),
            "amount": self.amount.to_string(),
            "expirationTime": self.expiration_time,
            "hashType": self.hash_type,
            "keyMaxSize": self.key_max_size,
            "hashLock": STANDARD.encode(&self.hash_lock),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "htlc info")?;
        Ok(Self::new(
            Hash::parse(required_str(object, "id")?)?,
            Address::parse(required_str(object, "timeLocked")?)?,
            Address::parse(required_str(object, "hashLocked")?)?,
            TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            required_big_uint(object, "amount")?,
            required_u64(object, "expirationTime")?,
            required_u64(object, "hashType")?,
            required_u64(object, "keyMaxSize")?,
            optional_base64(object, "hashLock")?,
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
        htlc_info: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/htlc.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid htlc conformance")
    }

    fn sample() -> HtlcInfo {
        HtlcInfo::new(
            Hash::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap(),
            TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap(),
            BigUint::from(100_000_000u64),
            1_700_000_000,
            0,
            32,
            vec![0xde, 0xad, 0xbe, 0xef],
        )
    }

    #[test]
    fn htlc_round_trip() {
        let original = sample();
        let round_trip = HtlcInfo::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn htlc_from_json_reads_conformance() {
        let info = HtlcInfo::from_json(&conf().htlc_info).expect("conformance parses");
        assert_eq!(
            info.time_locked().to_string(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(*info.amount(), BigUint::from(100_000_000u64));
        assert_eq!(info.expiration_time(), 1_700_000_000);
        assert_eq!(info.hash_lock(), [0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hash_lock_missing_or_null_defaults_to_empty_but_populated_parses() {
        let mut missing = conf().htlc_info;
        missing.as_object_mut().unwrap().remove("hashLock");
        assert!(
            HtlcInfo::from_json(&missing)
                .expect("missing hashLock parses")
                .hash_lock()
                .is_empty()
        );

        let mut nulled = conf().htlc_info;
        nulled["hashLock"] = Value::Null;
        assert!(
            HtlcInfo::from_json(&nulled)
                .expect("null hashLock parses")
                .hash_lock()
                .is_empty()
        );

        // A populated hashLock must decode.
        assert_eq!(
            HtlcInfo::from_json(&conf().htlc_info)
                .expect("conformance parses")
                .hash_lock(),
            [0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn htlc_rejects_malformed_hash_lock() {
        let mut bad_base64 = conf().htlc_info;
        bad_base64["hashLock"] = json!("@@@@");
        let result = HtlcInfo::from_json(&bad_base64);
        assert!(result.is_err(), "invalid base64 hashLock must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut wrong_kind = conf().htlc_info;
        wrong_kind["hashLock"] = json!(42);
        let result = HtlcInfo::from_json(&wrong_kind);
        assert!(result.is_err(), "non-string hashLock must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn htlc_rejects_malformed() {
        let mut bad = conf().htlc_info;
        bad["amount"] = json!("not-a-number");
        let result = HtlcInfo::from_json(&bad);
        assert!(result.is_err(), "non-decimal amount must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }
}

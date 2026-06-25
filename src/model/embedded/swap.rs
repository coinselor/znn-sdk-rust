//! Legacy Swap contract models.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::hash::Hash;
use num_bigint::BigUint;
use serde_json::{Value, json};

/// A swap asset entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapAssetEntry {
    key_id_hash: Hash,
    qsr: BigUint,
    znn: BigUint,
}

impl SwapAssetEntry {
    /// Creates a swap asset entry.
    pub fn new(key_id_hash: Hash, qsr: BigUint, znn: BigUint) -> Self {
        Self {
            key_id_hash,
            qsr,
            znn,
        }
    }

    /// Returns the key id hash.
    pub fn key_id_hash(&self) -> &Hash {
        &self.key_id_hash
    }

    /// Returns the qsr amount.
    pub fn qsr(&self) -> &BigUint {
        &self.qsr
    }

    /// Returns the znn amount.
    pub fn znn(&self) -> &BigUint {
        &self.znn
    }

    /// Returns whether either amount is non-zero.
    pub fn has_balance(&self) -> bool {
        self.qsr > BigUint::from(0u32) || self.znn > BigUint::from(0u32)
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "keyIdHash": self.key_id_hash.to_string(),
            "qsr": self.qsr.to_string(),
            "znn": self.znn.to_string(),
        })
    }

    /// Deserializes from a JSON object, taking the out-of-band key id hash.
    pub fn from_json(key_id_hash: Hash, value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "swap asset entry")?;
        Ok(Self::new(
            key_id_hash,
            required_big_uint(object, "qsr")?,
            required_big_uint(object, "znn")?,
        ))
    }
}

/// A legacy pillar swap entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapLegacyPillarEntry {
    num_pillars: u64,
    key_id_hash: Hash,
}

impl SwapLegacyPillarEntry {
    /// Creates a legacy pillar entry.
    pub fn new(num_pillars: u64, key_id_hash: Hash) -> Self {
        Self {
            num_pillars,
            key_id_hash,
        }
    }

    /// Returns the number of pillars.
    pub fn num_pillars(&self) -> u64 {
        self.num_pillars
    }

    /// Returns the key id hash.
    pub fn key_id_hash(&self) -> &Hash {
        &self.key_id_hash
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "numPillars": self.num_pillars,
            "keyIdHash": self.key_id_hash.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "swap legacy pillar entry")?;
        Ok(Self::new(
            required_u64(object, "numPillars")?,
            Hash::parse(required_str(object, "keyIdHash")?)?,
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
        swap_asset_entry: Value,
        swap_legacy_pillar_entry: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/swap.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid swap conformance")
    }

    fn hash_value() -> &'static str {
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }

    #[test]
    fn swap_asset_entry_round_trip() {
        let original = SwapAssetEntry::new(
            Hash::parse(hash_value()).unwrap(),
            BigUint::from(5_000_000_000u64),
            BigUint::from(10_000_000_000u64),
        );
        let key = Hash::parse(hash_value()).unwrap();
        let round_trip =
            SwapAssetEntry::from_json(key, &original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn swap_asset_entry_from_json_reads_body() {
        let value = conf().swap_asset_entry;
        let entry =
            SwapAssetEntry::from_json(Hash::parse(hash_value()).unwrap(), &value).expect("parses");
        assert_eq!(*entry.qsr(), BigUint::from(5_000_000_000u64));
        assert_eq!(*entry.znn(), BigUint::from(10_000_000_000u64));
        assert_eq!(entry.key_id_hash().to_string(), hash_value());
    }

    #[test]
    fn swap_asset_entry_has_balance_reflects_amounts() {
        let zero = SwapAssetEntry::new(
            Hash::parse(hash_value()).unwrap(),
            BigUint::from(0u32),
            BigUint::from(0u32),
        );
        assert!(!zero.has_balance(), "zero amounts report no balance");

        let nonzero_qsr = SwapAssetEntry::new(
            Hash::parse(hash_value()).unwrap(),
            BigUint::from(1u32),
            BigUint::from(0u32),
        );
        assert!(nonzero_qsr.has_balance(), "non-zero qsr reports a balance");

        let nonzero_znn = SwapAssetEntry::new(
            Hash::parse(hash_value()).unwrap(),
            BigUint::from(0u32),
            BigUint::from(1u32),
        );
        assert!(nonzero_znn.has_balance(), "non-zero znn reports a balance");
    }

    #[test]
    fn swap_asset_entry_rejects_malformed_body() {
        let mut bad = conf().swap_asset_entry;
        bad["qsr"] = json!("not-a-number");
        let result = SwapAssetEntry::from_json(Hash::parse(hash_value()).unwrap(), &bad);
        assert!(result.is_err(), "non-decimal qsr must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn swap_legacy_pillar_entry_round_trip() {
        let original = SwapLegacyPillarEntry::new(3, Hash::parse(hash_value()).unwrap());
        let round_trip =
            SwapLegacyPillarEntry::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn swap_legacy_pillar_entry_from_json_reads_conformance() {
        let entry =
            SwapLegacyPillarEntry::from_json(&conf().swap_legacy_pillar_entry).expect("parses");
        assert_eq!(entry.num_pillars(), 3);
        assert_eq!(entry.key_id_hash().to_string(), hash_value());
    }
}

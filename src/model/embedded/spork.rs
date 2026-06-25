//! Spork (network feature-flag) contract models.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::hash::Hash;
use serde_json::{Value, json};

/// A spork record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spork {
    id: Hash,
    name: String,
    description: String,
    activated: bool,
    enforcement_height: u64,
}

impl Spork {
    /// Creates a spork.
    pub fn new(
        id: Hash,
        name: String,
        description: String,
        activated: bool,
        enforcement_height: u64,
    ) -> Self {
        Self {
            id,
            name,
            description,
            activated,
            enforcement_height,
        }
    }

    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }
    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }
    /// Returns whether activated.
    pub fn activated(&self) -> bool {
        self.activated
    }
    /// Returns the enforcement height.
    pub fn enforcement_height(&self) -> u64 {
        self.enforcement_height
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id.to_string(),
            "name": self.name,
            "description": self.description,
            "activated": self.activated,
            "enforcementHeight": self.enforcement_height,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "spork")?;
        Ok(Self::new(
            Hash::parse(required_str(object, "id")?)?,
            required_str(object, "name")?.to_string(),
            required_str(object, "description")?.to_string(),
            required_bool(object, "activated")?,
            required_u64(object, "enforcementHeight")?,
        ))
    }
}

/// A paged list of sporks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SporkList {
    count: u64,
    list: Vec<Spork>,
}

impl SporkList {
    /// Creates a spork list.
    pub fn new(count: u64, list: Vec<Spork>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Returns the list.
    pub fn list(&self) -> &[Spork] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(Spork::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "spork list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", Spork::from_json)?,
        ))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Conformance {
        #[allow(dead_code)]
        description: String,
        spork: Value,
        spork_list: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/spork.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid spork conformance")
    }

    fn hash_value() -> &'static str {
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }

    #[test]
    fn spork_round_trip() {
        let original = Spork::new(
            Hash::parse(hash_value()).unwrap(),
            "acceleratorTestnetSpork".to_string(),
            "Activate the Accelerator on testnet".to_string(),
            true,
            100,
        );
        let round_trip = Spork::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn spork_from_json_reads_conformance() {
        let value = conf().spork;
        let spork = Spork::from_json(&value).expect("conformance parses");
        assert_eq!(spork.to_json(), value);
        assert_eq!(spork.name(), "acceleratorTestnetSpork");
        assert!(spork.activated());
        assert_eq!(spork.enforcement_height(), 100);
        assert_eq!(spork.id().to_string(), hash_value());
    }

    #[test]
    fn spork_rejects_malformed() {
        let mut bad = conf().spork;
        bad.as_object_mut().unwrap().remove("id");
        let result = Spork::from_json(&bad);
        assert!(result.is_err(), "missing id must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn spork_list_round_trip() {
        let value = conf().spork_list;
        let list = SporkList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
        assert_eq!(list.count(), 1);
        assert_eq!(list.list().len(), 1);
    }
}

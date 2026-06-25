//! Zenon `DetailedMomentum` and `DetailedMomentumList` ledger models.
//!
//! A [`DetailedMomentum`] pairs a [`Momentum`] with the [`AccountBlock`]s it
//! contains.

use crate::error::Error;
use crate::model::json::*;
use crate::model::nom::account_block::AccountBlock;
use crate::model::nom::momentum::Momentum;
use serde_json::{Map, Value, json};

/// A momentum paired with the account blocks it contains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailedMomentum {
    blocks: Vec<AccountBlock>,
    momentum: Momentum,
}

impl DetailedMomentum {
    /// Creates a detailed momentum from its blocks and momentum.
    pub fn new(blocks: Vec<AccountBlock>, momentum: Momentum) -> Self {
        Self { blocks, momentum }
    }

    /// Returns the contained account blocks.
    pub fn blocks(&self) -> &[AccountBlock] {
        &self.blocks
    }

    /// Returns the momentum.
    pub fn momentum(&self) -> &Momentum {
        &self.momentum
    }

    /// Serializes the detailed momentum to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "blocks": self.blocks.iter().map(AccountBlock::to_json).collect::<Vec<_>>(),
            "momentum": self.momentum.to_json(),
        })
    }

    /// Deserializes a detailed momentum from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "detailed momentum")?;
        let blocks = required_array_ref(object, "blocks")?
            .iter()
            .map(AccountBlock::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        let momentum = Momentum::from_json(required_value(object, "momentum")?)?;
        Ok(Self::new(blocks, momentum))
    }
}

/// A page of detailed momentums.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DetailedMomentumList {
    /// Page count, when present.
    pub count: Option<u64>,
    /// Page list, when present.
    pub list: Option<Vec<DetailedMomentum>>,
}

impl DetailedMomentumList {
    /// Serializes the page to a JSON object.
    pub fn to_json(&self) -> Value {
        let mut object = Map::new();
        if let Some(count) = self.count {
            object.insert("count".to_string(), json!(count));
        }
        if let Some(list) = &self.list {
            object.insert(
                "list".to_string(),
                Value::Array(list.iter().map(DetailedMomentum::to_json).collect()),
            );
        }
        Value::Object(object)
    }

    /// Deserializes a page from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "detailed momentum list")?;
        let count = optional_u64(object, "count")?;
        let list = match object.get("list") {
            Some(Value::Array(values)) => Some(
                values
                    .iter()
                    .map(DetailedMomentum::from_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Some(_) => return Err(Error::InvalidInput("list must be an array".into())),
            None => None,
        };
        Ok(Self { count, list })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::primitives::address::Address;
    use crate::primitives::hash::Hash;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct DetailedMomentumConformance {
        #[allow(dead_code)]
        description: String,
        detailed_momentum: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/nom/detailed_momentum.json");

    fn value() -> Value {
        serde_json::from_str::<DetailedMomentumConformance>(CONFORMANCE)
            .expect("valid detailed momentum conformance")
            .detailed_momentum
    }

    fn sample_momentum_with_height(height: u64) -> Momentum {
        Momentum::new(
            1,
            100,
            Hash::parse("c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9")
                .expect("hash parses"),
            Hash::parse("0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712")
                .expect("hash parses"),
            height,
            1_000_000_010,
            Vec::new(),
            Vec::new(),
            Hash::parse("0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8")
                .expect("hash parses"),
            "pk".to_string(),
            "sig".to_string(),
            Address::parse("z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju").expect("address parses"),
        )
    }

    fn sample_momentum() -> Momentum {
        sample_momentum_with_height(7)
    }

    #[test]
    fn new_and_accessors_return_each_field() {
        let momentum = sample_momentum();
        let detailed = DetailedMomentum::new(Vec::new(), momentum.clone());
        assert!(
            detailed.blocks().is_empty(),
            "blocks accessor returns the empty list passed to new"
        );
        assert_eq!(
            detailed.momentum(),
            &momentum,
            "momentum accessor returns the momentum passed to new"
        );
    }

    #[test]
    fn to_json_matches_the_conformance_value() {
        let detailed = DetailedMomentum::from_json(&value()).expect("conformance value parses");
        assert_eq!(detailed.to_json(), value());
    }

    #[test]
    fn from_json_reads_the_conformance_fields() {
        let detailed = DetailedMomentum::from_json(&value()).expect("conformance value parses");
        assert_eq!(detailed.blocks().len(), 1);
        assert_eq!(detailed.momentum().height(), 2);
    }

    #[test]
    fn round_trips_through_to_json_and_from_json() {
        // Anchor on a constructor-built value so a broken serializer/parser
        // pair cannot satisfy the round-trip: the re-parsed momentum must keep
        // height 7, not collapse to the placeholder's height 0.
        let original = DetailedMomentum::new(Vec::new(), sample_momentum());
        let round_trip =
            DetailedMomentum::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn rejects_a_malformed_detailed_momentum_object() {
        let mut missing = value();
        missing
            .as_object_mut()
            .expect("value is an object")
            .remove("momentum");
        let result = DetailedMomentum::from_json(&missing);
        assert!(result.is_err(), "missing momentum must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut bad_blocks = value();
        bad_blocks["blocks"] = serde_json::json!("not-an-array");
        let result = DetailedMomentum::from_json(&bad_blocks);
        assert!(result.is_err(), "non-array blocks must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn detailed_momentum_list_round_trips() {
        let list_value = serde_json::json!({ "count": 1, "list": [value()] });
        let list = DetailedMomentumList::from_json(&list_value).expect("list parses");
        assert_eq!(list.count, Some(1), "count is decoded from input");
        assert_eq!(
            list.list.as_ref().map(Vec::len),
            Some(1),
            "list is decoded from input"
        );
        assert_eq!(list.to_json(), list_value);
    }

    #[test]
    fn not_equal_when_momenta_differ() {
        let a = DetailedMomentum::new(Vec::new(), sample_momentum_with_height(7));
        let b = DetailedMomentum::new(Vec::new(), sample_momentum_with_height(8));
        assert_ne!(a, b);
    }
}

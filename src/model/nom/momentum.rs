//! Zenon `Momentum`, `MomentumList`, and `MomentumShort` ledger models.
//!
//! A [`Momentum`] is a chain checkpoint. Its JSON form encodes the `data` byte
//! field as standard base64 and `content` as an array of account-header objects.

use crate::error::Error;
use crate::model::json::*;
use crate::model::nom::account_header::AccountHeader;
use crate::primitives::address::Address;
#[cfg(test)]
use crate::primitives::address::{CORE_SIZE as ADDRESS_CORE_SIZE, PREFIX as ADDRESS_PREFIX};
use crate::primitives::hash::Hash;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Map, Value, json};

/// A momentum: a chain checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Momentum {
    version: u32,
    chain_identifier: u32,
    hash: Hash,
    previous_hash: Hash,
    height: u64,
    timestamp: u64,
    data: Vec<u8>,
    content: Vec<AccountHeader>,
    changes_hash: Hash,
    public_key: String,
    signature: String,
    producer: Address,
}

impl Momentum {
    /// Creates a momentum from its fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u32,
        chain_identifier: u32,
        hash: Hash,
        previous_hash: Hash,
        height: u64,
        timestamp: u64,
        data: Vec<u8>,
        content: Vec<AccountHeader>,
        changes_hash: Hash,
        public_key: String,
        signature: String,
        producer: Address,
    ) -> Self {
        Self {
            version,
            chain_identifier,
            hash,
            previous_hash,
            height,
            timestamp,
            data,
            content,
            changes_hash,
            public_key,
            signature,
            producer,
        }
    }

    /// Returns the version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns the chain identifier.
    pub fn chain_identifier(&self) -> u32 {
        self.chain_identifier
    }

    /// Returns the hash.
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    /// Returns the previous hash.
    pub fn previous_hash(&self) -> &Hash {
        &self.previous_hash
    }

    /// Returns the height.
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the content headers.
    pub fn content(&self) -> &[AccountHeader] {
        &self.content
    }

    /// Returns the changes hash.
    pub fn changes_hash(&self) -> &Hash {
        &self.changes_hash
    }

    /// Returns the producer public key.
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// Returns the signature.
    pub fn signature(&self) -> &str {
        &self.signature
    }

    /// Returns the producer address.
    pub fn producer(&self) -> &Address {
        &self.producer
    }

    /// Serializes the momentum to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "chainIdentifier": self.chain_identifier,
            "hash": self.hash.to_string(),
            "previousHash": self.previous_hash.to_string(),
            "height": self.height,
            "timestamp": self.timestamp,
            "data": STANDARD.encode(&self.data),
            "content": self.content.iter().map(AccountHeader::to_json).collect::<Vec<_>>(),
            "changesHash": self.changes_hash.to_string(),
            "publicKey": self.public_key,
            "signature": self.signature,
            "producer": self.producer.to_string(),
        })
    }

    /// Deserializes a momentum from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "momentum")?;
        let content = required_array_ref(object, "content")?
            .iter()
            .map(AccountHeader::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::new(
            required_u32(object, "version")?,
            required_u32(object, "chainIdentifier")?,
            Hash::parse(required_str(object, "hash")?)?,
            Hash::parse(required_str(object, "previousHash")?)?,
            required_u64(object, "height")?,
            required_u64(object, "timestamp")?,
            required_base64(object, "data")?,
            content,
            Hash::parse(required_str(object, "changesHash")?)?,
            required_str(object, "publicKey")?.to_string(),
            required_str(object, "signature")?.to_string(),
            Address::parse(required_str(object, "producer")?)?,
        ))
    }

    #[cfg(test)]
    #[allow(clippy::expect_used)]
    fn placeholder() -> Self {
        Self {
            version: 0,
            chain_identifier: 0,
            hash: Hash::empty(),
            previous_hash: Hash::empty(),
            height: 0,
            timestamp: 0,
            data: Vec::new(),
            content: Vec::new(),
            changes_hash: Hash::empty(),
            public_key: String::new(),
            signature: String::new(),
            producer: Address::new(ADDRESS_PREFIX, &[0u8; ADDRESS_CORE_SIZE])
                .expect("20 zero bytes form a valid address core"),
        }
    }
}

/// A page of momentums.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MomentumList {
    /// Page count.
    pub count: u64,
    /// Page list.
    pub list: Vec<Momentum>,
}

impl MomentumList {
    /// Serializes the page to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(Momentum::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes a page from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "momentum list")?;
        let list = required_array_ref(object, "list")?
            .iter()
            .map(Momentum::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            count: required_u64(object, "count")?,
            list,
        })
    }
}

/// A short momentum projection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MomentumShort {
    /// Momentum hash, when present.
    pub hash: Option<Hash>,
    /// Momentum height, when present.
    pub height: Option<u64>,
    /// Momentum timestamp, when present.
    pub timestamp: Option<u64>,
}

impl MomentumShort {
    /// Serializes the projection to a JSON object.
    pub fn to_json(&self) -> Value {
        let mut object = Map::new();
        if let Some(hash) = &self.hash {
            object.insert("hash".to_string(), json!(hash.to_string()));
        }
        if let Some(height) = self.height {
            object.insert("height".to_string(), json!(height));
        }
        if let Some(timestamp) = self.timestamp {
            object.insert("timestamp".to_string(), json!(timestamp));
        }
        Value::Object(object)
    }

    /// Deserializes a projection from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "momentum short")?;
        Ok(Self {
            hash: optional_hash(object, "hash")?,
            height: optional_u64(object, "height")?,
            timestamp: optional_u64(object, "timestamp")?,
        })
    }
}

/// `optional_u64` kept local: unlike the canonical helper, it treats a JSON
/// `null` the same as an absent field (`None`) rather than rejecting it. The
/// `MomentumShort` projection may carry `null` for `height`/`timestamp`.
fn optional_u64(object: &Map<String, Value>, field: &str) -> Result<Option<u64>, Error> {
    match object.get(field) {
        Some(Value::Null) | None => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| Error::InvalidInput(format!("{field} must be an unsigned integer"))),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn content_header_value() -> Value {
        json!({
            "address": "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz",
            "hash": "3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73",
            "height": 2
        })
    }

    fn momentum_value_with_content() -> Value {
        let mut value = json!({
            "version": 1,
            "chainIdentifier": 100,
            "hash": "c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9",
            "previousHash": "0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712",
            "height": 2,
            "timestamp": 1_000_000_010,
            "data": "",
            "content": [],
            "changesHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
            "publicKey": "SAPwVIVQma3zMak169crdLkcu2B2Gm3iBCdDgfQ6IxU=",
            "signature": "qvlKN6rTQgM11/FosNazpeReViuD1GH1tIww2F0zNfXruTp3g9ULhA1mYnRYAiPJyP2NlIGhENwhzBAHJ0dYBw==",
            "producer": "z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju"
        });
        value["content"] = json!([content_header_value()]);
        value
    }

    #[test]
    fn new_and_accessors_return_each_field() {
        let header = AccountHeader::new(
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            Hash::parse("3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73")
                .unwrap(),
            2,
        );
        let momentum = Momentum::new(
            1,
            100,
            Hash::parse("c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9")
                .unwrap(),
            Hash::parse("0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712")
                .unwrap(),
            2,
            1_000_000_010,
            Vec::new(),
            vec![header],
            Hash::parse("0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8")
                .unwrap(),
            "pk".to_string(),
            "sig".to_string(),
            Address::parse("z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju").unwrap(),
        );
        assert_eq!(momentum.version(), 1);
        assert_eq!(momentum.height(), 2);
        assert_eq!(momentum.timestamp(), 1_000_000_010);
        assert_eq!(momentum.content().len(), 1);
    }

    #[test]
    fn content_serializes_as_account_header_objects() {
        let value = momentum_value_with_content();
        let momentum = Momentum::from_json(&value).expect("momentum parses");
        let serialized = momentum.to_json();
        assert_eq!(serialized["content"], json!([content_header_value()]));
    }

    #[test]
    fn rejects_a_malformed_momentum_object() {
        let mut missing = momentum_value_with_content();
        missing
            .as_object_mut()
            .expect("momentum is an object")
            .remove("height");
        let result = Momentum::from_json(&missing);
        assert!(result.is_err(), "missing height must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut bad_hash = momentum_value_with_content();
        bad_hash["hash"] = json!("not-a-hash");
        let result = Momentum::from_json(&bad_hash);
        assert!(result.is_err(), "non-canonical hash must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn momentum_list_round_trips() {
        let value = json!({
            "count": 1,
            "list": [momentum_value_with_content()]
        });
        let list = MomentumList::from_json(&value).expect("list parses");
        assert_eq!(list.count, 1, "count is decoded from input");
        assert_eq!(list.list.len(), 1, "list is decoded from input");
        assert_eq!(list.to_json(), value);
    }

    #[test]
    fn momentum_short_round_trips() {
        let original = MomentumShort {
            hash: Some(
                Hash::parse("c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9")
                    .unwrap(),
            ),
            height: Some(2),
            timestamp: Some(1_000_000_010),
        };
        let round_trip = MomentumShort::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn not_equal_when_heights_differ() {
        let a = Momentum {
            height: 2,
            ..Momentum::placeholder()
        };
        let b = Momentum {
            height: 3,
            ..Momentum::placeholder()
        };
        assert_ne!(a, b);
    }
}

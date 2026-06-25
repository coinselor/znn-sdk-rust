//! Zenon `HashHeight` type.
//!
//! A `HashHeight` pairs a [`struct@Hash`] with an unsigned block or momentum height and
//! encodes as the JSON object `{"hash": <canonical hex>, "height": <unsigned int>}`.

use crate::error::Error;
use crate::primitives::hash::Hash;

/// Byte length of the serialized [`HashHeight`] form: the 32 hash bytes
/// followed by the 8-byte big-endian height.
pub const BYTES_LENGTH: usize = crate::primitives::hash::LENGTH + 8;

/// A Zenon hash height: a [`struct@Hash`] paired with an unsigned height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashHeight {
    hash: Hash,
    height: u64,
}

impl HashHeight {
    /// Creates a hash height from a hash and a height.
    pub fn new(hash: Hash, height: u64) -> Self {
        Self { hash, height }
    }

    /// Returns the empty hash height: the all-zeros hash at height `0`.
    pub fn empty() -> Self {
        Self {
            hash: Hash::empty(),
            height: 0,
        }
    }

    /// Returns the hash.
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    /// Returns the height.
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Serializes to the JSON object `{"hash": <canonical hex>, "height": <unsigned int>}`,
    /// where the hash is its canonical lowercase 64-character hex string.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "hash": self.hash.to_string(),
            "height": self.height,
        })
    }

    /// Deserializes from a JSON object, parsing the `hash` field with
    /// [`Hash::parse`] and reading the `height` field as an unsigned 64-bit
    /// integer.
    ///
    /// Returns [`Error::InvalidInput`] when the value is not a JSON object,
    /// when the `hash` or `height` field is missing, when the `hash` field is
    /// not a canonical hash hex string, or when the `height` field is not an
    /// unsigned 64-bit integer.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        let object = value
            .as_object()
            .ok_or_else(|| Error::InvalidInput("hash height must be a JSON object".into()))?;
        let hash = object
            .get("hash")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| Error::InvalidInput("missing or non-string hash field".into()))?;
        let hash = Hash::parse(hash)?;
        let height = object
            .get("height")
            .ok_or_else(|| Error::InvalidInput("missing height field".into()))?
            .as_u64()
            .ok_or_else(|| {
                Error::InvalidInput("height must be an unsigned 64-bit integer".into())
            })?;
        Ok(Self::new(hash, height))
    }

    /// Returns the serialized byte form: the 32 hash bytes followed by the
    /// 8-byte big-endian encoding of the height ([`BYTES_LENGTH`] bytes total).
    pub fn get_bytes(&self) -> [u8; BYTES_LENGTH] {
        let mut bytes = [0u8; BYTES_LENGTH];
        let (hash_part, height_part) = bytes.split_at_mut(crate::primitives::hash::LENGTH);
        hash_part.copy_from_slice(self.hash.bytes());
        height_part.copy_from_slice(&crate::utils::bytes::long_to_bytes(self.height));
        bytes
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct HashHeightVectors {
        hash_heights: Vec<HashHeightVector>,
    }

    #[derive(Deserialize)]
    struct HashHeightVector {
        name: String,
        hash: String,
        height: u64,
    }

    #[derive(Deserialize)]
    struct InvalidVectors {
        invalid: Vec<InvalidVector>,
    }

    #[derive(Deserialize)]
    struct InvalidVector {
        name: String,
        hash: String,
        height: u64,
    }

    #[derive(Deserialize)]
    struct MalformedVectors {
        malformed: Vec<MalformedVector>,
    }

    #[derive(Deserialize)]
    struct MalformedVector {
        name: String,
        value: serde_json::Value,
    }

    #[derive(Deserialize)]
    struct GetBytesVectors {
        cases: Vec<GetBytesCase>,
    }

    #[derive(Deserialize)]
    struct GetBytesCase {
        name: String,
        hash: String,
        height: u64,
        bytes: String,
    }

    const HASH_HEIGHTS: &str =
        include_str!("../../tests/vectors/primitives/hash_height/hash_heights.json");
    const INVALID: &str = include_str!("../../tests/vectors/primitives/hash_height/invalid.json");
    const MALFORMED: &str =
        include_str!("../../tests/vectors/primitives/hash_height/malformed.json");
    const GET_BYTES: &str =
        include_str!("../../tests/vectors/primitives/hash_height/get_bytes.json");
    const NON_ZERO_HEX: &str = "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a";
    const OTHER_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn hash_heights() -> Vec<HashHeightVector> {
        serde_json::from_str::<HashHeightVectors>(HASH_HEIGHTS)
            .expect("valid hash height vectors")
            .hash_heights
    }

    fn invalid() -> Vec<InvalidVector> {
        serde_json::from_str::<InvalidVectors>(INVALID)
            .expect("valid invalid vectors")
            .invalid
    }

    fn malformed() -> Vec<MalformedVector> {
        serde_json::from_str::<MalformedVectors>(MALFORMED)
            .expect("valid malformed vectors")
            .malformed
    }

    fn get_bytes_cases() -> Vec<GetBytesCase> {
        serde_json::from_str::<GetBytesVectors>(GET_BYTES)
            .expect("valid get_bytes vectors")
            .cases
    }

    // HashHeight type and accessors.

    #[test]
    fn new_preserves_hash_and_height() {
        let hash = Hash::parse(NON_ZERO_HEX).expect("vector hash parses");
        let hh = HashHeight::new(hash.clone(), 42);
        assert_eq!(hh.hash(), &hash, "hash accessor must return the input hash");
        assert_eq!(
            hh.height(),
            42,
            "height accessor must return the input height"
        );
    }

    // Empty hash height.

    #[test]
    fn empty_is_the_empty_hash_at_height_zero() {
        let hh = HashHeight::empty();
        assert_eq!(
            hh.hash(),
            &Hash::empty(),
            "empty hash height uses the empty hash"
        );
        assert_eq!(hh.height(), 0, "empty hash height is at height 0");
    }

    // JSON serialization.

    #[test]
    fn to_json_matches_expected_object() {
        for v in hash_heights() {
            let hash = Hash::parse(&v.hash).expect("vector hash parses");
            let hh = HashHeight::new(hash, v.height);
            let expected = serde_json::json!({ "hash": v.hash, "height": v.height });
            assert_eq!(hh.to_json(), expected, "to_json for {}", v.name);
        }
    }

    #[test]
    fn to_json_height_is_an_unsigned_number() {
        let hash = Hash::parse(NON_ZERO_HEX).expect("vector hash parses");
        let hh = HashHeight::new(hash, 100);
        let json = hh.to_json();
        assert_eq!(
            json.get("height").and_then(serde_json::Value::as_u64),
            Some(100),
            "height must serialize as the unsigned number 100"
        );
    }

    // JSON deserialization.

    #[test]
    fn from_json_decodes_a_non_empty_value() {
        let object = serde_json::json!({ "hash": NON_ZERO_HEX, "height": 42 });
        let hh = HashHeight::from_json(&object).expect("non-empty object decodes");
        let expected_hash = Hash::parse(NON_ZERO_HEX).expect("hex parses");
        assert_eq!(
            hh.hash(),
            &expected_hash,
            "hash must be parsed from the hex field"
        );
        assert_eq!(hh.height(), 42, "height must be read from the height field");
    }

    #[test]
    fn from_json_round_trips_a_non_empty_value() {
        let hash = Hash::parse(NON_ZERO_HEX).expect("hex parses");
        let original = HashHeight::new(hash, 42);
        let decoded = HashHeight::from_json(&original.to_json()).expect("round-trip decodes");
        assert_eq!(
            decoded, original,
            "non-empty value must survive to_json then from_json"
        );
    }

    #[test]
    fn from_json_decodes_known_vectors() {
        for v in hash_heights() {
            let object = serde_json::json!({ "hash": v.hash, "height": v.height });
            let hh = HashHeight::from_json(&object).expect("vector object decodes");
            let expected_hash = Hash::parse(&v.hash).expect("vector hash parses");
            assert_eq!(hh.hash(), &expected_hash, "decoded hash for {}", v.name);
            assert_eq!(hh.height(), v.height, "decoded height for {}", v.name);
        }
    }

    #[test]
    fn from_json_round_trips_through_to_json() {
        for v in hash_heights() {
            let hash = Hash::parse(&v.hash).expect("vector hash parses");
            let hh = HashHeight::new(hash, v.height);
            let decoded = HashHeight::from_json(&hh.to_json()).expect("round-trip decodes");
            assert_eq!(decoded, hh, "round-trip for {}", v.name);
        }
    }

    #[test]
    fn from_json_rejects_each_invalid_hash_as_invalid_input() {
        for v in invalid() {
            let object = serde_json::json!({ "hash": v.hash, "height": v.height });
            let err =
                HashHeight::from_json(&object).expect_err(&format!("{} must be rejected", v.name));
            assert!(
                matches!(err, Error::InvalidInput(_)),
                "expected Error::InvalidInput for {}, got {err:?}",
                v.name
            );
        }
    }

    #[test]
    fn from_json_rejects_each_malformed_value_as_invalid_input() {
        for v in malformed() {
            let err =
                HashHeight::from_json(&v.value).expect_err(&format!("{} must be rejected", v.name));
            assert!(
                matches!(err, Error::InvalidInput(_)),
                "expected Error::InvalidInput for {}, got {err:?}",
                v.name
            );
        }
    }

    // Equality.

    #[test]
    fn equal_when_hash_and_height_match() {
        let hash = Hash::parse(NON_ZERO_HEX).expect("parses");
        let a = HashHeight::new(hash.clone(), 7);
        let b = HashHeight::new(hash, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn not_equal_when_heights_differ() {
        let hash = Hash::parse(NON_ZERO_HEX).expect("parses");
        let a = HashHeight::new(hash.clone(), 7);
        let b = HashHeight::new(hash, 8);
        assert_ne!(a, b, "values with different heights must not be equal");
    }

    #[test]
    fn not_equal_when_hashes_differ() {
        let a = HashHeight::new(Hash::parse(NON_ZERO_HEX).expect("parses"), 7);
        let b = HashHeight::new(Hash::parse(OTHER_HEX).expect("parses"), 7);
        assert_ne!(a, b, "values with different hashes must not be equal");
    }

    // Serialized bytes (get_bytes).

    #[test]
    fn get_bytes_matches_known_vectors() {
        for case in get_bytes_cases() {
            let hash = Hash::parse(&case.hash).expect("vector hash parses");
            let hh = HashHeight::new(hash, case.height);
            let expected = const_hex::decode(&case.bytes).expect("vector bytes decode");
            assert_eq!(
                hh.get_bytes().as_slice(),
                expected.as_slice(),
                "get_bytes for {}",
                case.name
            );
        }
    }

    #[test]
    fn get_bytes_layout_is_hash_then_big_endian_height() {
        let hash = Hash::parse(NON_ZERO_HEX).expect("parses");
        let hh = HashHeight::new(hash.clone(), 42);
        let bytes = hh.get_bytes();
        assert_eq!(bytes.len(), 40, "serialized form is 40 bytes");
        let (head, tail) = bytes.split_at(32);
        assert_eq!(head, hash.bytes().as_slice(), "first 32 bytes are the hash");
        assert_eq!(
            tail,
            42u64.to_be_bytes().as_slice(),
            "last 8 bytes are the big-endian height"
        );
    }

    #[test]
    fn empty_hash_height_get_bytes_is_forty_zeros() {
        assert_eq!(HashHeight::empty().get_bytes(), [0u8; 40]);
    }
}

//! Zenon `Hash` type.
//!
//! A `Hash` wraps a 32-byte value and encodes as a canonical lowercase 64-character
//! hex string.

use crate::error::Error;
use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

/// Byte length of a `Hash`.
pub const LENGTH: usize = 32;

/// A Zenon hash: a 32-byte value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hash {
    bytes: [u8; LENGTH],
}

impl Hash {
    /// Creates a hash from a 32-byte value, rejecting an input whose length is
    /// not [`LENGTH`] with [`Error::InvalidInput`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let bytes: [u8; LENGTH] = bytes.try_into().map_err(|_| {
            Error::InvalidInput(format!("hash must be {LENGTH} bytes, got {}", bytes.len()))
        })?;
        Ok(Self { bytes })
    }

    /// Parses exactly 64 hexadecimal characters into a 32-byte hash, returning
    /// [`Error::InvalidInput`] for a malformed string or wrong decoded length.
    ///
    /// The input must be a bare hex string of exactly `2 * LENGTH` characters.
    /// An `0x` or `0X` prefix is rejected. Uppercase and mixed-case hex are
    /// accepted on input; [`fmt::Display`] always renders the canonical
    /// lowercase form.
    pub fn parse(s: &str) -> Result<Self, Error> {
        if s.len() != 2 * LENGTH {
            return Err(Error::InvalidInput(format!(
                "hash hex must be {} characters, got {}",
                2 * LENGTH,
                s.len()
            )));
        }
        let bytes = const_hex::decode(s)
            .map_err(|e| Error::InvalidInput(format!("invalid hash hex: {e}")))?;
        Self::from_bytes(&bytes)
    }

    /// Returns the empty hash, the all-zeros 32-byte value.
    pub fn empty() -> Self {
        Self {
            bytes: [0u8; LENGTH],
        }
    }

    /// Returns the 32 bytes.
    pub fn bytes(&self) -> &[u8; LENGTH] {
        &self.bytes
    }

    /// Returns the short form: first 6 characters, `...`, then the last 6 of the
    /// canonical hex string.
    pub fn to_short_string(&self) -> String {
        let s = self.to_string();
        format!("{}...{}", &s[..6], &s[s.len() - 6..])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&const_hex::encode(self.bytes))
    }
}

impl FromStr for Hash {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Ord for Hash {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl PartialOrd for Hash {
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
    struct HashVectors {
        hashes: Vec<HashVector>,
    }

    #[derive(Deserialize)]
    struct HashVector {
        name: String,
        hex: String,
        short: String,
    }

    #[derive(Deserialize)]
    struct InvalidVectors {
        invalid: Vec<InvalidVector>,
    }

    #[derive(Deserialize)]
    struct InvalidVector {
        name: String,
        hex: String,
    }

    const HASHES: &str = include_str!("../../tests/vectors/primitives/hash/hashes.json");
    const INVALID: &str = include_str!("../../tests/vectors/primitives/hash/invalid.json");
    const EMPTY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn hashes() -> Vec<HashVector> {
        serde_json::from_str::<HashVectors>(HASHES)
            .expect("valid hash vectors")
            .hashes
    }

    fn invalid() -> Vec<InvalidVector> {
        serde_json::from_str::<InvalidVectors>(INVALID)
            .expect("valid invalid vectors")
            .invalid
    }

    // Hash type and constant.

    #[test]
    fn length_constant_is_32() {
        assert_eq!(LENGTH, 32);
    }

    #[test]
    fn from_bytes_preserves_a_32_byte_value() {
        let input = [0xABu8; 32];
        let h = Hash::from_bytes(&input).expect("32-byte value is accepted");
        assert_eq!(h.bytes(), &input, "from_bytes must preserve the 32 bytes");
    }

    #[test]
    fn from_bytes_rejects_31_bytes_as_invalid_input() {
        let err = Hash::from_bytes(&[0u8; 31]).expect_err("31 bytes must be rejected");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "expected Error::InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn from_bytes_rejects_33_bytes_as_invalid_input() {
        let err = Hash::from_bytes(&[0u8; 33]).expect_err("33 bytes must be rejected");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "expected Error::InvalidInput, got {err:?}"
        );
    }

    // Parsing and canonical encoding.

    #[test]
    fn parse_empty_hash_is_all_zeros() {
        let h = Hash::parse(EMPTY_HEX).expect("empty hash parses");
        assert_eq!(h.bytes(), &[0u8; 32]);
    }

    #[test]
    fn parse_decodes_known_vectors_to_their_bytes() {
        for v in hashes() {
            let expected = const_hex::decode(&v.hex).expect("vector hex decodes");
            let h = Hash::parse(&v.hex).expect("vector parses");
            assert_eq!(
                h.bytes().as_slice(),
                expected.as_slice(),
                "decoded bytes for {}",
                v.name
            );
        }
    }

    #[test]
    fn display_is_canonical_lowercase_hex_round_trip() {
        for v in hashes() {
            let h = Hash::parse(&v.hex).expect("vector parses");
            assert_eq!(h.to_string(), v.hex, "canonical hex for {}", v.name);
        }
    }

    #[test]
    fn from_str_delegates_to_parse() {
        let h: Hash = EMPTY_HEX.parse().expect("FromStr parses the empty hash");
        assert_eq!(h.bytes(), &[0u8; 32]);
    }

    #[test]
    fn parse_rejects_each_invalid_vector_as_invalid_input() {
        for v in invalid() {
            let result = Hash::parse(&v.hex);
            let err = result.expect_err(&format!("{} must be rejected", v.name));
            assert!(
                matches!(err, Error::InvalidInput(_)),
                "expected Error::InvalidInput for {}, got {err:?}",
                v.name
            );
        }
    }

    // Empty hash.

    #[test]
    fn empty_hash_bytes_are_all_zeros() {
        assert_eq!(Hash::empty().bytes(), &[0u8; 32]);
    }

    #[test]
    fn empty_hash_display_is_64_zeros() {
        assert_eq!(Hash::empty().to_string(), EMPTY_HEX);
    }

    // Short string form.

    #[test]
    fn to_short_string_matches_vectors() {
        for v in hashes() {
            let h = Hash::parse(&v.hex).expect("vector parses");
            assert_eq!(h.to_short_string(), v.short, "short form for {}", v.name);
        }
    }

    #[test]
    fn to_short_string_is_first_6_dots_last_6() {
        let v = hashes()
            .into_iter()
            .find(|v| v.name == "sha3_256_empty")
            .expect("sha3_256_empty vector present");
        let h = Hash::parse(&v.hex).expect("parses");
        let short = h.to_short_string();
        // Validate the length first so format regressions fail as assertions
        // instead of panicking in the substring checks below.
        assert_eq!(
            short.len(),
            15,
            "short form is 6 + '...' + 6 = 15 chars, got {short:?}"
        );
        assert_eq!(&short[..6], &v.hex[..6], "first 6 chars");
        assert_eq!(&short[6..9], "...", "separator");
        assert_eq!(&short[9..], &v.hex[v.hex.len() - 6..], "last 6 chars");
    }

    #[test]
    fn parse_is_case_insensitive_and_display_is_canonical_lowercase() {
        let v = hashes()
            .into_iter()
            .find(|v| v.name == "sha3_256_empty")
            .expect("sha3_256_empty vector present");
        let upper = v.hex.to_uppercase();
        let mixed: String = v
            .hex
            .char_indices()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();
        for input in [&upper, &mixed] {
            let h = Hash::parse(input).expect("upper/mixed-case hex parses");
            assert_eq!(
                h.to_string(),
                v.hex,
                "Display must normalize {input} to canonical lowercase"
            );
        }
    }

    // Equality and ordering.

    #[test]
    fn equal_hashes_compare_equal() {
        let a = Hash::parse(EMPTY_HEX).expect("parses");
        let b = Hash::parse(EMPTY_HEX).expect("parses");
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_hashes_are_not_equal() {
        let zeros = Hash::parse(EMPTY_HEX).expect("parses");
        let ones = Hash::parse("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
            .expect("parses");
        assert_ne!(zeros, ones);
    }

    #[test]
    fn ordering_follows_canonical_hex_lexicographic() {
        let a_hex = EMPTY_HEX;
        let b_hex = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let a = Hash::parse(a_hex).expect("parses");
        let b = Hash::parse(b_hex).expect("parses");
        assert_eq!(a.cmp(&b), Ordering::Less, "all-zeros < all-ones");
        assert_eq!(b.cmp(&a), Ordering::Greater, "all-ones > all-zeros");
        assert_eq!(
            a.cmp(&b),
            a_hex.cmp(b_hex),
            "matches hex lexicographic order"
        );
    }
}

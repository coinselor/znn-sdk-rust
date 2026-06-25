//! Zenon `TokenStandard` (ZTS) type.
//!
//! A `TokenStandard` is a human-readable part (`zts`) and a 10-byte core, encoded
//! as a canonical lowercase Bech32 string. It identifies a Zenon token.

use crate::error::Error;
use bech32::primitives::decode::CheckedHrpstring;
use bech32::{Bech32, Hrp};
use core::fmt;
use core::str::FromStr;

/// Human-readable part of a Zenon token standard.
pub const PREFIX: &str = "zts";
/// Byte length of a token standard core.
pub const CORE_SIZE: usize = 10;

/// Canonical string of the ZNN token standard.
pub const ZNN_TOKEN_STANDARD: &str = "zts1znnxxxxxxxxxxxxx9z4ulx";
/// Canonical string of the QSR token standard.
pub const QSR_TOKEN_STANDARD: &str = "zts1qsrxxxxxxxxxxxxxmrhjll";
/// Canonical string of the empty token standard.
pub const EMPTY_TOKEN_STANDARD: &str = "zts1qqqqqqqqqqqqqqqqtq587y";

/// Core of the ZNN token standard.
const ZNN_CORE: [u8; CORE_SIZE] = [0x14, 0xe6, 0x63, 0x18, 0xc6, 0x31, 0x8c, 0x63, 0x18, 0xc6];
/// Core of the QSR token standard.
const QSR_CORE: [u8; CORE_SIZE] = [0x04, 0x06, 0x63, 0x18, 0xc6, 0x31, 0x8c, 0x63, 0x18, 0xc6];
/// Core of the empty token standard.
const EMPTY_CORE: [u8; CORE_SIZE] = [0u8; CORE_SIZE];

/// A Zenon token standard: a human-readable part and a 10-byte core.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenStandard {
    hrp: String,
    core: [u8; CORE_SIZE],
}

impl TokenStandard {
    /// Parses a canonical lowercase Zenon `zts` Bech32 token standard.
    ///
    /// Uses the Bech32 checksum variant (a Bech32m string is rejected) and
    /// validates that the human-readable part is `zts` and the decoded core is
    /// [`CORE_SIZE`] bytes. An uppercase string decodes to the human-readable
    /// part `ZTS` and is rejected by the prefix check; a mixed-case string is
    /// rejected by the Bech32 parser. Returns [`Error::InvalidInput`] for any
    /// malformed, wrong-variant, wrong-case, wrong-prefix, or wrong-length
    /// input.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let checked = CheckedHrpstring::new::<Bech32>(s)
            .map_err(|e| Error::InvalidInput(format!("invalid bech32 token standard: {e}")))?;

        let hrp = checked.hrp().to_string();
        if hrp != PREFIX {
            return Err(Error::InvalidInput(format!(
                "unexpected human-readable part: {hrp}"
            )));
        }

        let core: Vec<u8> = checked.byte_iter().collect();
        Self::from_bytes(&core)
    }

    /// Builds a token standard from a [`CORE_SIZE`]-byte core, setting the
    /// human-readable part to [`PREFIX`]. Returns [`Error::InvalidInput`] for
    /// any other length.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let core: [u8; CORE_SIZE] = bytes.try_into().map_err(|_| {
            Error::InvalidInput(format!(
                "token standard core must be {CORE_SIZE} bytes, got {}",
                bytes.len()
            ))
        })?;
        Ok(Self {
            hrp: PREFIX.to_string(),
            core,
        })
    }

    /// Resolves `znn`/`ZNN` to the ZNN token standard and `qsr`/`QSR` to the QSR
    /// token standard. Returns [`Error::InvalidInput`] for any other symbol.
    pub fn by_symbol(symbol: &str) -> Result<Self, Error> {
        match symbol {
            "znn" | "ZNN" => Ok(znn_token_standard()),
            "qsr" | "QSR" => Ok(qsr_token_standard()),
            other => Err(Error::InvalidInput(format!(
                "unknown token symbol: {other}"
            ))),
        }
    }

    /// Returns the human-readable part.
    pub fn hrp(&self) -> &str {
        &self.hrp
    }

    /// Returns the 10-byte core.
    pub fn core(&self) -> &[u8; CORE_SIZE] {
        &self.core
    }

    /// Alias for [`core`](Self::core), named to match the Dart/TS SDKs.
    pub fn get_bytes(&self) -> &[u8; CORE_SIZE] {
        self.core()
    }
}

/// Returns the ZNN token standard.
pub fn znn_token_standard() -> TokenStandard {
    TokenStandard {
        hrp: PREFIX.to_string(),
        core: ZNN_CORE,
    }
}

/// Returns the QSR token standard.
pub fn qsr_token_standard() -> TokenStandard {
    TokenStandard {
        hrp: PREFIX.to_string(),
        core: QSR_CORE,
    }
}

/// Returns the empty token standard.
pub fn empty_token_standard() -> TokenStandard {
    TokenStandard {
        hrp: PREFIX.to_string(),
        core: EMPTY_CORE,
    }
}

impl fmt::Display for TokenStandard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hrp = Hrp::parse(&self.hrp).map_err(|_| fmt::Error)?;
        let s = bech32::encode::<Bech32>(hrp, &self.core).map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

impl FromStr for TokenStandard {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TokenStandardVectors {
        token_standards: Vec<TokenStandardVector>,
    }

    #[derive(Deserialize)]
    struct TokenStandardVector {
        name: String,
        zts: String,
        core_hex: String,
        symbols: Vec<String>,
    }

    #[derive(Deserialize)]
    struct InvalidVectors {
        invalid: Vec<InvalidVector>,
    }

    #[derive(Deserialize)]
    struct InvalidVector {
        name: String,
        zts: String,
    }

    const TOKEN_STANDARDS: &str =
        include_str!("../../tests/vectors/primitives/token_standard/token_standards.json");
    const INVALID: &str =
        include_str!("../../tests/vectors/primitives/token_standard/invalid.json");
    const ZNN_CORE_HEX: &str = "14e66318c6318c6318c6";

    fn token_standards() -> Vec<TokenStandardVector> {
        serde_json::from_str::<TokenStandardVectors>(TOKEN_STANDARDS)
            .expect("valid token standard vectors")
            .token_standards
    }

    fn invalid() -> Vec<InvalidVector> {
        serde_json::from_str::<InvalidVectors>(INVALID)
            .expect("valid invalid vectors")
            .invalid
    }

    fn vector(name: &str) -> TokenStandardVector {
        token_standards()
            .into_iter()
            .find(|v| v.name == name)
            .expect("named vector present")
    }

    // Type and constants.

    #[test]
    fn prefix_and_core_size_constants() {
        assert_eq!(PREFIX, "zts");
        assert_eq!(CORE_SIZE, 10);
    }

    // Parsing and canonical encoding.

    #[test]
    fn parse_decodes_known_vectors_to_prefix_and_core() {
        for v in token_standards() {
            let expected = const_hex::decode(&v.core_hex).expect("vector hex decodes");
            let ts = TokenStandard::parse(&v.zts).expect("vector parses");
            assert_eq!(ts.hrp(), PREFIX, "human-readable part for {}", v.name);
            assert_eq!(
                ts.core().as_slice(),
                expected.as_slice(),
                "core bytes for {}",
                v.name
            );
        }
    }

    #[test]
    fn parse_empty_token_standard_is_all_zeros() {
        let ts = TokenStandard::parse(EMPTY_TOKEN_STANDARD).expect("empty token standard parses");
        assert_eq!(ts.core(), &[0u8; CORE_SIZE]);
    }

    #[test]
    fn display_round_trips_known_vectors() {
        for v in token_standards() {
            let ts = TokenStandard::parse(&v.zts).expect("vector parses");
            assert_eq!(ts.to_string(), v.zts, "canonical string for {}", v.name);
        }
    }

    #[test]
    fn parse_rejects_each_invalid_vector_as_invalid_input() {
        for v in invalid() {
            let err =
                TokenStandard::parse(&v.zts).expect_err(&format!("{} must be rejected", v.name));
            assert!(
                matches!(err, Error::InvalidInput(_)),
                "expected Error::InvalidInput for {}, got {err:?}",
                v.name
            );
        }
    }

    #[test]
    fn parse_rejects_the_bech32m_variant() {
        // The ZNN core's 10-byte data part encoded with a Bech32m checksum.
        // Token standards use the Bech32 variant, so this must be rejected; an
        // implementation that selected Bech32m would otherwise accept it.
        let err = TokenStandard::parse("zts1znnxxxxxxxxxxxxxs79s6y")
            .expect_err("Bech32m checksum must be rejected");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "expected Error::InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn parse_rejects_the_uppercase_form() {
        // The all-uppercase form of the canonical ZNN token standard. Only the
        // canonical lowercase form is accepted.
        let err = TokenStandard::parse("ZTS1ZNNXXXXXXXXXXXXX9Z4ULX")
            .expect_err("uppercase form must be rejected");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "expected Error::InvalidInput, got {err:?}"
        );
    }

    // from_bytes.

    #[test]
    fn from_bytes_accepts_a_10_byte_core() {
        let core = const_hex::decode(ZNN_CORE_HEX).expect("hex decodes");
        let ts = TokenStandard::from_bytes(&core).expect("10-byte core accepted");
        assert_eq!(ts.hrp(), PREFIX, "from_bytes sets the zts prefix");
        assert_eq!(ts.core().as_slice(), core.as_slice(), "core preserved");
        assert_eq!(
            ts.to_string(),
            ZNN_TOKEN_STANDARD,
            "canonical string of the ZNN core"
        );
    }

    #[test]
    fn from_bytes_rejects_a_9_byte_core() {
        let err = TokenStandard::from_bytes(&[0u8; 9]).expect_err("9 bytes must be rejected");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "expected Error::InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn from_bytes_rejects_an_11_byte_core() {
        let err = TokenStandard::from_bytes(&[0u8; 11]).expect_err("11 bytes must be rejected");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "expected Error::InvalidInput, got {err:?}"
        );
    }

    // by_symbol.

    #[test]
    fn by_symbol_resolves_znn_and_qsr_in_both_cases() {
        let znn = znn_token_standard();
        let qsr = qsr_token_standard();
        for symbol in ["znn", "ZNN"] {
            assert_eq!(
                TokenStandard::by_symbol(symbol).expect("znn symbol resolves"),
                znn,
                "{symbol} must resolve to the ZNN token standard"
            );
        }
        for symbol in ["qsr", "QSR"] {
            assert_eq!(
                TokenStandard::by_symbol(symbol).expect("qsr symbol resolves"),
                qsr,
                "{symbol} must resolve to the QSR token standard"
            );
        }
    }

    #[test]
    fn by_symbol_rejects_an_unknown_symbol() {
        let err = TokenStandard::by_symbol("btc").expect_err("unknown symbol must be rejected");
        assert!(
            matches!(err, Error::InvalidInput(_)),
            "expected Error::InvalidInput, got {err:?}"
        );
    }

    // Well-known constants.

    #[test]
    fn well_known_token_standards_render_canonical_strings() {
        assert_eq!(znn_token_standard().to_string(), ZNN_TOKEN_STANDARD);
        assert_eq!(qsr_token_standard().to_string(), QSR_TOKEN_STANDARD);
        assert_eq!(empty_token_standard().to_string(), EMPTY_TOKEN_STANDARD);
    }

    // Byte core accessor.

    #[test]
    fn get_bytes_returns_the_core() {
        let znn = vector("znn");
        let expected = const_hex::decode(&znn.core_hex).expect("hex decodes");
        let ts = znn_token_standard();
        assert_eq!(
            ts.get_bytes().as_slice(),
            expected.as_slice(),
            "get_bytes returns the ZNN core"
        );
    }

    // FromStr.

    #[test]
    fn from_str_reconstructs_the_parsed_value() {
        let ts: TokenStandard = ZNN_TOKEN_STANDARD.parse().expect("FromStr parses ZNN");
        assert_eq!(ts, znn_token_standard());
    }

    // Equality.

    #[test]
    fn equal_when_core_and_prefix_match() {
        let a = TokenStandard::parse(ZNN_TOKEN_STANDARD).expect("parses");
        let b = TokenStandard::parse(ZNN_TOKEN_STANDARD).expect("parses");
        assert_eq!(a, b);
    }

    #[test]
    fn not_equal_when_cores_differ() {
        let znn = TokenStandard::parse(ZNN_TOKEN_STANDARD).expect("parses");
        let qsr = TokenStandard::parse(QSR_TOKEN_STANDARD).expect("parses");
        assert_ne!(znn, qsr, "ZNN and QSR must not be equal");
    }

    #[test]
    fn not_equal_to_the_empty_token_standard() {
        let znn = TokenStandard::parse(ZNN_TOKEN_STANDARD).expect("parses");
        let empty = TokenStandard::parse(EMPTY_TOKEN_STANDARD).expect("parses");
        assert_ne!(
            znn, empty,
            "ZNN and the empty token standard must not be equal"
        );
    }

    // Symbol vectors are covered by `by_symbol_resolves_znn_and_qsr_in_both_cases`;
    // this guards the vector data itself so a malformed `symbols` field fails here
    // rather than silently weakening that test.
    #[test]
    fn vector_symbols_match_expected() {
        assert_eq!(vector("znn").symbols, ["znn", "ZNN"]);
        assert_eq!(vector("qsr").symbols, ["qsr", "QSR"]);
        assert!(vector("empty").symbols.is_empty());
    }
}

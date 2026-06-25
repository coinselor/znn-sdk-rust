//! Zenon `Token` and `TokenList` ledger models.
//!
//! A [`Token`] describes an on-chain token: its name, symbol, domain, supplies,
//! decimals, owner, token standard, and mint/burn/utility flags. Supplies are
//! stored as unscaled [`BigUint`] base-unit values and serialize to JSON as
//! decimal strings. Two tokens compare equal when their token standards match.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::token_standard::TokenStandard;
use num_bigint::BigUint;
use serde_json::{Map, Value, json};

/// A Zenon token.
#[derive(Debug, Clone, Eq)]
#[allow(clippy::struct_field_names)]
pub struct Token {
    name: String,
    symbol: String,
    domain: String,
    total_supply: BigUint,
    decimals: u8,
    owner: Address,
    token_standard: TokenStandard,
    max_supply: BigUint,
    is_burnable: bool,
    is_mintable: bool,
    is_utility: bool,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        // Equality is by token standard only.
        self.token_standard == other.token_standard
    }
}

impl Token {
    /// Creates a token from its fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        symbol: impl Into<String>,
        domain: impl Into<String>,
        total_supply: BigUint,
        decimals: u8,
        owner: Address,
        token_standard: TokenStandard,
        max_supply: BigUint,
        is_burnable: bool,
        is_mintable: bool,
        is_utility: bool,
    ) -> Self {
        Self {
            name: name.into(),
            symbol: symbol.into(),
            domain: domain.into(),
            total_supply,
            decimals,
            owner,
            token_standard,
            max_supply,
            is_burnable,
            is_mintable,
            is_utility,
        }
    }

    /// Returns the token name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the token symbol.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns the token domain.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the total supply as an unscaled base-unit value.
    pub fn total_supply(&self) -> &BigUint {
        &self.total_supply
    }

    /// Returns the number of decimals.
    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    /// Returns the owner address.
    pub fn owner(&self) -> &Address {
        &self.owner
    }

    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }

    /// Returns the max supply as an unscaled base-unit value.
    pub fn max_supply(&self) -> &BigUint {
        &self.max_supply
    }

    /// Returns `true` when the token is burnable.
    pub fn is_burnable(&self) -> bool {
        self.is_burnable
    }

    /// Returns `true` when the token is mintable.
    pub fn is_mintable(&self) -> bool {
        self.is_mintable
    }

    /// Returns `true` when the token is a utility token.
    pub fn is_utility(&self) -> bool {
        self.is_utility
    }

    /// Returns `10^decimals`.
    pub fn decimals_exponent(&self) -> u64 {
        10u64.saturating_pow(u32::from(self.decimals))
    }

    /// Serializes the token to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "symbol": self.symbol,
            "domain": self.domain,
            "totalSupply": self.total_supply.to_string(),
            "decimals": self.decimals,
            "owner": self.owner.to_string(),
            "tokenStandard": self.token_standard.to_string(),
            "maxSupply": self.max_supply.to_string(),
            "isBurnable": self.is_burnable,
            "isMintable": self.is_mintable,
            "isUtility": self.is_utility,
        })
    }

    /// Deserializes a token from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "token")?;
        let total_supply = required_big_uint(object, "totalSupply")?;
        let max_supply = required_big_uint(object, "maxSupply")?;
        let decimals = required_u8(object, "decimals")?;
        let owner = Address::parse(required_str(object, "owner")?)?;
        let token_standard = TokenStandard::parse(required_str(object, "tokenStandard")?)?;
        Ok(Self::new(
            required_str(object, "name")?,
            required_str(object, "symbol")?,
            required_str(object, "domain")?,
            total_supply,
            decimals,
            owner,
            token_standard,
            max_supply,
            required_bool(object, "isBurnable")?,
            required_bool(object, "isMintable")?,
            required_bool(object, "isUtility")?,
        ))
    }
}

/// A page of tokens.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenList {
    /// Page count, when present.
    pub count: Option<u64>,
    /// Page list, when present.
    pub list: Option<Vec<Token>>,
}

impl TokenList {
    /// Serializes the page to a JSON object.
    pub fn to_json(&self) -> Value {
        let mut object = Map::new();
        if let Some(count) = self.count {
            object.insert("count".to_string(), json!(count));
        }
        if let Some(list) = &self.list {
            object.insert(
                "list".to_string(),
                Value::Array(list.iter().map(Token::to_json).collect()),
            );
        }
        Value::Object(object)
    }

    /// Deserializes a page from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "token list")?;
        let count = optional_u64(object, "count")?;
        let list = match object.get("list") {
            Some(Value::Array(values)) => Some(
                values
                    .iter()
                    .map(Token::from_json)
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
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    struct TokenConformance {
        #[allow(dead_code)]
        description: String,
        token: Value,
        token_list: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/nom/token.json");

    fn conformance() -> TokenConformance {
        serde_json::from_str(CONFORMANCE).expect("valid token conformance")
    }

    fn token_value() -> Value {
        conformance().token
    }

    fn list_value() -> Value {
        conformance().token_list
    }

    fn sample_token() -> Token {
        Token::new(
            "Zenon Coin",
            "ZNN",
            "zenon.network",
            BigUint::from(19_500_000_000_000u64),
            8,
            Address::parse("z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg").unwrap(),
            TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap(),
            BigUint::from(4_611_686_018_427_387_903u64),
            true,
            true,
            true,
        )
    }

    #[test]
    fn new_and_accessors_return_each_field() {
        let token = Token::new(
            "Zenon Coin",
            "ZNN",
            "zenon.network",
            BigUint::from(19_500_000_000_000u64),
            8,
            Address::parse("z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg").unwrap(),
            TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap(),
            BigUint::from(4_611_686_018_427_387_903u64),
            true,
            true,
            true,
        );
        assert_eq!(token.name(), "Zenon Coin");
        assert_eq!(token.symbol(), "ZNN");
        assert_eq!(token.domain(), "zenon.network");
        assert_eq!(*token.total_supply(), BigUint::from(19_500_000_000_000u64));
        assert_eq!(token.decimals(), 8);
        assert_eq!(
            token.owner().to_string(),
            "z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg"
        );
        assert_eq!(
            token.token_standard().to_string(),
            "zts1znnxxxxxxxxxxxxx9z4ulx"
        );
        assert_eq!(
            *token.max_supply(),
            BigUint::from(4_611_686_018_427_387_903u64)
        );
        assert!(token.is_burnable());
        assert!(token.is_mintable());
        assert!(token.is_utility());
    }

    #[test]
    fn decimals_exponent_is_ten_pow_decimals() {
        let token = Token::new(
            "x",
            "x",
            "x",
            BigUint::from(0u32),
            8,
            Address::parse("z1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsggv2f").unwrap(),
            TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap(),
            BigUint::from(0u32),
            false,
            false,
            false,
        );
        assert_eq!(token.decimals_exponent(), 100_000_000);

        let zero = Token {
            decimals: 0,
            ..token
        };
        assert_eq!(zero.decimals_exponent(), 1);
    }

    #[test]
    fn to_json_matches_the_conformance_token() {
        let token = Token::from_json(&token_value()).expect("conformance token parses");
        assert_eq!(token.to_json(), token_value());
    }

    #[test]
    fn from_json_reads_the_conformance_fields() {
        let token = Token::from_json(&token_value()).expect("conformance token parses");
        assert_eq!(token.name(), "Zenon Coin");
        assert_eq!(token.symbol(), "ZNN");
        assert_eq!(token.decimals(), 8);
        assert_eq!(*token.total_supply(), BigUint::from(19_500_000_000_000u64));
        assert_eq!(
            *token.max_supply(),
            BigUint::from(4_611_686_018_427_387_903u64)
        );
        assert_eq!(
            token.token_standard().to_string(),
            "zts1znnxxxxxxxxxxxxx9z4ulx"
        );
        assert!(token.is_burnable() && token.is_mintable() && token.is_utility());
    }

    #[test]
    fn round_trips_through_to_json_and_from_json() {
        // Anchor on a constructor-built token so a broken serializer/parser
        // pair (e.g. `to_json` returning `{}` and `from_json` a placeholder)
        // cannot satisfy the round-trip: the re-parsed token must equal the
        // real token, not a placeholder.
        let original = sample_token();
        let round_trip = Token::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn rejects_a_malformed_token_object() {
        let mut missing = token_value();
        missing
            .as_object_mut()
            .expect("token is an object")
            .remove("tokenStandard");
        let result = Token::from_json(&missing);
        assert!(result.is_err(), "missing tokenStandard must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut bad_supply = token_value();
        bad_supply["totalSupply"] = json!("not-a-number");
        let result = Token::from_json(&bad_supply);
        assert!(result.is_err(), "non-decimal totalSupply must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn equal_when_token_standards_match_even_if_names_differ() {
        // Equality is by token standard only, so renaming a token must not
        // change the comparison result.
        let a = sample_token();
        let renamed = Token {
            name: "Other Coin".to_string(),
            ..sample_token()
        };
        assert_eq!(a, renamed, "same token standard must compare equal");
    }

    #[test]
    fn not_equal_when_token_standards_differ() {
        let a = sample_token();
        let other_standard = Token {
            token_standard: TokenStandard::parse("zts1qsrxxxxxxxxxxxxxmrhjll").unwrap(),
            ..sample_token()
        };
        assert_ne!(
            a, other_standard,
            "different token standards must not compare equal"
        );
    }

    #[test]
    fn equal_when_fully_identical() {
        assert_eq!(sample_token(), sample_token());
    }

    #[test]
    fn token_list_round_trips() {
        let list = TokenList::from_json(&list_value()).expect("conformance list parses");
        assert_eq!(list.count, Some(1), "count is decoded from input");
        assert_eq!(
            list.list.as_ref().map(Vec::len),
            Some(1),
            "list is decoded from input"
        );
        assert_eq!(list.to_json(), list_value());
    }
}

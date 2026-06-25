//! Liquidity contract models.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::TokenStandard;
use num_bigint::BigUint;
use serde_json::{Value, json};

/// A per-token reward tuple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenTuple {
    token_standard: TokenStandard,
    znn_percentage: u64,
    qsr_percentage: u64,
    min_amount: BigUint,
}

impl TokenTuple {
    /// Creates a token tuple.
    pub fn new(
        token_standard: TokenStandard,
        znn_percentage: u64,
        qsr_percentage: u64,
        min_amount: BigUint,
    ) -> Self {
        Self {
            token_standard,
            znn_percentage,
            qsr_percentage,
            min_amount,
        }
    }

    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }
    /// Returns the znn percentage.
    pub fn znn_percentage(&self) -> u64 {
        self.znn_percentage
    }
    /// Returns the qsr percentage.
    pub fn qsr_percentage(&self) -> u64 {
        self.qsr_percentage
    }
    /// Returns the min amount.
    pub fn min_amount(&self) -> &BigUint {
        &self.min_amount
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "tokenStandard": self.token_standard.to_string(),
            "znnPercentage": self.znn_percentage,
            "qsrPercentage": self.qsr_percentage,
            "minAmount": self.min_amount.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "token tuple")?;
        Ok(Self::new(
            TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            required_u64(object, "znnPercentage")?,
            required_u64(object, "qsrPercentage")?,
            required_big_uint(object, "minAmount")?,
        ))
    }
}

/// Liquidity contract administrator info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiquidityInfo {
    administrator: Address,
    is_halted: bool,
    znn_reward: BigUint,
    qsr_reward: BigUint,
    token_tuples: Vec<TokenTuple>,
}

impl LiquidityInfo {
    /// Creates liquidity info.
    pub fn new(
        administrator: Address,
        is_halted: bool,
        znn_reward: BigUint,
        qsr_reward: BigUint,
        token_tuples: Vec<TokenTuple>,
    ) -> Self {
        Self {
            administrator,
            is_halted,
            znn_reward,
            qsr_reward,
            token_tuples,
        }
    }

    /// Returns the administrator.
    pub fn administrator(&self) -> &Address {
        &self.administrator
    }
    /// Returns whether halted.
    pub fn is_halted(&self) -> bool {
        self.is_halted
    }
    /// Returns the znn reward.
    pub fn znn_reward(&self) -> &BigUint {
        &self.znn_reward
    }
    /// Returns the qsr reward.
    pub fn qsr_reward(&self) -> &BigUint {
        &self.qsr_reward
    }
    /// Returns the token tuples.
    pub fn token_tuples(&self) -> &[TokenTuple] {
        &self.token_tuples
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "administrator": self.administrator.to_string(),
            "isHalted": self.is_halted,
            "znnReward": self.znn_reward.to_string(),
            "qsrReward": self.qsr_reward.to_string(),
            "tokenTuples": self.token_tuples.iter().map(TokenTuple::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "liquidity info")?;
        Ok(Self::new(
            Address::parse(required_str(object, "administrator")?)?,
            required_bool(object, "isHalted")?,
            required_big_uint(object, "znnReward")?,
            required_big_uint(object, "qsrReward")?,
            required_array(object, "tokenTuples", TokenTuple::from_json)?,
        ))
    }
}

/// A liquidity stake entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiquidityStakeEntry {
    amount: BigUint,
    token_standard: TokenStandard,
    weighted_amount: BigUint,
    start_time: u64,
    revoke_time: u64,
    expiration_time: u64,
    stake_address: Address,
    id: Hash,
}

impl LiquidityStakeEntry {
    /// Creates a liquidity stake entry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        amount: BigUint,
        token_standard: TokenStandard,
        weighted_amount: BigUint,
        start_time: u64,
        revoke_time: u64,
        expiration_time: u64,
        stake_address: Address,
        id: Hash,
    ) -> Self {
        Self {
            amount,
            token_standard,
            weighted_amount,
            start_time,
            revoke_time,
            expiration_time,
            stake_address,
            id,
        }
    }

    /// Returns the amount.
    pub fn amount(&self) -> &BigUint {
        &self.amount
    }
    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }
    /// Returns the weighted amount.
    pub fn weighted_amount(&self) -> &BigUint {
        &self.weighted_amount
    }
    /// Returns the start time.
    pub fn start_time(&self) -> u64 {
        self.start_time
    }
    /// Returns the revoke time.
    pub fn revoke_time(&self) -> u64 {
        self.revoke_time
    }
    /// Returns the expiration time.
    pub fn expiration_time(&self) -> u64 {
        self.expiration_time
    }
    /// Returns the stake address.
    pub fn stake_address(&self) -> &Address {
        &self.stake_address
    }
    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "amount": self.amount.to_string(),
            "tokenStandard": self.token_standard.to_string(),
            "weightedAmount": self.weighted_amount.to_string(),
            "startTime": self.start_time,
            "revokeTime": self.revoke_time,
            "expirationTime": self.expiration_time,
            "stakeAddress": self.stake_address.to_string(),
            "id": self.id.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "liquidity stake entry")?;
        Ok(Self::new(
            required_big_uint(object, "amount")?,
            TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            required_big_uint(object, "weightedAmount")?,
            required_u64(object, "startTime")?,
            required_u64(object, "revokeTime")?,
            required_u64(object, "expirationTime")?,
            Address::parse(required_str(object, "stakeAddress")?)?,
            Hash::parse(required_str(object, "id")?)?,
        ))
    }
}

/// A paged list of liquidity stake entries with totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiquidityStakeList {
    total_amount: BigUint,
    total_weighted_amount: BigUint,
    count: u64,
    list: Vec<LiquidityStakeEntry>,
}

impl LiquidityStakeList {
    /// Creates a liquidity stake list.
    pub fn new(
        total_amount: BigUint,
        total_weighted_amount: BigUint,
        count: u64,
        list: Vec<LiquidityStakeEntry>,
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
    pub fn list(&self) -> &[LiquidityStakeEntry] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "totalAmount": self.total_amount.to_string(),
            "totalWeightedAmount": self.total_weighted_amount.to_string(),
            "count": self.count,
            "list": self.list.iter().map(LiquidityStakeEntry::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "liquidity stake list")?;
        Ok(Self::new(
            required_big_uint(object, "totalAmount")?,
            required_big_uint(object, "totalWeightedAmount")?,
            required_u64(object, "count")?,
            required_array(object, "list", LiquidityStakeEntry::from_json)?,
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
        token_tuple: Value,
        liquidity_info: Value,
        liquidity_stake_entry: Value,
        liquidity_stake_list: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/liquidity.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid liquidity conformance")
    }

    #[test]
    fn token_tuple_round_trip() {
        let original = TokenTuple::new(
            TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap(),
            50,
            50,
            BigUint::from(1_000_000_000u64),
        );
        let round_trip = TokenTuple::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn liquidity_info_round_trip() {
        let value = conf().liquidity_info;
        let info = LiquidityInfo::from_json(&value).expect("conformance parses");
        assert_eq!(info.to_json(), value);
        assert_eq!(info.token_tuples().len(), 1);
    }

    #[test]
    fn liquidity_info_rejects_non_array_token_tuples() {
        let mut bad = conf().liquidity_info;
        bad["tokenTuples"] = json!("not-an-array");
        let result = LiquidityInfo::from_json(&bad);
        assert!(result.is_err(), "non-array tokenTuples must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn liquidity_stake_entry_round_trip() {
        let original = LiquidityStakeEntry::new(
            BigUint::from(5_000_000_000u64),
            TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap(),
            BigUint::from(7_500_000_000u64),
            1_700_000_000,
            0,
            1_730_000_000,
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            Hash::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
        );
        let round_trip =
            LiquidityStakeEntry::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn liquidity_stake_list_round_trip() {
        let value = conf().liquidity_stake_list;
        let list = LiquidityStakeList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
        assert_eq!(list.count(), 1);
        assert_eq!(list.list().len(), 1);
    }
}

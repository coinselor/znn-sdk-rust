//! Zenon `AccountInfo` and `BalanceInfoListItem` ledger models.
//!
//! [`AccountInfo`] holds an account's address (an opaque string), its account
//! height, and a per-token balance list. The JSON form carries the balances as
//! a map keyed by token standard (`balanceInfoMap`).

use crate::error::Error;
use crate::model::json::*;
use crate::model::nom::token::Token;
use crate::primitives::token_standard::{TokenStandard, qsr_token_standard, znn_token_standard};
use num_bigint::BigUint;
use serde_json::{Map, Value, json};

/// A balance entry: an optional [`Token`] and its balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceInfoListItem {
    token: Option<Token>,
    balance: BigUint,
}

impl BalanceInfoListItem {
    /// Creates a balance entry from an optional token and a balance.
    pub fn new(token: Option<Token>, balance: BigUint) -> Self {
        Self { token, balance }
    }

    /// Returns the token, when present.
    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Returns the balance.
    pub fn balance(&self) -> &BigUint {
        &self.balance
    }

    /// Serializes the entry to a JSON object.
    pub fn to_json(&self) -> Value {
        let mut object = Map::new();
        if let Some(token) = &self.token {
            object.insert("token".to_string(), token.to_json());
        }
        object.insert("balance".to_string(), json!(self.balance.to_string()));
        Value::Object(object)
    }

    /// Deserializes an entry from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "balance info list item")?;
        let token = match object.get("token") {
            Some(Value::Null) | None => None,
            Some(value) => Some(Token::from_json(value)?),
        };
        Ok(Self {
            token,
            balance: required_big_uint(object, "balance")?,
        })
    }
}

/// An account's ledger state.
#[derive(Debug, Clone)]
pub struct AccountInfo {
    address: String,
    block_count: u64,
    balance_info_list: Vec<BalanceInfoListItem>,
}

impl PartialEq for AccountInfo {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
            && self.block_count == other.block_count
            && balance_lists_equal(&self.balance_info_list, &other.balance_info_list)
    }
}

impl Eq for AccountInfo {}

impl AccountInfo {
    /// Creates an account info from an address string, a block count, and a
    /// balance list.
    pub fn new(
        address: String,
        block_count: u64,
        balance_info_list: Vec<BalanceInfoListItem>,
    ) -> Self {
        Self {
            address,
            block_count,
            balance_info_list,
        }
    }

    /// Returns the address string.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns the account height.
    pub fn block_count(&self) -> u64 {
        self.block_count
    }

    /// Returns the balance list.
    pub fn balance_info_list(&self) -> &[BalanceInfoListItem] {
        &self.balance_info_list
    }

    /// Returns the balance of the given token standard, or zero when absent.
    pub fn get_balance(&self, token_standard: &TokenStandard) -> BigUint {
        self.balance_info_list
            .iter()
            .find(|item| {
                item.token()
                    .is_some_and(|token| token.token_standard() == token_standard)
            })
            .map(|item| item.balance.clone())
            .unwrap_or_default()
    }

    /// Returns the ZNN balance.
    pub fn znn(&self) -> BigUint {
        self.get_balance(&znn_token_standard())
    }

    /// Returns the QSR balance.
    pub fn qsr(&self) -> BigUint {
        self.get_balance(&qsr_token_standard())
    }

    /// Returns the token for the given standard, when present.
    pub fn find_token_by_token_standard(&self, token_standard: &TokenStandard) -> Option<&Token> {
        self.balance_info_list
            .iter()
            .filter_map(BalanceInfoListItem::token)
            .find(|token| token.token_standard() == token_standard)
    }

    /// Serializes the account to a JSON object.
    pub fn to_json(&self) -> Value {
        let mut balances = Map::new();
        for item in &self.balance_info_list {
            let key = balance_info_key(item, &balances);
            balances.insert(key, item.to_json());
        }

        json!({
            "address": self.address,
            "accountHeight": self.block_count,
            "balanceInfoMap": Value::Object(balances),
        })
    }

    /// Deserializes an account from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "account info")?;
        let address = required_str(object, "address")?.to_string();
        let block_count = required_u64(object, "accountHeight")?;
        let balance_map = required_value(object, "balanceInfoMap")?
            .as_object()
            .ok_or_else(|| Error::InvalidInput("balanceInfoMap must be an object".into()))?;
        let balance_info_list = if block_count == 0 {
            Vec::new()
        } else {
            balance_map
                .values()
                .map(BalanceInfoListItem::from_json)
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(Self {
            address,
            block_count,
            balance_info_list,
        })
    }
}

fn balance_lists_equal(left: &[BalanceInfoListItem], right: &[BalanceInfoListItem]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut unmatched = right.iter().collect::<Vec<_>>();
    for item in left {
        let Some(position) = unmatched.iter().position(|candidate| **candidate == *item) else {
            return false;
        };
        unmatched.remove(position);
    }
    true
}

fn balance_info_key(item: &BalanceInfoListItem, balances: &Map<String, Value>) -> String {
    let base = item.token().map_or_else(
        || "__missing_token".to_string(),
        |token| token.token_standard().to_string(),
    );
    if !balances.contains_key(&base) {
        return base;
    }

    let mut suffix = 1u64;
    loop {
        let candidate = format!("{base}#{suffix}");
        if !balances.contains_key(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    struct AccountInfoConformance {
        #[allow(dead_code)]
        description: String,
        account_info: Value,
        empty_account_info: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/nom/account_info.json");

    fn conformance() -> AccountInfoConformance {
        serde_json::from_str(CONFORMANCE).expect("valid account info conformance")
    }

    fn account_value() -> Value {
        conformance().account_info
    }

    fn empty_value() -> Value {
        conformance().empty_account_info
    }

    fn znn_token() -> Token {
        Token::from_json(&account_value()["balanceInfoMap"]["zts1znnxxxxxxxxxxxxx9z4ulx"]["token"])
            .expect("conformance token parses")
    }

    fn sample_item() -> BalanceInfoListItem {
        BalanceInfoListItem::new(Some(znn_token()), BigUint::from(1_000_000_000u64))
    }

    fn qsr_item() -> BalanceInfoListItem {
        let token = znn_token();
        BalanceInfoListItem::new(
            Some(Token::new(
                token.name(),
                token.symbol(),
                token.domain(),
                token.total_supply().clone(),
                token.decimals(),
                token.owner().clone(),
                qsr_token_standard(),
                token.max_supply().clone(),
                token.is_burnable(),
                token.is_mintable(),
                token.is_utility(),
            )),
            BigUint::from(2_000_000_000u64),
        )
    }

    fn sample_account() -> AccountInfo {
        AccountInfo::new(
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz".to_string(),
            2,
            vec![sample_item()],
        )
    }

    #[test]
    fn balance_item_new_and_accessors_return_each_field() {
        let item = sample_item();
        assert!(
            item.token().is_some(),
            "token accessor returns the provided token"
        );
        assert_eq!(*item.balance(), BigUint::from(1_000_000_000u64));
    }

    #[test]
    fn balance_item_round_trips() {
        let item = sample_item();
        let round_trip =
            BalanceInfoListItem::from_json(&item.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, item);
    }

    #[test]
    fn account_new_and_accessors_return_each_field() {
        let account = sample_account();
        assert_eq!(
            account.address(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(account.block_count(), 2);
        assert_eq!(account.balance_info_list().len(), 1);
    }

    #[test]
    fn to_json_matches_the_conformance_account() {
        let account = AccountInfo::from_json(&account_value()).expect("conformance account parses");
        assert_eq!(account.to_json(), account_value());
    }

    #[test]
    fn from_json_reads_the_conformance_fields() {
        let account = AccountInfo::from_json(&account_value()).expect("conformance account parses");
        assert_eq!(
            account.address(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(account.block_count(), 2);
        assert_eq!(account.balance_info_list().len(), 1);
    }

    #[test]
    fn round_trips_through_to_json_and_from_json() {
        // Anchor on a constructor-built account so a broken serializer/parser
        // pair cannot satisfy the round-trip: the re-parsed account must keep
        // the real address, block count, and balance, not the placeholder.
        let original = sample_account();
        let round_trip = AccountInfo::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn tokenless_balance_items_are_not_dropped_on_account_serialization() {
        let original = AccountInfo::new(
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz".to_string(),
            1,
            vec![BalanceInfoListItem::new(None, BigUint::from(42u32))],
        );
        let serialized = original.to_json();
        let balances = serialized["balanceInfoMap"]
            .as_object()
            .expect("balanceInfoMap is an object");
        assert_eq!(balances.len(), 1, "tokenless item must be serialized");

        let round_trip = AccountInfo::from_json(&serialized).expect("round-trip parses");
        assert_eq!(round_trip, original);
        assert_eq!(round_trip.balance_info_list().len(), 1);
        assert!(round_trip.balance_info_list()[0].token().is_none());
        assert_eq!(
            *round_trip.balance_info_list()[0].balance(),
            BigUint::from(42u32)
        );
    }

    #[test]
    fn account_info_equality_is_stable_across_balance_map_ordering() {
        let znn = sample_item();
        let qsr = qsr_item();
        let original = AccountInfo::new(
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz".to_string(),
            2,
            vec![znn.clone(), qsr.clone()],
        );
        let reversed = AccountInfo::new(
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz".to_string(),
            2,
            vec![qsr, znn],
        );

        assert_eq!(original, reversed);
        let round_trip = AccountInfo::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn get_balance_returns_a_present_token_balance() {
        let account = AccountInfo::from_json(&account_value()).expect("conformance account parses");
        let znn = znn_token_standard();
        assert_eq!(account.get_balance(&znn), BigUint::from(1_000_000_000u64));
    }

    #[test]
    fn get_balance_returns_zero_when_token_absent() {
        let account = AccountInfo::from_json(&account_value()).expect("conformance account parses");
        let qsr = qsr_token_standard();
        assert_eq!(account.get_balance(&qsr), BigUint::from(0u32));
    }

    #[test]
    fn znn_and_qsr_return_each_present_balance() {
        // Build a two-entry account (ZNN present from conformance, QSR derived
        // by reusing the ZNN token JSON with the QSR token standard) so both
        // accessors resolve a non-zero balance.
        let znn_entry = account_value()["balanceInfoMap"]["zts1znnxxxxxxxxxxxxx9z4ulx"].clone();
        let mut qsr_entry = znn_entry.clone();
        qsr_entry["token"]["tokenStandard"] = json!("zts1qsrxxxxxxxxxxxxxmrhjll");
        qsr_entry["balance"] = json!("2000000000");

        let mut two_entry = account_value();
        two_entry["balanceInfoMap"]["zts1qsrxxxxxxxxxxxxxmrhjll"] = qsr_entry;

        let account = AccountInfo::from_json(&two_entry).expect("two-entry account parses");
        assert_eq!(account.znn(), BigUint::from(1_000_000_000u64));
        assert_eq!(account.qsr(), BigUint::from(2_000_000_000u64));
    }

    #[test]
    fn find_token_returns_the_token_when_present() {
        let account = AccountInfo::from_json(&account_value()).expect("conformance account parses");
        let znn = znn_token_standard();
        let token = account
            .find_token_by_token_standard(&znn)
            .expect("ZNN token present");
        assert_eq!(
            token.token_standard().to_string(),
            "zts1znnxxxxxxxxxxxxx9z4ulx"
        );
    }

    #[test]
    fn find_token_returns_none_when_absent() {
        let account = AccountInfo::from_json(&account_value()).expect("conformance account parses");
        let qsr = qsr_token_standard();
        assert!(account.find_token_by_token_standard(&qsr).is_none());
    }

    #[test]
    fn empty_account_has_no_balances() {
        let account = AccountInfo::from_json(&empty_value()).expect("empty account parses");
        assert_eq!(account.block_count(), 0);
        assert!(account.balance_info_list().is_empty());
    }

    #[test]
    fn rejects_a_malformed_account_object() {
        let mut missing = account_value();
        missing
            .as_object_mut()
            .expect("account is an object")
            .remove("accountHeight");
        let result = AccountInfo::from_json(&missing);
        assert!(result.is_err(), "missing accountHeight must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut bad_balance = account_value();
        bad_balance["balanceInfoMap"]["zts1znnxxxxxxxxxxxxx9z4ulx"]["balance"] =
            json!("not-a-number");
        let result = AccountInfo::from_json(&bad_balance);
        assert!(result.is_err(), "non-decimal balance must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn equal_when_all_fields_match() {
        assert_eq!(sample_account(), sample_account());
    }

    #[test]
    fn not_equal_when_block_counts_differ() {
        let other = AccountInfo {
            block_count: 3,
            ..sample_account()
        };
        assert_ne!(sample_account(), other);
    }
}

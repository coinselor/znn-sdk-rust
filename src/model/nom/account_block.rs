//! Zenon `AccountBlock`, `AccountBlockConfirmationDetail`, and `AccountBlockList`
//! ledger models.
//!
//! [`AccountBlock`] composes an [`AccountBlockTemplate`] with confirmation and
//! plasma data. Its JSON form merges the template fields with the account-block
//! fields, carrying inherited byte fields as base64.

use crate::error::Error;
use crate::model::json::*;
use crate::model::nom::account_block_template::AccountBlockTemplate;
#[cfg(test)]
use crate::model::nom::account_block_template::BlockType;
use crate::model::nom::token::Token;
use crate::primitives::hash::Hash;
use serde_json::{Map, Value, json};

/// Confirmation metadata for a confirmed account block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountBlockConfirmationDetail {
    num_confirmations: u64,
    momentum_height: u64,
    momentum_hash: Hash,
    momentum_timestamp: u64,
}

impl AccountBlockConfirmationDetail {
    /// Creates a confirmation detail from its fields.
    pub fn new(
        num_confirmations: u64,
        momentum_height: u64,
        momentum_hash: Hash,
        momentum_timestamp: u64,
    ) -> Self {
        Self {
            num_confirmations,
            momentum_height,
            momentum_hash,
            momentum_timestamp,
        }
    }

    /// Returns the number of confirmations.
    pub fn num_confirmations(&self) -> u64 {
        self.num_confirmations
    }

    /// Returns the confirming momentum height.
    pub fn momentum_height(&self) -> u64 {
        self.momentum_height
    }

    /// Returns the confirming momentum hash.
    pub fn momentum_hash(&self) -> &Hash {
        &self.momentum_hash
    }

    /// Returns the confirming momentum timestamp.
    pub fn momentum_timestamp(&self) -> u64 {
        self.momentum_timestamp
    }

    /// Serializes the detail to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "numConfirmations": self.num_confirmations,
            "momentumHeight": self.momentum_height,
            "momentumHash": self.momentum_hash.to_string(),
            "momentumTimestamp": self.momentum_timestamp,
        })
    }

    /// Deserializes a detail from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "account block confirmation detail")?;
        Ok(Self {
            num_confirmations: required_u64(object, "numConfirmations")?,
            momentum_height: required_u64(object, "momentumHeight")?,
            momentum_hash: Hash::parse(required_str(object, "momentumHash")?)?,
            momentum_timestamp: required_u64(object, "momentumTimestamp")?,
        })
    }
}

/// A confirmed or confirmable account block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct AccountBlock {
    template: AccountBlockTemplate,
    descendant_blocks: Vec<AccountBlock>,
    base_plasma: u32,
    used_plasma: u32,
    changes_hash: Hash,
    token: Option<Token>,
    confirmation_detail: Option<AccountBlockConfirmationDetail>,
    paired_account_block: Option<Box<AccountBlock>>,
}

impl AccountBlock {
    /// Creates an account block from its template and account-block fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        template: AccountBlockTemplate,
        descendant_blocks: Vec<AccountBlock>,
        base_plasma: u32,
        used_plasma: u32,
        changes_hash: Hash,
        token: Option<Token>,
        confirmation_detail: Option<AccountBlockConfirmationDetail>,
        paired_account_block: Option<Box<AccountBlock>>,
    ) -> Self {
        Self {
            template,
            descendant_blocks,
            base_plasma,
            used_plasma,
            changes_hash,
            token,
            confirmation_detail,
            paired_account_block,
        }
    }

    /// Returns the embedded template (the inherited account-block fields).
    pub fn template(&self) -> &AccountBlockTemplate {
        &self.template
    }

    /// Returns the descendant blocks.
    pub fn descendant_blocks(&self) -> &[AccountBlock] {
        &self.descendant_blocks
    }

    /// Returns the base plasma.
    pub fn base_plasma(&self) -> u32 {
        self.base_plasma
    }

    /// Returns the used plasma.
    pub fn used_plasma(&self) -> u32 {
        self.used_plasma
    }

    /// Returns the changes hash.
    pub fn changes_hash(&self) -> &Hash {
        &self.changes_hash
    }

    /// Returns the resolved token, when present.
    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Returns the confirmation detail, when present.
    pub fn confirmation_detail(&self) -> Option<&AccountBlockConfirmationDetail> {
        self.confirmation_detail.as_ref()
    }

    /// Returns the paired account block, when present.
    pub fn paired_account_block(&self) -> Option<&AccountBlock> {
        self.paired_account_block.as_deref()
    }

    /// Returns `true` when the block carries a confirmation detail.
    pub fn is_completed(&self) -> bool {
        self.confirmation_detail.is_some()
    }

    /// Serializes the account block to a JSON object.
    pub fn to_json(&self) -> Value {
        let mut object = match self.template.to_json() {
            Value::Object(object) => object,
            _ => Map::new(),
        };
        object.insert(
            "descendantBlocks".to_string(),
            Value::Array(
                self.descendant_blocks
                    .iter()
                    .map(AccountBlock::to_json)
                    .collect(),
            ),
        );
        object.insert("basePlasma".to_string(), json!(self.base_plasma));
        object.insert("usedPlasma".to_string(), json!(self.used_plasma));
        object.insert(
            "changesHash".to_string(),
            json!(self.changes_hash.to_string()),
        );
        object.insert(
            "token".to_string(),
            self.token.as_ref().map_or(Value::Null, Token::to_json),
        );
        object.insert(
            "confirmationDetail".to_string(),
            self.confirmation_detail
                .as_ref()
                .map_or(Value::Null, AccountBlockConfirmationDetail::to_json),
        );
        object.insert(
            "pairedAccountBlock".to_string(),
            self.paired_account_block
                .as_ref()
                .map_or(Value::Null, |block| block.to_json()),
        );
        Value::Object(object)
    }

    /// Deserializes an account block from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "account block")?;
        let template = AccountBlockTemplate::from_json(value)?;
        let descendant_blocks = required_array_ref(object, "descendantBlocks")?
            .iter()
            .map(AccountBlock::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        let token = optional_token(object, "token")?;
        let confirmation_detail = optional_confirmation_detail(object, "confirmationDetail")?;
        let paired_account_block = optional_paired_account_block(object, "pairedAccountBlock")?;

        Ok(Self::new(
            template,
            descendant_blocks,
            required_u32(object, "basePlasma")?,
            required_u32(object, "usedPlasma")?,
            Hash::parse(required_str(object, "changesHash")?)?,
            token,
            confirmation_detail,
            paired_account_block,
        ))
    }

    #[cfg(test)]
    fn placeholder() -> Self {
        Self {
            template: AccountBlockTemplate::new(BlockType::Unknown),
            descendant_blocks: Vec::new(),
            base_plasma: 0,
            used_plasma: 0,
            changes_hash: Hash::empty(),
            token: None,
            confirmation_detail: None,
            paired_account_block: None,
        }
    }
}

/// A page of account blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountBlockList {
    /// Page count, when present.
    pub count: Option<u64>,
    /// Page list, when present.
    pub list: Option<Vec<AccountBlock>>,
    /// Whether more blocks exist beyond `count`.
    pub more: Option<bool>,
}

impl AccountBlockList {
    /// Serializes the page to a JSON object.
    pub fn to_json(&self) -> Value {
        let mut object = Map::new();
        if let Some(count) = self.count {
            object.insert("count".to_string(), json!(count));
        }
        if let Some(list) = &self.list {
            object.insert(
                "list".to_string(),
                Value::Array(list.iter().map(AccountBlock::to_json).collect()),
            );
        }
        if let Some(more) = self.more {
            object.insert("more".to_string(), json!(more));
        }
        Value::Object(object)
    }

    /// Deserializes a page from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "account block list")?;
        let count = optional_u64(object, "count")?;
        let list = match object.get("list") {
            Some(Value::Array(values)) => Some(
                values
                    .iter()
                    .map(AccountBlock::from_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Some(_) => return Err(Error::InvalidInput("list must be an array".into())),
            None => None,
        };
        let more = optional_bool(object, "more")?;
        Ok(Self { count, list, more })
    }
}

fn optional_token(object: &Map<String, Value>, field: &str) -> Result<Option<Token>, Error> {
    match required_value(object, field)? {
        Value::Null => Ok(None),
        value => Token::from_json(value).map(Some),
    }
}

fn optional_confirmation_detail(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<AccountBlockConfirmationDetail>, Error> {
    match required_value(object, field)? {
        Value::Null => Ok(None),
        value => AccountBlockConfirmationDetail::from_json(value).map(Some),
    }
}

fn optional_paired_account_block(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<Box<AccountBlock>>, Error> {
    match required_value(object, field)? {
        Value::Null => Ok(None),
        value => AccountBlock::from_json(value).map(Box::new).map(Some),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::primitives::address::Address;
    use crate::primitives::token_standard::TokenStandard;
    use num_bigint::BigUint;
    use serde_json::json;

    #[test]
    fn confirmation_detail_new_and_accessors_return_each_field() {
        let detail = AccountBlockConfirmationDetail::new(
            2,
            3,
            Hash::parse("0f92b0be5eef439be78f9d48add78288391d6723e40c7059fae0f1241a9e639f")
                .unwrap(),
            1_000_000_010,
        );
        assert_eq!(detail.num_confirmations(), 2);
        assert_eq!(detail.momentum_height(), 3);
        assert_eq!(detail.momentum_timestamp(), 1_000_000_010);
        assert_eq!(
            detail.momentum_hash().to_string(),
            "0f92b0be5eef439be78f9d48add78288391d6723e40c7059fae0f1241a9e639f"
        );
    }

    #[test]
    fn confirmation_detail_round_trips() {
        let detail = AccountBlockConfirmationDetail::new(
            2,
            3,
            Hash::parse("0f92b0be5eef439be78f9d48add78288391d6723e40c7059fae0f1241a9e639f")
                .unwrap(),
            1_000_000_010,
        );
        let round_trip = AccountBlockConfirmationDetail::from_json(&detail.to_json())
            .expect("round-trip parses");
        assert_eq!(round_trip, detail);
    }

    #[test]
    fn account_block_list_round_trips() {
        let value = json!({ "count": 0, "list": [], "more": false });
        let list = AccountBlockList::from_json(&value).expect("list parses");
        assert_eq!(list.count, Some(0), "count is decoded from input");
        assert_eq!(
            list.list.as_ref().map(Vec::len),
            Some(0),
            "list is decoded from input"
        );
        assert_eq!(list.more, Some(false), "more is decoded from input");
        assert_eq!(list.to_json(), value);
    }

    #[test]
    fn not_equal_when_used_plasma_differs() {
        let a = AccountBlock {
            used_plasma: 21_000,
            ..AccountBlock::placeholder()
        };
        let b = AccountBlock {
            used_plasma: 22_000,
            ..AccountBlock::placeholder()
        };
        assert_ne!(a, b);
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
    fn rejects_a_malformed_confirmation_detail() {
        let value = json!({
            "numConfirmations": 2,
            "momentumHeight": 3,
            "momentumHash": "0f92b0be5eef439be78f9d48add78288391d6723e40c7059fae0f1241a9e639f",
            "momentumTimestamp": 1_000_000_010
        });
        let mut missing = value.clone();
        missing
            .as_object_mut()
            .expect("detail is an object")
            .remove("momentumHash");
        let result = AccountBlockConfirmationDetail::from_json(&missing);
        assert!(result.is_err(), "missing momentumHash must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut bad_hash = value.clone();
        bad_hash["momentumHash"] = json!("not-a-hash");
        let result = AccountBlockConfirmationDetail::from_json(&bad_hash);
        assert!(
            result.is_err(),
            "non-canonical momentumHash must be rejected"
        );
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn account_block_new_and_accessors_return_each_field() {
        let template = AccountBlockTemplate::send(
            Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap(),
            TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap(),
            BigUint::from(100u64),
            None,
        );
        let changes_hash =
            Hash::parse("a31a31bb26f7a7ee5b5c8e83e6b47aeeab6e2330476199d93ee8ca37ac71465a")
                .unwrap();
        let token = sample_token();
        let confirmation = AccountBlockConfirmationDetail::new(
            2,
            3,
            Hash::parse("0f92b0be5eef439be78f9d48add78288391d6723e40c7059fae0f1241a9e639f")
                .unwrap(),
            1_000_000_010,
        );
        let paired = AccountBlock::placeholder();
        let block = AccountBlock::new(
            template.clone(),
            Vec::new(),
            21_000,
            22_000,
            changes_hash.clone(),
            Some(token.clone()),
            Some(confirmation.clone()),
            Some(Box::new(paired.clone())),
        );
        assert_eq!(block.template(), &template, "template accessor");
        assert!(
            block.descendant_blocks().is_empty(),
            "descendant_blocks accessor"
        );
        assert_eq!(block.base_plasma(), 21_000, "base_plasma accessor");
        assert_eq!(block.used_plasma(), 22_000, "used_plasma accessor");
        assert_eq!(*block.changes_hash(), changes_hash, "changes_hash accessor");
        assert_eq!(block.token(), Some(&token), "token accessor");
        assert_eq!(
            block.confirmation_detail(),
            Some(&confirmation),
            "confirmation_detail accessor"
        );
        assert_eq!(
            block.paired_account_block(),
            Some(&paired),
            "paired_account_block accessor"
        );
        assert!(
            block.is_completed(),
            "a block carrying a confirmation detail is completed"
        );
    }
}

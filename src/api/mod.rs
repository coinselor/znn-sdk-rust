//! JSON-RPC API wrappers.

pub mod embedded;
pub mod ledger;
pub mod stats;
pub mod subscribe;

use crate::error::Error;
use crate::primitives::address::Address;
use num_bigint::BigUint;
use serde_json::Value;

/// A page index/size pair carried by paginated JSON-RPC getters.
///
/// Uses the JSON-RPC `pageIndex`/`pageSize` convention: the [`Default`] is
/// `(0, RPC_MAX_PAGE_SIZE)` and [`PageQuery::mempool`] is `(0, MEMORY_POOL_PAGE_SIZE)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageQuery {
    /// Zero-based page index.
    pub index: u32,
    /// Page size.
    pub size: u32,
}

impl PageQuery {
    /// Returns the memory-pool page query `(0, MEMORY_POOL_PAGE_SIZE)`.
    pub fn mempool() -> Self {
        Self {
            index: 0,
            size: crate::client::constants::MEMORY_POOL_PAGE_SIZE,
        }
    }
}

impl Default for PageQuery {
    fn default() -> Self {
        Self {
            index: 0,
            size: crate::client::constants::RPC_MAX_PAGE_SIZE,
        }
    }
}

/// Decodes `value` into `Option<T>`: `None` when the node returned `null`.
pub(crate) fn optional<T, F>(value: &Value, decode: F) -> Result<Option<T>, Error>
where
    F: FnOnce(&Value) -> Result<T, Error>,
{
    if value.is_null() {
        Ok(None)
    } else {
        decode(value).map(Some)
    }
}

/// Parses a JSON value as a decimal string into a [`BigUint`].
pub(crate) fn big_uint_from_value(value: &Value, field: &str) -> Result<BigUint, Error> {
    let text = value
        .as_str()
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be a decimal string")))?;
    BigUint::parse_bytes(text.as_bytes(), 10)
        .ok_or_else(|| Error::InvalidInput(format!("{field} is not a valid decimal number")))
}

/// Parses a JSON value as a boolean.
pub(crate) fn bool_from_value(value: &Value, field: &str) -> Result<bool, Error> {
    value
        .as_bool()
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be a boolean")))
}

/// Parses a JSON value as a 64-bit unsigned integer.
pub(crate) fn u64_from_value(value: &Value, field: &str) -> Result<u64, Error> {
    value
        .as_u64()
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be an unsigned integer")))
}

/// Serializes a page index/size pair as positional JSON params.
pub(crate) fn page_params(page: PageQuery) -> Vec<Value> {
    vec![Value::from(page.index), Value::from(page.size)]
}

/// Serializes an address plus a page index/size pair as positional JSON params.
pub(crate) fn address_page_params(address: &Address, page: PageQuery) -> Vec<Value> {
    vec![
        Value::from(address.to_string()),
        Value::from(page.index),
        Value::from(page.size),
    ]
}

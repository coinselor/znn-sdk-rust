//! Embedded contract APIs (`embedded.*` JSON-RPC methods).
//!
//! Each contract API wraps a shared [`Client`] and exposes read methods that
//! dispatch `embedded.*` calls and decode the JSON responses, plus builder
//! methods that return
//! [`AccountBlockTemplate`](crate::model::nom::account_block_template::AccountBlockTemplate)s
//! targeting the contract via the ABI codec.

use crate::abi::{Abi, AbiType, AbiValue, encode_arguments, selector};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::error::Error;
use crate::primitives::address::Address;
use serde_json::Value;
use std::sync::Arc;

pub mod accelerator;
pub mod bridge;
pub mod htlc;
pub mod liquidity;
pub mod pillar;
pub mod plasma;
pub mod sentinel;
pub mod spork;
pub mod stake;
pub mod swap;
pub mod token;

/// Aggregates all embedded contract API roots over one shared client.
pub struct EmbeddedApi<C: Client = WsClient> {
    /// Accelerator contract API.
    pub accelerator: accelerator::AcceleratorApi<C>,
    /// Bridge contract API.
    pub bridge: bridge::BridgeApi<C>,
    /// HTLC contract API.
    pub htlc: htlc::HtlcApi<C>,
    /// Liquidity contract API.
    pub liquidity: liquidity::LiquidityApi<C>,
    /// Pillar contract API.
    pub pillar: pillar::PillarApi<C>,
    /// Plasma contract API.
    pub plasma: plasma::PlasmaApi<C>,
    /// Sentinel contract API.
    pub sentinel: sentinel::SentinelApi<C>,
    /// Spork contract API.
    pub spork: spork::SporkApi<C>,
    /// Stake contract API.
    pub stake: stake::StakeApi<C>,
    /// Swap contract API.
    pub swap: swap::SwapApi<C>,
    /// Token contract API.
    pub token: token::TokenApi<C>,
}

impl<C: Client> EmbeddedApi<C> {
    /// Creates every embedded sub-API from the same shared client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            accelerator: accelerator::AcceleratorApi::new(Arc::clone(&client)),
            bridge: bridge::BridgeApi::new(Arc::clone(&client)),
            htlc: htlc::HtlcApi::new(Arc::clone(&client)),
            liquidity: liquidity::LiquidityApi::new(Arc::clone(&client)),
            pillar: pillar::PillarApi::new(Arc::clone(&client)),
            plasma: plasma::PlasmaApi::new(Arc::clone(&client)),
            sentinel: sentinel::SentinelApi::new(Arc::clone(&client)),
            spork: spork::SporkApi::new(Arc::clone(&client)),
            stake: stake::StakeApi::new(Arc::clone(&client)),
            swap: swap::SwapApi::new(Arc::clone(&client)),
            token: token::TokenApi::new(client),
        }
    }
}

/// Dispatches `method` with `params`, returning the node's JSON result.
pub(crate) async fn dispatch<C: Client>(
    client: &C,
    method: &str,
    params: &[Value],
) -> Result<Value, Error> {
    client
        .send_request(method, params)
        .await
        .map_err(Error::from)
}

/// Encodes a contract method call (4-byte selector + ABI-encoded arguments) for
/// a method declared in `definition`.
///
/// The embedded definitions are compile-time constants validated by the
/// definitions test suite, so the parse and encode steps are infallible in
/// practice; a malformed definition or argument list is a programming error.
#[allow(clippy::expect_used)]
pub(crate) fn encode_call(definition: &str, name: &str, values: &[AbiValue]) -> Vec<u8> {
    let value: Value = serde_json::from_str(definition).expect("embedded definition is valid JSON");
    Abi::from_json(&value)
        .expect("embedded definition parses")
        .encode_function(name, values)
        .expect("embedded method encodes")
}

/// Encodes a method call (selector + ABI arguments) from an explicit canonical
/// method name and argument types, without an ABI-definition lookup.
///
/// Used when a builder's canonical method name differs from the embedded
/// definition's JSON entry name. The plasma `Cancel` method's definition entry
/// is legacy-named `CancelFuse`, but the contract's canonical selector is
/// derived from `Cancel(hash)`.
#[allow(clippy::expect_used)]
pub(crate) fn encode_named_call(name: &str, types: &[AbiType], values: &[AbiValue]) -> Vec<u8> {
    let mut data = selector(name, types).to_vec();
    data.extend(encode_arguments(types, values).expect("method arguments encode"));
    data
}

/// Parses a canonical embedded contract address string.
#[allow(clippy::expect_used)]
pub(crate) fn embedded_address(canonical: &str) -> Address {
    Address::parse(canonical).expect("canonical embedded address parses")
}

// The shared decode/page helpers live in `crate::api` so both the ledger and
// embedded namespaces import one canonical definition. They are re-exported
// here so existing `crate::api::embedded::{...}` call sites stay unchanged.
pub(crate) use crate::api::{
    address_page_params, big_uint_from_value, bool_from_value, optional, page_params,
    u64_from_value,
};

/// Decodes an array whose entries may individually be `null` into a vector of
/// `Option<T>`.
pub(crate) fn nullable_array<T, F>(value: &Value, decode: F) -> Result<Vec<Option<T>>, Error>
where
    F: Fn(&Value) -> Result<T, Error>,
{
    let array = value.as_array().ok_or_else(|| {
        Error::InvalidInput("expected a JSON array of nullable entries".to_string())
    })?;
    array
        .iter()
        .map(|entry| {
            if entry.is_null() {
                Ok(None)
            } else {
                decode(entry).map(Some)
            }
        })
        .collect()
}

/// Serializes a string plus a page index/size pair as positional JSON params.
pub(crate) fn string_page_params(value: &str, page: crate::api::PageQuery) -> Vec<Value> {
    vec![
        Value::from(value),
        Value::from(page.index),
        Value::from(page.size),
    ]
}

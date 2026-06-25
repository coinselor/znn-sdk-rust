//! HTLC embedded contract API.

use crate::abi::AbiValue;
use crate::api::embedded::{bool_from_value, dispatch, embedded_address, encode_call};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::definitions::HTLC_DEFINITION;
use crate::error::Error;
use crate::model::embedded::htlc::HtlcInfo;
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::model::nom::token::Token;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::znn_token_standard;
use num_bigint::{BigInt, BigUint};
use serde_json::json;
use std::sync::Arc;

/// HTLC API root.
pub struct HtlcApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> HtlcApi<C> {
    /// Creates an HTLC API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns HTLC info by hash.
    pub async fn get_by_hash(&self, hash: &Hash) -> Result<HtlcInfo, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.htlc.getByHash",
            &[json!(hash.to_string())],
        )
        .await?;
        HtlcInfo::from_json(&response)
    }

    /// Returns proxy unlock status.
    pub async fn get_proxy_unlock_status(&self, address: &Address) -> Result<bool, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.htlc.getProxyUnlockStatus",
            &[json!(address.to_string())],
        )
        .await?;
        bool_from_value(&response, "proxy unlock status")
    }

    /// Builds a create template.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        token: &Token,
        amount: BigUint,
        hash_locked: Address,
        expiration_time: i64,
        hash_type: u8,
        key_max_size: u8,
        hash_lock: Vec<u8>,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            HTLC_DEFINITION,
            "Create",
            &[
                AbiValue::Address(hash_locked),
                AbiValue::Int(BigInt::from(expiration_time)),
                AbiValue::UInt(BigUint::from(hash_type)),
                AbiValue::UInt(BigUint::from(key_max_size)),
                AbiValue::Bytes(hash_lock),
            ],
        );
        AccountBlockTemplate::call_contract(
            htlc_address(),
            token.token_standard().clone(),
            amount,
            data,
        )
    }

    /// Builds a reclaim template.
    pub fn reclaim(&self, id: &Hash) -> AccountBlockTemplate {
        let data = encode_call(HTLC_DEFINITION, "Reclaim", &[AbiValue::Hash(id.clone())]);
        AccountBlockTemplate::call_contract(
            htlc_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds an unlock template.
    pub fn unlock(&self, id: &Hash, preimage: Vec<u8>) -> AccountBlockTemplate {
        let data = encode_call(
            HTLC_DEFINITION,
            "Unlock",
            &[AbiValue::Hash(id.clone()), AbiValue::Bytes(preimage)],
        );
        AccountBlockTemplate::call_contract(
            htlc_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a deny-proxy-unlock template.
    pub fn deny_proxy_unlock(&self) -> AccountBlockTemplate {
        let data = encode_call(HTLC_DEFINITION, "DenyProxyUnlock", &[]);
        AccountBlockTemplate::call_contract(
            htlc_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds an allow-proxy-unlock template.
    pub fn allow_proxy_unlock(&self) -> AccountBlockTemplate {
        let data = encode_call(HTLC_DEFINITION, "AllowProxyUnlock", &[]);
        AccountBlockTemplate::call_contract(
            htlc_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the HTLC contract address.
pub fn htlc_address() -> Address {
    embedded_address("z1qxemdeddedxhtlcxxxxxxxxxxxxxxxxxygecvw")
}

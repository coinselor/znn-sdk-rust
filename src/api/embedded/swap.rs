//! Swap embedded contract API.

use crate::abi::AbiValue;
use crate::api::embedded::{dispatch, embedded_address, encode_call};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::constants::{
    GENESIS_TIMESTAMP, SWAP_ASSET_DECAY_EPOCHS_OFFSET, SWAP_ASSET_DECAY_TICK_EPOCHS,
    SWAP_ASSET_DECAY_TICK_VALUE_PERCENTAGE, SWAP_ASSET_DECAY_TIMESTAMP_START,
};
use crate::embedded::definitions::SWAP_DEFINITION;
use crate::error::Error;
use crate::model::embedded::swap::{SwapAssetEntry, SwapLegacyPillarEntry};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::znn_token_standard;
use num_bigint::BigUint;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Seconds per day, used to convert timestamps to swap-decay epochs.
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// Swap API root.
pub struct SwapApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> SwapApi<C> {
    /// Creates a swap API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns swap assets by key-id hash.
    pub async fn get_assets_by_key_id_hash(
        &self,
        key_id_hash: &str,
    ) -> Result<SwapAssetEntry, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.swap.getAssetsByKeyIdHash",
            &[json!(key_id_hash)],
        )
        .await?;
        let key = Hash::parse(key_id_hash)?;
        SwapAssetEntry::from_json(key, &response)
    }

    /// Returns all swap assets, keyed by their key-id hash.
    pub async fn get_assets(&self) -> Result<HashMap<Hash, SwapAssetEntry>, Error> {
        let response = dispatch(&*self.client, "embedded.swap.getAssets", &[]).await?;
        let object = response.as_object().ok_or_else(|| {
            Error::InvalidInput("swap assets response must be a JSON object".to_string())
        })?;
        object
            .iter()
            .map(|(key, value)| {
                let hash = Hash::parse(key)?;
                Ok((hash.clone(), SwapAssetEntry::from_json(hash, value)?))
            })
            .collect()
    }

    /// Returns the legacy pillar entries.
    pub async fn get_legacy_pillars(&self) -> Result<Vec<SwapLegacyPillarEntry>, Error> {
        let response = dispatch(&*self.client, "embedded.swap.getLegacyPillars", &[]).await?;
        let array = response.as_array().ok_or_else(|| {
            Error::InvalidInput("legacy pillars response must be a JSON array".to_string())
        })?;
        array.iter().map(SwapLegacyPillarEntry::from_json).collect()
    }

    /// Returns the swap-asset decay percentage at `current_timestamp`.
    ///
    /// Zero before the decay window opens, then `10%` per decay tick up to `100%`.
    pub fn swap_decay_percentage(current_timestamp: u64) -> u64 {
        if current_timestamp < SWAP_ASSET_DECAY_TIMESTAMP_START {
            return 0;
        }
        let current_epoch = current_timestamp.saturating_sub(GENESIS_TIMESTAMP) / SECONDS_PER_DAY;
        let num_ticks = (current_epoch + 1).saturating_sub(SWAP_ASSET_DECAY_EPOCHS_OFFSET)
            / SWAP_ASSET_DECAY_TICK_EPOCHS;
        let decay_factor = SWAP_ASSET_DECAY_TICK_VALUE_PERCENTAGE * num_ticks;
        let percentage_to_give = 100_u64.saturating_sub(decay_factor);
        100 - percentage_to_give
    }

    /// Builds a retrieve-assets template.
    pub fn retrieve_assets(&self, pub_key: &str, signature: &str) -> AccountBlockTemplate {
        let data = encode_call(
            SWAP_DEFINITION,
            "RetrieveAssets",
            &[
                AbiValue::String(pub_key.to_string()),
                AbiValue::String(signature.to_string()),
            ],
        );
        AccountBlockTemplate::call_contract(
            swap_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the swap contract address.
pub fn swap_address() -> Address {
    embedded_address("z1qxemdeddedxswapxxxxxxxxxxxxxxxxxxl4yww")
}

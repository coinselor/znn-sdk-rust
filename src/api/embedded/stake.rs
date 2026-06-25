//! Stake embedded contract API.

use crate::abi::AbiValue;
use crate::api::PageQuery;
use crate::api::embedded::{address_page_params, dispatch, embedded_address, encode_call};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::definitions::{COMMON_DEFINITION, STAKE_DEFINITION};
use crate::error::Error;
use crate::model::embedded::common::{RewardHistoryList, UncollectedReward};
use crate::model::embedded::stake::StakeList;
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::znn_token_standard;
use num_bigint::{BigInt, BigUint};
use serde_json::json;
use std::sync::Arc;

/// Stake API root.
pub struct StakeApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> StakeApi<C> {
    /// Creates a stake API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns stake entries for `address`.
    pub async fn get_entries_by_address(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<StakeList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.stake.getEntriesByAddress",
            &address_page_params(address, page),
        )
        .await?;
        StakeList::from_json(&response)
    }

    /// Returns the uncollected reward for `address`.
    pub async fn get_uncollected_reward(
        &self,
        address: &Address,
    ) -> Result<UncollectedReward, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.stake.getUncollectedReward",
            &[json!(address.to_string())],
        )
        .await?;
        UncollectedReward::from_json(&response)
    }

    /// Returns reward history by page for `address`.
    pub async fn get_frontier_reward_by_page(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<RewardHistoryList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.stake.getFrontierRewardByPage",
            &address_page_params(address, page),
        )
        .await?;
        RewardHistoryList::from_json(&response)
    }

    /// Builds a stake template.
    pub fn stake(&self, duration_in_sec: u64, amount: BigUint) -> AccountBlockTemplate {
        let data = encode_call(
            STAKE_DEFINITION,
            "Stake",
            &[AbiValue::Int(BigInt::from(duration_in_sec))],
        );
        AccountBlockTemplate::call_contract(stake_address(), znn_token_standard(), amount, data)
    }

    /// Builds a cancel template.
    pub fn cancel(&self, id: &Hash) -> AccountBlockTemplate {
        let data = encode_call(STAKE_DEFINITION, "Cancel", &[AbiValue::Hash(id.clone())]);
        AccountBlockTemplate::call_contract(
            stake_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a collect-reward template.
    pub fn collect_reward(&self) -> AccountBlockTemplate {
        let data = encode_call(COMMON_DEFINITION, "CollectReward", &[]);
        AccountBlockTemplate::call_contract(
            stake_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the stake contract address.
pub fn stake_address() -> Address {
    embedded_address("z1qxemdeddedxstakexxxxxxxxxxxxxxxxjv8v62")
}

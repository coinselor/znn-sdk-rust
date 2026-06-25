//! Sentinel embedded contract API.

use crate::api::PageQuery;
use crate::api::embedded::{
    address_page_params, big_uint_from_value, dispatch, embedded_address, encode_call, optional,
    page_params,
};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::constants::SENTINEL_REGISTER_ZNN_AMOUNT;
use crate::embedded::definitions::{COMMON_DEFINITION, SENTINEL_DEFINITION};
use crate::error::Error;
use crate::model::embedded::common::{RewardHistoryList, UncollectedReward};
use crate::model::embedded::sentinel::{SentinelInfo, SentinelInfoList};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::token_standard::{qsr_token_standard, znn_token_standard};
use num_bigint::BigUint;
use serde_json::json;
use std::sync::Arc;

/// Sentinel API root.
pub struct SentinelApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> SentinelApi<C> {
    /// Creates a sentinel API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns all active sentinels.
    pub async fn get_all_active(&self, page: PageQuery) -> Result<SentinelInfoList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.sentinel.getAllActive",
            &page_params(page),
        )
        .await?;
        SentinelInfoList::from_json(&response)
    }

    /// Returns the sentinel owned by `address`, or `None`.
    pub async fn get_by_owner(&self, address: &Address) -> Result<Option<SentinelInfo>, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.sentinel.getByOwner",
            &[json!(address.to_string())],
        )
        .await?;
        optional(&response, SentinelInfo::from_json)
    }

    /// Returns the QSR deposited for sentinel registration by `address`.
    pub async fn get_deposited_qsr(&self, address: &Address) -> Result<BigUint, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.sentinel.getDepositedQsr",
            &[json!(address.to_string())],
        )
        .await?;
        big_uint_from_value(&response, "deposited QSR")
    }

    /// Returns the uncollected reward for `address`.
    pub async fn get_uncollected_reward(
        &self,
        address: &Address,
    ) -> Result<UncollectedReward, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.sentinel.getUncollectedReward",
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
            "embedded.sentinel.getFrontierRewardByPage",
            &address_page_params(address, page),
        )
        .await?;
        RewardHistoryList::from_json(&response)
    }

    /// Builds a register template.
    pub fn register(&self) -> AccountBlockTemplate {
        let data = encode_call(SENTINEL_DEFINITION, "Register", &[]);
        AccountBlockTemplate::call_contract(
            sentinel_address(),
            znn_token_standard(),
            BigUint::from(SENTINEL_REGISTER_ZNN_AMOUNT),
            data,
        )
    }

    /// Builds a revoke template.
    pub fn revoke(&self) -> AccountBlockTemplate {
        let data = encode_call(SENTINEL_DEFINITION, "Revoke", &[]);
        AccountBlockTemplate::call_contract(
            sentinel_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a collect-reward template.
    pub fn collect_reward(&self) -> AccountBlockTemplate {
        let data = encode_call(COMMON_DEFINITION, "CollectReward", &[]);
        AccountBlockTemplate::call_contract(
            sentinel_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a deposit-QSR template.
    pub fn deposit_qsr(&self, amount: BigUint) -> AccountBlockTemplate {
        let data = encode_call(COMMON_DEFINITION, "DepositQsr", &[]);
        AccountBlockTemplate::call_contract(sentinel_address(), qsr_token_standard(), amount, data)
    }

    /// Builds a withdraw-QSR template.
    pub fn withdraw_qsr(&self) -> AccountBlockTemplate {
        let data = encode_call(COMMON_DEFINITION, "WithdrawQsr", &[]);
        AccountBlockTemplate::call_contract(
            sentinel_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the sentinel contract address.
pub fn sentinel_address() -> Address {
    embedded_address("z1qxemdeddedxsentynelxxxxxxxxxxxxxwy0r2r")
}

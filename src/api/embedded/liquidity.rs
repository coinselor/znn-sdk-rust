//! Liquidity embedded contract API.

use crate::abi::AbiValue;
use crate::api::PageQuery;
use crate::api::embedded::{address_page_params, dispatch, embedded_address, encode_call};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::definitions::LIQUIDITY_DEFINITION;
use crate::error::Error;
use crate::model::embedded::bridge::TimeChallengesList;
use crate::model::embedded::common::{RewardDeposit, RewardHistoryList, SecurityInfo};
use crate::model::embedded::liquidity::{LiquidityInfo, LiquidityStakeList, TokenTuple};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::{TokenStandard, znn_token_standard};
use num_bigint::{BigInt, BigUint};
use serde_json::json;
use std::sync::Arc;

/// Liquidity API root.
pub struct LiquidityApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> LiquidityApi<C> {
    /// Creates a liquidity API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns uncollected reward.
    pub async fn get_uncollected_reward(&self, address: &Address) -> Result<RewardDeposit, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.liquidity.getUncollectedReward",
            &[json!(address.to_string())],
        )
        .await?;
        RewardDeposit::from_json(&response)
    }

    /// Returns reward history.
    pub async fn get_frontier_reward_by_page(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<RewardHistoryList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.liquidity.getFrontierRewardByPage",
            &address_page_params(address, page),
        )
        .await?;
        RewardHistoryList::from_json(&response)
    }

    /// Returns security info.
    pub async fn get_security_info(&self) -> Result<SecurityInfo, Error> {
        let response = dispatch(&*self.client, "embedded.liquidity.getSecurityInfo", &[]).await?;
        SecurityInfo::from_json(&response)
    }

    /// Returns time challenges.
    pub async fn get_time_challenges_info(&self) -> Result<TimeChallengesList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.liquidity.getTimeChallengesInfo",
            &[],
        )
        .await?;
        TimeChallengesList::from_json(&response)
    }

    /// Returns liquidity info.
    pub async fn get_liquidity_info(&self) -> Result<LiquidityInfo, Error> {
        let response = dispatch(&*self.client, "embedded.liquidity.getLiquidityInfo", &[]).await?;
        LiquidityInfo::from_json(&response)
    }

    /// Returns stake entries for an address.
    pub async fn get_liquidity_stake_entries_by_address(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<LiquidityStakeList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.liquidity.getLiquidityStakeEntriesByAddress",
            &address_page_params(address, page),
        )
        .await?;
        LiquidityStakeList::from_json(&response)
    }

    /// Builds a liquidity-stake template.
    pub fn liquidity_stake(
        &self,
        duration: u64,
        amount: BigUint,
        zts: TokenStandard,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "LiquidityStake",
            &[AbiValue::Int(BigInt::from(duration))],
        );
        AccountBlockTemplate::call_contract(liquidity_address(), zts, amount, data)
    }

    /// Builds a cancel template.
    pub fn cancel_liquidity_stake(&self, id: &Hash) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "CancelLiquidityStake",
            &[AbiValue::Hash(id.clone())],
        );
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds an unlock template.
    pub fn unlock_liquidity_stake_entries(&self, _ids: &[Hash]) -> AccountBlockTemplate {
        let data = encode_call(LIQUIDITY_DEFINITION, "UnlockLiquidityStakeEntries", &[]);
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a set-token-tuple template.
    #[allow(clippy::needless_pass_by_value)]
    pub fn set_token_tuple(&self, tuple: TokenTuple) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "SetTokenTuple",
            &[
                AbiValue::Array(vec![AbiValue::String(tuple.token_standard().to_string())]),
                AbiValue::Array(vec![AbiValue::UInt(BigUint::from(tuple.znn_percentage()))]),
                AbiValue::Array(vec![AbiValue::UInt(BigUint::from(tuple.qsr_percentage()))]),
                AbiValue::Array(vec![AbiValue::UInt(tuple.min_amount().clone())]),
            ],
        );
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a nominate-guardians template.
    pub fn nominate_guardians(&self, guardians: &[Address]) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "NominateGuardians",
            &[AbiValue::Array(
                guardians.iter().cloned().map(AbiValue::Address).collect(),
            )],
        );
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a propose-administrator template.
    pub fn propose_administrator(&self, address: Address) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "ProposeAdministrator",
            &[AbiValue::Address(address)],
        );
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a set-halt-state template.
    pub fn set_is_halted(&self, is_halted: bool) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "SetIsHalted",
            &[AbiValue::Bool(is_halted)],
        );
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a set-additional-reward template.
    pub fn set_additional_reward(
        &self,
        znn_reward: BigUint,
        qsr_reward: BigUint,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "SetAdditionalReward",
            &[AbiValue::UInt(znn_reward), AbiValue::UInt(qsr_reward)],
        );
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a change-administrator template.
    pub fn change_administrator(&self, administrator: Address) -> AccountBlockTemplate {
        let data = encode_call(
            LIQUIDITY_DEFINITION,
            "ChangeAdministrator",
            &[AbiValue::Address(administrator)],
        );
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a collect-reward template.
    pub fn collect_reward(&self) -> AccountBlockTemplate {
        let data = encode_call(LIQUIDITY_DEFINITION, "CollectReward", &[]);
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds an emergency template.
    pub fn emergency(&self) -> AccountBlockTemplate {
        let data = encode_call(LIQUIDITY_DEFINITION, "Emergency", &[]);
        AccountBlockTemplate::call_contract(
            liquidity_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the liquidity contract address.
pub fn liquidity_address() -> Address {
    embedded_address("z1qxemdeddedxlyquydytyxxxxxxxxxxxxflaaae")
}

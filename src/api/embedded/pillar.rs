//! Pillar embedded contract API.

use crate::abi::{AbiType, AbiValue};
use crate::api::PageQuery;
use crate::api::embedded::{
    address_page_params, big_uint_from_value, bool_from_value, dispatch, embedded_address,
    encode_call, encode_named_call, optional, page_params, string_page_params,
};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::constants::PILLAR_REGISTER_ZNN_AMOUNT;
use crate::embedded::definitions::{COMMON_DEFINITION, PILLAR_DEFINITION};
use crate::error::Error;
use crate::model::embedded::common::{RewardHistoryList, UncollectedReward};
use crate::model::embedded::pillar::{
    DelegationInfo, PillarEpochHistoryList, PillarInfo, PillarInfoList,
};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::token_standard::{qsr_token_standard, znn_token_standard};
use num_bigint::BigUint;
use serde_json::json;
use std::sync::Arc;

/// Pillar API root.
pub struct PillarApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> PillarApi<C> {
    /// Creates a pillar API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns deposited QSR.
    pub async fn get_deposited_qsr(&self, address: &Address) -> Result<BigUint, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.getDepositedQsr",
            &[json!(address.to_string())],
        )
        .await?;
        big_uint_from_value(&response, "deposited QSR")
    }

    /// Returns uncollected reward.
    pub async fn get_uncollected_reward(
        &self,
        address: &Address,
    ) -> Result<UncollectedReward, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.getUncollectedReward",
            &[json!(address.to_string())],
        )
        .await?;
        UncollectedReward::from_json(&response)
    }

    /// Returns reward history.
    pub async fn get_frontier_reward_by_page(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<RewardHistoryList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.getFrontierRewardByPage",
            &address_page_params(address, page),
        )
        .await?;
        RewardHistoryList::from_json(&response)
    }

    /// Returns QSR registration cost.
    pub async fn get_qsr_registration_cost(&self) -> Result<BigUint, Error> {
        let response =
            dispatch(&*self.client, "embedded.pillar.getQsrRegistrationCost", &[]).await?;
        big_uint_from_value(&response, "QSR registration cost")
    }

    /// Returns all pillars.
    pub async fn get_all(&self, page: PageQuery) -> Result<PillarInfoList, Error> {
        let response =
            dispatch(&*self.client, "embedded.pillar.getAll", &page_params(page)).await?;
        PillarInfoList::from_json(&response)
    }

    /// Returns pillars by owner.
    pub async fn get_by_owner(&self, address: &Address) -> Result<Vec<PillarInfo>, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.getByOwner",
            &[json!(address.to_string())],
        )
        .await?;
        let array = response.as_array().ok_or_else(|| {
            Error::InvalidInput("pillar owner response must be a JSON array".to_string())
        })?;
        array.iter().map(PillarInfo::from_json).collect()
    }

    /// Returns pillar by name.
    pub async fn get_by_name(&self, name: &str) -> Result<Option<PillarInfo>, Error> {
        let response = dispatch(&*self.client, "embedded.pillar.getByName", &[json!(name)]).await?;
        optional(&response, PillarInfo::from_json)
    }

    /// Checks name availability.
    pub async fn check_name_availability(&self, name: &str) -> Result<bool, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.checkNameAvailability",
            &[json!(name)],
        )
        .await?;
        bool_from_value(&response, "pillar name availability")
    }

    /// Returns delegated pillar.
    pub async fn get_delegated_pillar(
        &self,
        address: &Address,
    ) -> Result<Option<DelegationInfo>, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.getDelegatedPillar",
            &[json!(address.to_string())],
        )
        .await?;
        optional(&response, DelegationInfo::from_json)
    }

    /// Returns pillar epoch history.
    pub async fn get_pillar_epoch_history(
        &self,
        name: &str,
        page: PageQuery,
    ) -> Result<PillarEpochHistoryList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.getPillarEpochHistory",
            &string_page_params(name, page),
        )
        .await?;
        PillarEpochHistoryList::from_json(&response)
    }

    /// Returns pillars history by epoch.
    pub async fn get_pillars_history_by_epoch(
        &self,
        epoch: u64,
        page: PageQuery,
    ) -> Result<PillarEpochHistoryList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.pillar.getPillarsHistoryByEpoch",
            &[json!(epoch), json!(page.index), json!(page.size)],
        )
        .await?;
        PillarEpochHistoryList::from_json(&response)
    }

    /// Builds a register template.
    pub fn register(
        &self,
        name: &str,
        producer: Address,
        reward: Address,
        give_block_pct: u8,
        give_delegate_pct: u8,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            PILLAR_DEFINITION,
            "Register",
            &[
                AbiValue::String(name.to_string()),
                AbiValue::Address(producer),
                AbiValue::Address(reward),
                AbiValue::UInt(BigUint::from(give_block_pct)),
                AbiValue::UInt(BigUint::from(give_delegate_pct)),
            ],
        );
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(PILLAR_REGISTER_ZNN_AMOUNT),
            data,
        )
    }

    /// Builds a legacy-register template.
    pub fn register_legacy(
        &self,
        name: &str,
        producer: Address,
        reward: Address,
        give_block_pct: u8,
        give_delegate_pct: u8,
    ) -> AccountBlockTemplate {
        let data = encode_named_call(
            "RegisterLegacy",
            &[
                AbiType::String,
                AbiType::Address,
                AbiType::Address,
                AbiType::UInt(8),
                AbiType::UInt(8),
            ],
            &[
                AbiValue::String(name.to_string()),
                AbiValue::Address(producer),
                AbiValue::Address(reward),
                AbiValue::UInt(BigUint::from(give_block_pct)),
                AbiValue::UInt(BigUint::from(give_delegate_pct)),
            ],
        );
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(PILLAR_REGISTER_ZNN_AMOUNT),
            data,
        )
    }

    /// Builds an update-pillar template.
    pub fn update_pillar(
        &self,
        name: &str,
        producer: Address,
        reward: Address,
        give_block_pct: u8,
        give_delegate_pct: u8,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            PILLAR_DEFINITION,
            "UpdatePillar",
            &[
                AbiValue::String(name.to_string()),
                AbiValue::Address(producer),
                AbiValue::Address(reward),
                AbiValue::UInt(BigUint::from(give_block_pct)),
                AbiValue::UInt(BigUint::from(give_delegate_pct)),
            ],
        );
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a revoke template.
    pub fn revoke(&self, name: &str) -> AccountBlockTemplate {
        let data = encode_call(
            PILLAR_DEFINITION,
            "Revoke",
            &[AbiValue::String(name.to_string())],
        );
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a delegate template.
    pub fn delegate(&self, name: &str) -> AccountBlockTemplate {
        let data = encode_call(
            PILLAR_DEFINITION,
            "Delegate",
            &[AbiValue::String(name.to_string())],
        );
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds an undelegate template.
    pub fn undelegate(&self) -> AccountBlockTemplate {
        let data = encode_call(PILLAR_DEFINITION, "Undelegate", &[]);
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a collect-reward template.
    pub fn collect_reward(&self) -> AccountBlockTemplate {
        let data = encode_call(COMMON_DEFINITION, "CollectReward", &[]);
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a deposit-QSR template.
    pub fn deposit_qsr(&self, amount: BigUint) -> AccountBlockTemplate {
        let data = encode_call(COMMON_DEFINITION, "DepositQsr", &[]);
        AccountBlockTemplate::call_contract(pillar_address(), qsr_token_standard(), amount, data)
    }

    /// Builds a withdraw-QSR template.
    pub fn withdraw_qsr(&self) -> AccountBlockTemplate {
        let data = encode_call(COMMON_DEFINITION, "WithdrawQsr", &[]);
        AccountBlockTemplate::call_contract(
            pillar_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the pillar contract address.
pub fn pillar_address() -> Address {
    embedded_address("z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg")
}

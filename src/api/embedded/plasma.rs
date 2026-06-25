//! Plasma embedded contract API.

use crate::abi::{AbiType, AbiValue};
use crate::api::PageQuery;
use crate::api::embedded::{
    address_page_params, dispatch, embedded_address, encode_call, encode_named_call, u64_from_value,
};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::definitions::PLASMA_DEFINITION;
use crate::error::Error;
use crate::model::embedded::plasma::{
    FusionEntryList, GetRequiredParam, GetRequiredResponse, PlasmaInfo,
};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::{qsr_token_standard, znn_token_standard};
use num_bigint::BigUint;
use serde_json::json;
use std::sync::Arc;

/// Plasma multiplier: one QSR yields this much plasma.
const PLASMA_PER_QSR: u64 = 2100;

/// Plasma API root.
pub struct PlasmaApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> PlasmaApi<C> {
    /// Creates a plasma API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns plasma info for `address`.
    pub async fn get(&self, address: &Address) -> Result<PlasmaInfo, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.plasma.get",
            &[json!(address.to_string())],
        )
        .await?;
        PlasmaInfo::from_json(&response)
    }

    /// Returns fusion entries for `address`.
    pub async fn get_entries_by_address(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<FusionEntryList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.plasma.getEntriesByAddress",
            &address_page_params(address, page),
        )
        .await?;
        FusionEntryList::from_json(&response)
    }

    /// Returns the QSR amount required to fuse for `required_plasma`.
    pub async fn get_required_fusion_amount(&self, required_plasma: u32) -> Result<u64, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.plasma.getRequiredFusionAmount",
            &[json!(required_plasma)],
        )
        .await?;
        u64_from_value(&response, "required fusion amount")
    }

    /// Returns the proof-of-work required for an account block.
    pub async fn get_required_pow_for_account_block(
        &self,
        param: &GetRequiredParam,
    ) -> Result<GetRequiredResponse, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            &[param.to_json()],
        )
        .await?;
        GetRequiredResponse::from_json(&response)
    }

    /// Converts a fused QSR amount to its plasma equivalent (`qsr * 2100`).
    pub fn plasma_by_qsr(qsr: &BigUint) -> BigUint {
        qsr * BigUint::from(PLASMA_PER_QSR)
    }

    /// Builds a fuse template.
    pub fn fuse(&self, beneficiary: Address, amount: BigUint) -> AccountBlockTemplate {
        let data = encode_call(PLASMA_DEFINITION, "Fuse", &[AbiValue::Address(beneficiary)]);
        AccountBlockTemplate::call_contract(plasma_address(), qsr_token_standard(), amount, data)
    }

    /// Builds a cancel-fuse template.
    pub fn cancel(&self, id: &Hash) -> AccountBlockTemplate {
        // The plasma contract's canonical method is `Cancel`; the embedded
        // definition entry is legacy-named `CancelFuse`.
        let data = encode_named_call("Cancel", &[AbiType::Hash], &[AbiValue::Hash(id.clone())]);
        AccountBlockTemplate::call_contract(
            plasma_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the plasma contract address.
pub fn plasma_address() -> Address {
    embedded_address("z1qxemdeddedxplasmaxxxxxxxxxxxxxxxxsctrp")
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn plasma_by_qsr_multiplies_by_2100() {
        assert_eq!(
            PlasmaApi::<crate::client::websocket::WsClient>::plasma_by_qsr(&BigUint::from(
                crate::utils::nom_constants::ONE_QSR
            )),
            BigUint::from(2100u64) * BigUint::from(crate::utils::nom_constants::ONE_QSR)
        );
    }
}

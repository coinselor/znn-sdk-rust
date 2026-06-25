//! Bridge embedded contract API.

use crate::abi::AbiValue;
use crate::api::PageQuery;
use crate::api::embedded::{
    dispatch, embedded_address, encode_call, page_params, string_page_params,
};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::definitions::BRIDGE_DEFINITION;
use crate::error::Error;
use crate::model::embedded::bridge::{
    BridgeInfo, BridgeNetworkInfo, BridgeNetworkInfoList, OrchestratorInfo, TimeChallengesList,
    TokenPair, UnwrapTokenRequest, UnwrapTokenRequestList, WrapTokenRequest, WrapTokenRequestList,
};
use crate::model::embedded::common::SecurityInfo;
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::{TokenStandard, znn_token_standard};
use num_bigint::BigUint;
use serde_json::json;
use std::sync::Arc;

/// Bridge API root.
pub struct BridgeApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> BridgeApi<C> {
    /// Creates a bridge API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns bridge security info.
    pub async fn get_security_info(&self) -> Result<SecurityInfo, Error> {
        let response = dispatch(&*self.client, "embedded.bridge.getSecurityInfo", &[]).await?;
        SecurityInfo::from_json(&response)
    }

    /// Returns time challenges.
    pub async fn get_time_challenges_info(&self) -> Result<TimeChallengesList, Error> {
        let response =
            dispatch(&*self.client, "embedded.bridge.getTimeChallengesInfo", &[]).await?;
        TimeChallengesList::from_json(&response)
    }

    /// Returns bridge info.
    pub async fn get_bridge_info(&self) -> Result<BridgeInfo, Error> {
        let response = dispatch(&*self.client, "embedded.bridge.getBridgeInfo", &[]).await?;
        BridgeInfo::from_json(&response)
    }

    /// Returns orchestrator info.
    pub async fn get_orchestrator_info(&self) -> Result<OrchestratorInfo, Error> {
        let response = dispatch(&*self.client, "embedded.bridge.getOrchestratorInfo", &[]).await?;
        OrchestratorInfo::from_json(&response)
    }

    /// Returns one network.
    pub async fn get_network_info(
        &self,
        network_class: u32,
        chain_id: u32,
    ) -> Result<BridgeNetworkInfo, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getNetworkInfo",
            &[json!(network_class), json!(chain_id)],
        )
        .await?;
        BridgeNetworkInfo::from_json(&response)
    }

    /// Returns all networks.
    pub async fn get_all_networks(&self, page: PageQuery) -> Result<BridgeNetworkInfoList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getAllNetworks",
            &page_params(page),
        )
        .await?;
        BridgeNetworkInfoList::from_json(&response)
    }

    /// Returns a wrap request by hash.
    pub async fn get_wrap_token_request_by_hash(
        &self,
        hash: &Hash,
    ) -> Result<WrapTokenRequest, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getWrapTokenRequestByHash",
            &[json!(hash.to_string())],
        )
        .await?;
        WrapTokenRequest::from_json(&response)
    }

    /// Returns all wrap requests.
    pub async fn get_all_wrap_token_requests(
        &self,
        page: PageQuery,
    ) -> Result<WrapTokenRequestList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getAllWrapTokenRequests",
            &page_params(page),
        )
        .await?;
        WrapTokenRequestList::from_json(&response)
    }

    /// Returns wrap requests by destination.
    pub async fn get_all_wrap_token_requests_by_to_address(
        &self,
        addr: &str,
        page: PageQuery,
    ) -> Result<WrapTokenRequestList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getAllWrapTokenRequestsByToAddress",
            &string_page_params(addr, page),
        )
        .await?;
        WrapTokenRequestList::from_json(&response)
    }

    /// Returns wrap requests by destination network.
    pub async fn get_all_wrap_token_requests_by_to_address_network_class_and_chain_id(
        &self,
        addr: &str,
        network_class: u32,
        chain_id: u32,
        page: PageQuery,
    ) -> Result<WrapTokenRequestList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getAllWrapTokenRequestsByToAddressNetworkClassAndChainId",
            &[
                json!(addr),
                json!(network_class),
                json!(chain_id),
                json!(page.index),
                json!(page.size),
            ],
        )
        .await?;
        WrapTokenRequestList::from_json(&response)
    }

    /// Returns unsigned wrap requests.
    pub async fn get_all_unsigned_wrap_token_requests(
        &self,
        page: PageQuery,
    ) -> Result<WrapTokenRequestList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getAllUnsignedWrapTokenRequests",
            &page_params(page),
        )
        .await?;
        WrapTokenRequestList::from_json(&response)
    }

    /// Returns an unwrap request by hash and log index.
    pub async fn get_unwrap_token_request_by_hash_and_log(
        &self,
        hash: &Hash,
        log_index: u32,
    ) -> Result<UnwrapTokenRequest, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getUnwrapTokenRequestByHashAndLog",
            &[json!(hash.to_string()), json!(log_index)],
        )
        .await?;
        UnwrapTokenRequest::from_json(&response)
    }

    /// Returns all unwrap requests.
    pub async fn get_all_unwrap_token_requests(
        &self,
        page: PageQuery,
    ) -> Result<UnwrapTokenRequestList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getAllUnwrapTokenRequests",
            &page_params(page),
        )
        .await?;
        UnwrapTokenRequestList::from_json(&response)
    }

    /// Returns unwrap requests by destination.
    pub async fn get_all_unwrap_token_requests_by_to_address(
        &self,
        addr: &str,
        page: PageQuery,
    ) -> Result<UnwrapTokenRequestList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getAllUnwrapTokenRequestsByToAddress",
            &string_page_params(addr, page),
        )
        .await?;
        UnwrapTokenRequestList::from_json(&response)
    }

    /// Returns a fee token pair.
    pub async fn get_fee_token_pair(&self, zts: &TokenStandard) -> Result<TokenPair, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.bridge.getFeeTokenPair",
            &[json!(zts.to_string())],
        )
        .await?;
        TokenPair::from_json(&response)
    }

    /// Builds a wrap-token template.
    pub fn wrap_token(
        &self,
        network_class: u32,
        chain_id: u32,
        to_address: &str,
        amount: BigUint,
        token_standard: TokenStandard,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "WrapToken",
            &[
                AbiValue::UInt(BigUint::from(network_class)),
                AbiValue::UInt(BigUint::from(chain_id)),
                AbiValue::String(to_address.to_string()),
            ],
        );
        AccountBlockTemplate::call_contract(bridge_address(), token_standard, amount, data)
    }

    /// Builds an update-wrap-request template.
    pub fn update_wrap_request(&self, id: &Hash, signature: &str) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "UpdateWrapRequest",
            &[
                AbiValue::Hash(id.clone()),
                AbiValue::String(signature.to_string()),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a halt template.
    pub fn halt(&self, signature: &str) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "Halt",
            &[AbiValue::String(signature.to_string())],
        );
        zero_znn_template(data)
    }

    /// Builds a TSS public-key change template.
    pub fn change_tss_ecdsa_pub_key(
        &self,
        pub_key: &str,
        old_pub_key_signature: &str,
        new_pub_key_signature: &str,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "ChangeTssECDSAPubKey",
            &[
                AbiValue::String(pub_key.to_string()),
                AbiValue::String(old_pub_key_signature.to_string()),
                AbiValue::String(new_pub_key_signature.to_string()),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a redeem template.
    pub fn redeem(&self, transaction_hash: &Hash, log_index: u32) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "Redeem",
            &[
                AbiValue::Hash(transaction_hash.clone()),
                AbiValue::UInt(BigUint::from(log_index)),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds an unwrap-token template.
    #[allow(clippy::too_many_arguments)]
    pub fn unwrap_token(
        &self,
        network_class: u32,
        chain_id: u32,
        transaction_hash: &Hash,
        log_index: u32,
        to_address: Address,
        token_address: &str,
        amount: BigUint,
        signature: &str,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "UnwrapToken",
            &[
                AbiValue::UInt(BigUint::from(network_class)),
                AbiValue::UInt(BigUint::from(chain_id)),
                AbiValue::Hash(transaction_hash.clone()),
                AbiValue::UInt(BigUint::from(log_index)),
                AbiValue::Address(to_address),
                AbiValue::String(token_address.to_string()),
                AbiValue::UInt(amount),
                AbiValue::String(signature.to_string()),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a propose-administrator template.
    pub fn propose_administrator(&self, address: Address) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "ProposeAdministrator",
            &[AbiValue::Address(address)],
        );
        zero_znn_template(data)
    }

    /// Builds a set-network template.
    pub fn set_network(
        &self,
        network_class: u32,
        chain_id: u32,
        name: &str,
        contract_address: &str,
        metadata: &str,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "SetNetwork",
            &[
                AbiValue::UInt(BigUint::from(network_class)),
                AbiValue::UInt(BigUint::from(chain_id)),
                AbiValue::String(name.to_string()),
                AbiValue::String(contract_address.to_string()),
                AbiValue::String(metadata.to_string()),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a remove-network template.
    pub fn remove_network(&self, network_class: u32, chain_id: u32) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "RemoveNetwork",
            &[
                AbiValue::UInt(BigUint::from(network_class)),
                AbiValue::UInt(BigUint::from(chain_id)),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a set-token-pair template.
    #[allow(clippy::too_many_arguments)]
    pub fn set_token_pair(
        &self,
        network_class: u32,
        chain_id: u32,
        token_standard: TokenStandard,
        token_address: &str,
        bridgeable: bool,
        redeemable: bool,
        owned: bool,
        min_amount: BigUint,
        fee_percentage: u32,
        redeem_delay: u32,
        metadata: &str,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "SetTokenPair",
            &[
                AbiValue::UInt(BigUint::from(network_class)),
                AbiValue::UInt(BigUint::from(chain_id)),
                AbiValue::TokenStandard(token_standard),
                AbiValue::String(token_address.to_string()),
                AbiValue::Bool(bridgeable),
                AbiValue::Bool(redeemable),
                AbiValue::Bool(owned),
                AbiValue::UInt(min_amount),
                AbiValue::UInt(BigUint::from(fee_percentage)),
                AbiValue::UInt(BigUint::from(redeem_delay)),
                AbiValue::String(metadata.to_string()),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a set-network-metadata template.
    pub fn set_network_metadata(
        &self,
        network_class: u32,
        chain_id: u32,
        metadata: &str,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "SetNetworkMetadata",
            &[
                AbiValue::UInt(BigUint::from(network_class)),
                AbiValue::UInt(BigUint::from(chain_id)),
                AbiValue::String(metadata.to_string()),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a remove-token-pair template.
    pub fn remove_token_pair(
        &self,
        network_class: u32,
        chain_id: u32,
        token_standard: TokenStandard,
        token_address: &str,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "RemoveTokenPair",
            &[
                AbiValue::UInt(BigUint::from(network_class)),
                AbiValue::UInt(BigUint::from(chain_id)),
                AbiValue::TokenStandard(token_standard),
                AbiValue::String(token_address.to_string()),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds an unhalt template.
    pub fn unhalt(&self) -> AccountBlockTemplate {
        zero_znn_template(encode_call(BRIDGE_DEFINITION, "Unhalt", &[]))
    }

    /// Builds an emergency template.
    pub fn emergency(&self) -> AccountBlockTemplate {
        zero_znn_template(encode_call(BRIDGE_DEFINITION, "Emergency", &[]))
    }

    /// Builds a change-administrator template.
    pub fn change_administrator(&self, administrator: Address) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "ChangeAdministrator",
            &[AbiValue::Address(administrator)],
        );
        zero_znn_template(data)
    }

    /// Builds a set-allow-key-gen template.
    pub fn set_allow_key_gen(&self, allow_key_gen: bool) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "SetAllowKeyGen",
            &[AbiValue::Bool(allow_key_gen)],
        );
        zero_znn_template(data)
    }

    /// Builds a set-redeem-delay template.
    pub fn set_redeem_delay(&self, redeem_delay: u64) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "SetRedeemDelay",
            &[AbiValue::UInt(BigUint::from(redeem_delay))],
        );
        zero_znn_template(data)
    }

    /// Builds a set-bridge-metadata template.
    pub fn set_bridge_metadata(&self, metadata: &str) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "SetBridgeMetadata",
            &[AbiValue::String(metadata.to_string())],
        );
        zero_znn_template(data)
    }

    /// Builds a revoke-unwrap-request template.
    pub fn revoke_unwrap_request(
        &self,
        transaction_hash: &Hash,
        log_index: u32,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "RevokeUnwrapRequest",
            &[
                AbiValue::Hash(transaction_hash.clone()),
                AbiValue::UInt(BigUint::from(log_index)),
            ],
        );
        zero_znn_template(data)
    }

    /// Builds a nominate-guardians template.
    pub fn nominate_guardians(&self, guardians: &[Address]) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "NominateGuardians",
            &[AbiValue::Array(
                guardians.iter().cloned().map(AbiValue::Address).collect(),
            )],
        );
        zero_znn_template(data)
    }

    /// Builds a set-orchestrator-info template.
    pub fn set_orchestrator_info(
        &self,
        window_size: u64,
        key_gen_threshold: u32,
        confirmations_to_finality: u32,
        estimated_momentum_time: u32,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            BRIDGE_DEFINITION,
            "SetOrchestratorInfo",
            &[
                AbiValue::UInt(BigUint::from(window_size)),
                AbiValue::UInt(BigUint::from(key_gen_threshold)),
                AbiValue::UInt(BigUint::from(confirmations_to_finality)),
                AbiValue::UInt(BigUint::from(estimated_momentum_time)),
            ],
        );
        zero_znn_template(data)
    }
}

/// Returns the bridge contract address.
pub fn bridge_address() -> Address {
    embedded_address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d")
}

fn zero_znn_template(data: Vec<u8>) -> AccountBlockTemplate {
    AccountBlockTemplate::call_contract(
        bridge_address(),
        znn_token_standard(),
        BigUint::from(0u32),
        data,
    )
}

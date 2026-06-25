//! Integration tests for embedded contract APIs (#55–#65).
#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    clippy::unwrap_used
)]

mod support;

use num_bigint::BigUint;
use serde_json::{Value, json};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use support::{MockNode, capture_method, connect};
use znn_sdk_rust::abi::{Abi, AbiValue};
use znn_sdk_rust::api::PageQuery;
use znn_sdk_rust::api::embedded::accelerator::{AcceleratorApi, accelerator_address};
use znn_sdk_rust::api::embedded::bridge::{BridgeApi, bridge_address};
use znn_sdk_rust::api::embedded::htlc::{HtlcApi, htlc_address};
use znn_sdk_rust::api::embedded::liquidity::{LiquidityApi, liquidity_address};
use znn_sdk_rust::api::embedded::pillar::{PillarApi, pillar_address};
use znn_sdk_rust::api::embedded::plasma::{PlasmaApi, plasma_address};
use znn_sdk_rust::api::embedded::sentinel::{SentinelApi, sentinel_address};
use znn_sdk_rust::api::embedded::spork::{SporkApi, spork_address};
use znn_sdk_rust::api::embedded::stake::{StakeApi, stake_address};
use znn_sdk_rust::api::embedded::swap::{SwapApi, swap_address};
use znn_sdk_rust::api::embedded::token::{TokenApi, token_address};
use znn_sdk_rust::embedded::constants::{
    GENESIS_TIMESTAMP, PILLAR_REGISTER_ZNN_AMOUNT, PROJECT_CREATION_FEE_IN_ZNN,
    SENTINEL_REGISTER_ZNN_AMOUNT, SWAP_ASSET_DECAY_EPOCHS_OFFSET, SWAP_ASSET_DECAY_TICK_EPOCHS,
    SWAP_ASSET_DECAY_TICK_VALUE_PERCENTAGE, SWAP_ASSET_DECAY_TIMESTAMP_START,
    TOKEN_ZTS_ISSUE_FEE_IN_ZNN,
};
use znn_sdk_rust::embedded::definitions::{
    ACCELERATOR_DEFINITION, BRIDGE_DEFINITION, LIQUIDITY_DEFINITION, SWAP_DEFINITION,
};
use znn_sdk_rust::model::embedded::accelerator::{Phase, Project, ProjectList};
use znn_sdk_rust::model::embedded::bridge::{
    BridgeInfo, BridgeNetworkInfoList, OrchestratorInfo, TokenPair, UnwrapTokenRequest,
    WrapTokenRequest,
};
use znn_sdk_rust::model::embedded::common::{
    PillarVote, RewardDeposit, RewardHistoryList, SecurityInfo, UncollectedReward, VoteBreakdown,
};
use znn_sdk_rust::model::embedded::htlc::HtlcInfo;
use znn_sdk_rust::model::embedded::liquidity::{LiquidityInfo, LiquidityStakeList, TokenTuple};
use znn_sdk_rust::model::embedded::pillar::{DelegationInfo, PillarInfo, PillarInfoList};
use znn_sdk_rust::model::embedded::plasma::{
    FusionEntryList, GetRequiredParam, GetRequiredResponse, PlasmaInfo,
};
use znn_sdk_rust::model::embedded::sentinel::SentinelInfoList;
use znn_sdk_rust::model::embedded::spork::SporkList;
use znn_sdk_rust::model::embedded::stake::StakeList;
use znn_sdk_rust::model::embedded::swap::{SwapAssetEntry, SwapLegacyPillarEntry};
use znn_sdk_rust::model::nom::account_block_template::AccountBlockTemplate;
use znn_sdk_rust::model::nom::token::{Token, TokenList};
use znn_sdk_rust::primitives::address::Address;
use znn_sdk_rust::primitives::hash::Hash;
use znn_sdk_rust::primitives::token_standard::{qsr_token_standard, znn_token_standard};
use znn_sdk_rust::utils::nom_constants::ONE_QSR;

const ACCELERATOR: &str = include_str!("conformance/embedded/accelerator.json");
const BRIDGE: &str = include_str!("conformance/embedded/bridge.json");
const COMMON: &str = include_str!("conformance/embedded/common.json");
const HTLC: &str = include_str!("conformance/embedded/htlc.json");
const LIQUIDITY: &str = include_str!("conformance/embedded/liquidity.json");
const PILLAR: &str = include_str!("conformance/embedded/pillar.json");
const PLASMA: &str = include_str!("conformance/embedded/plasma.json");
const SENTINEL: &str = include_str!("conformance/embedded/sentinel.json");
const SPORK: &str = include_str!("conformance/embedded/spork.json");
const STAKE: &str = include_str!("conformance/embedded/stake.json");
const SWAP: &str = include_str!("conformance/embedded/swap.json");
const TOKEN: &str = include_str!("conformance/nom/token.json");

fn fixture(source: &str, key: &str) -> Value {
    serde_json::from_str::<Value>(source).expect("fixture parses")[key].clone()
}

fn address(s: &str) -> Address {
    Address::parse(s).expect("canonical embedded address parses")
}

fn sample_address() -> Address {
    Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").expect("address parses")
}

fn hash() -> Hash {
    Hash::parse("c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9")
        .expect("hash parses")
}

fn selector_bytes(signature: &str) -> [u8; 4] {
    let digest = Sha3_256::digest(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

fn assert_data_selector(data: &[u8], signature: &str) {
    assert_eq!(
        data.get(0..4),
        Some(&selector_bytes(signature)[..]),
        "template data must start with selector for {signature}"
    );
}

fn assert_templates_target_and_encode(cases: Vec<(&str, AccountBlockTemplate, Address, &str)>) {
    let mismatches = cases
        .into_iter()
        .filter_map(|(name, template, expected_address, signature)| {
            let selector = selector_bytes(signature);
            let address_matches = *template.to_address() == expected_address;
            let selector_matches = template.data().get(0..4) == Some(&selector[..]);
            (!address_matches || !selector_matches).then(|| {
                format!(
                    "{name}: address={}, expected_address={}, selector={:?}, expected_selector={:?}",
                    template.to_address(),
                    expected_address,
                    template.data().get(0..4),
                    &selector[..]
                )
            })
        })
        .collect::<Vec<_>>();
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

fn decode_template_args(definition: &str, data: &[u8]) -> Vec<AbiValue> {
    let abi_json = serde_json::from_str::<Value>(definition).expect("definition JSON parses");
    Abi::from_json(&abi_json)
        .expect("definition parses")
        .decode_function(data)
        .expect("template data decodes")
}

#[test]
fn embedded_address_accessors_return_canonical_embedded_addresses() {
    let cases = [
        (
            "accelerator",
            accelerator_address(),
            "z1qxemdeddedxaccelerat0rxxxxxxxxxxp4tk22",
        ),
        (
            "bridge",
            bridge_address(),
            "z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d",
        ),
        (
            "htlc",
            htlc_address(),
            "z1qxemdeddedxhtlcxxxxxxxxxxxxxxxxxygecvw",
        ),
        (
            "liquidity",
            liquidity_address(),
            "z1qxemdeddedxlyquydytyxxxxxxxxxxxxflaaae",
        ),
        (
            "pillar",
            pillar_address(),
            "z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg",
        ),
        (
            "plasma",
            plasma_address(),
            "z1qxemdeddedxplasmaxxxxxxxxxxxxxxxxsctrp",
        ),
        (
            "sentinel",
            sentinel_address(),
            "z1qxemdeddedxsentynelxxxxxxxxxxxxxwy0r2r",
        ),
        (
            "spork",
            spork_address(),
            "z1qxemdeddedxsp0rkxxxxxxxxxxxxxxxx956u48",
        ),
        (
            "stake",
            stake_address(),
            "z1qxemdeddedxstakexxxxxxxxxxxxxxxxjv8v62",
        ),
        (
            "swap",
            swap_address(),
            "z1qxemdeddedxswapxxxxxxxxxxxxxxxxxxl4yww",
        ),
        (
            "token",
            token_address(),
            "z1qxemdeddedxt0kenxxxxxxxxxxxxxxxxh9amk0",
        ),
    ];
    let mismatches = cases
        .into_iter()
        .filter_map(|(name, actual, expected)| {
            let expected = address(expected);
            (actual != expected || !actual.is_embedded()).then(|| {
                format!(
                    "{name}: got {actual}, expected {expected}, embedded={}",
                    actual.is_embedded()
                )
            })
        })
        .collect::<Vec<_>>();
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

#[tokio::test]
async fn accelerator_project_list_decodes() {
    let response = fixture(ACCELERATOR, "project_list");
    let expected = ProjectList::from_json(&response).expect("project list parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.accelerator.getAll", response);
    })
    .await;
    let api = AcceleratorApi::new(connect(&node.url).await);
    assert_eq!(api.get_all(PageQuery::default()).await, Ok(expected));
}

#[tokio::test]
async fn accelerator_pillar_votes_preserve_null_entries() {
    let vote = PillarVote::from_json(&fixture(COMMON, "pillar_vote")).expect("vote parses");
    let response = json!([fixture(COMMON, "pillar_vote"), Value::Null]);
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.accelerator.getPillarVotes", response);
    })
    .await;
    let api = AcceleratorApi::new(connect(&node.url).await);
    let result = api.get_pillar_votes("pillar", &[hash().to_string()]).await;
    assert_eq!(result, Ok(vec![Some(vote), None]));
}

#[tokio::test]
async fn accelerator_project_by_id_decodes() {
    let response = fixture(ACCELERATOR, "project");
    let expected = Project::from_json(&response).expect("project parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.accelerator.getProjectById", response);
    })
    .await;
    let api = AcceleratorApi::new(connect(&node.url).await);
    assert_eq!(api.get_project_by_id("project-id").await, Ok(expected));
}

#[tokio::test]
async fn accelerator_phase_by_hash_decodes() {
    let response = fixture(ACCELERATOR, "phase");
    let expected = Phase::from_json(&response).expect("phase parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.accelerator.getPhaseByHash", response);
    })
    .await;
    let api = AcceleratorApi::new(connect(&node.url).await);
    assert_eq!(api.get_phase_by_hash(&hash()).await, Ok(expected));
}

#[tokio::test]
async fn accelerator_vote_breakdown_decodes() {
    let response = fixture(COMMON, "vote_breakdown");
    let expected = VoteBreakdown::from_json(&response).expect("vote breakdown parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.accelerator.getVoteBreakdown", response);
    })
    .await;
    let api = AcceleratorApi::new(connect(&node.url).await);
    assert_eq!(api.get_vote_breakdown(&hash()).await, Ok(expected));
}

#[test]
fn accelerator_donate_builder_targets_contract_and_selector() {
    let api = AcceleratorApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let donate = api.donate(123u32.into(), qsr_token_standard());
    assert_data_selector(donate.data(), "Donate()");
    assert_eq!(
        *donate.to_address(),
        address("z1qxemdeddedxaccelerat0rxxxxxxxxxxp4tk22")
    );
    assert_eq!(*donate.amount(), BigUint::from(123u32));
    assert_eq!(*donate.token_standard(), qsr_token_standard());
}

#[test]
fn accelerator_builders_target_contract_and_selector() {
    let api = AcceleratorApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let create = api.create_project("p", "d", "zenon.network", 10u32.into(), 20u32.into());
    assert_data_selector(
        create.data(),
        "CreateProject(string,string,string,uint256,uint256)",
    );
    assert_eq!(
        *create.to_address(),
        address("z1qxemdeddedxaccelerat0rxxxxxxxxxxp4tk22")
    );
    assert_eq!(*create.amount(), BigUint::from(PROJECT_CREATION_FEE_IN_ZNN));
    assert_eq!(*create.token_standard(), znn_token_standard());

    let donate = api.donate(123u32.into(), qsr_token_standard());
    assert_data_selector(donate.data(), "Donate()");
    assert_eq!(*donate.amount(), BigUint::from(123u32));
    assert_eq!(*donate.token_standard(), qsr_token_standard());
}

#[test]
fn accelerator_add_phase_and_vote_by_prod_address_encode_arguments() {
    let api = AcceleratorApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let id = hash();
    let add_phase = api.add_phase(
        &id,
        "phase",
        "description",
        "zenon.network",
        10u32.into(),
        20u32.into(),
    );
    assert_eq!(
        decode_template_args(ACCELERATOR_DEFINITION, add_phase.data()),
        vec![
            AbiValue::Hash(id.clone()),
            AbiValue::String("phase".to_string()),
            AbiValue::String("description".to_string()),
            AbiValue::String("zenon.network".to_string()),
            AbiValue::UInt(10u32.into()),
            AbiValue::UInt(20u32.into()),
        ]
    );

    let vote = api.vote_by_prod_address(&id, 1);
    assert_eq!(
        decode_template_args(ACCELERATOR_DEFINITION, vote.data()),
        vec![AbiValue::Hash(id), AbiValue::UInt(1u32.into())]
    );
}

#[tokio::test]
async fn bridge_info_and_security_info_decode() {
    let bridge_value = fixture(BRIDGE, "bridge_info");
    let security_value = fixture(COMMON, "security_info");
    let expected_bridge = BridgeInfo::from_json(&bridge_value).expect("bridge info parses");
    let expected_security = SecurityInfo::from_json(&security_value).expect("security info parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.bridge.getBridgeInfo", bridge_value);
        capture_method(module, "embedded.bridge.getSecurityInfo", security_value);
    })
    .await;
    let api = BridgeApi::new(connect(&node.url).await);
    assert_eq!(api.get_bridge_info().await, Ok(expected_bridge));
    assert_eq!(api.get_security_info().await, Ok(expected_security));
}

#[tokio::test]
async fn bridge_orchestrator_info_decodes() {
    let response = fixture(BRIDGE, "orchestrator_info");
    let expected = OrchestratorInfo::from_json(&response).expect("orchestrator info parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.bridge.getOrchestratorInfo", response);
    })
    .await;
    let api = BridgeApi::new(connect(&node.url).await);
    assert_eq!(api.get_orchestrator_info().await, Ok(expected));
}

#[tokio::test]
async fn bridge_all_networks_decode() {
    let response = fixture(BRIDGE, "bridge_network_info_list");
    let expected = BridgeNetworkInfoList::from_json(&response).expect("network list parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.bridge.getAllNetworks", response);
    })
    .await;
    let api = BridgeApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_all_networks(PageQuery::default()).await,
        Ok(expected)
    );
}

#[tokio::test]
async fn bridge_wrap_request_by_hash_decodes() {
    let response = fixture(BRIDGE, "wrap_token_request");
    let expected = WrapTokenRequest::from_json(&response).expect("wrap request parses");
    let node = MockNode::spawn(|module| {
        capture_method(
            module,
            "embedded.bridge.getWrapTokenRequestByHash",
            response,
        );
    })
    .await;
    let api = BridgeApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_wrap_token_request_by_hash(&hash()).await,
        Ok(expected)
    );
}

#[tokio::test]
async fn bridge_fee_token_pair_decodes() {
    let response = fixture(BRIDGE, "token_pair");
    let expected = TokenPair::from_json(&response).expect("token pair parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.bridge.getFeeTokenPair", response);
    })
    .await;
    let api = BridgeApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_fee_token_pair(&znn_token_standard()).await,
        Ok(expected)
    );
}

#[tokio::test]
async fn bridge_unwrap_request_by_hash_and_log_dispatches_params_and_decodes() {
    let response = fixture(BRIDGE, "unwrap_token_request");
    let expected = UnwrapTokenRequest::from_json(&response).expect("unwrap request parses");
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        captured = Some(capture_method(
            module,
            "embedded.bridge.getUnwrapTokenRequestByHashAndLog",
            response,
        ));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = BridgeApi::new(connect(&node.url).await);
    let result = api
        .get_unwrap_token_request_by_hash_and_log(&hash(), 7)
        .await;
    assert_eq!(
        captured.lock().expect("params cell").clone(),
        vec![json!(hash().to_string()), json!(7)]
    );
    assert_eq!(result, Ok(expected));
}

#[test]
fn bridge_builders_target_contract_and_selector() {
    let api = BridgeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let wrapped = api.wrap_token(1, 56, "0xabc", 99u32.into(), znn_token_standard());
    assert_data_selector(wrapped.data(), "WrapToken(uint32,uint32,string)");
    assert_eq!(
        *wrapped.to_address(),
        address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d")
    );
    assert_eq!(*wrapped.amount(), BigUint::from(99u32));
    assert_eq!(*wrapped.token_standard(), znn_token_standard());

    let nominated = api.nominate_guardians(&[sample_address()]);
    assert_data_selector(nominated.data(), "NominateGuardians(address[])");
    assert_eq!(
        *nominated.to_address(),
        address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d")
    );
}

#[test]
fn bridge_nominate_guardians_builder_targets_contract_and_selector() {
    let api = BridgeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let nominated = api.nominate_guardians(&[sample_address()]);
    assert_data_selector(nominated.data(), "NominateGuardians(address[])");
    assert_eq!(
        *nominated.to_address(),
        address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d")
    );
}

#[test]
fn bridge_unwrap_token_builder_targets_contract_and_selector() {
    let api = BridgeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.unwrap_token(
        1,
        56,
        &hash(),
        7,
        sample_address(),
        "0xabc",
        99u32.into(),
        "sig",
    );
    assert_eq!(
        decode_template_args(BRIDGE_DEFINITION, template.data()),
        vec![
            AbiValue::UInt(1u32.into()),
            AbiValue::UInt(56u32.into()),
            AbiValue::Hash(hash()),
            AbiValue::UInt(7u32.into()),
            AbiValue::Address(sample_address()),
            AbiValue::String("0xabc".to_string()),
            AbiValue::UInt(99u32.into()),
            AbiValue::String("sig".to_string()),
        ]
    );
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d")
    );
}

#[test]
fn bridge_set_token_pair_builder_targets_contract_and_selector() {
    let api = BridgeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.set_token_pair(
        1,
        56,
        znn_token_standard(),
        "0xabc",
        true,
        true,
        false,
        1u32.into(),
        10,
        20,
        "metadata",
    );
    assert_eq!(
        decode_template_args(BRIDGE_DEFINITION, template.data()),
        vec![
            AbiValue::UInt(1u32.into()),
            AbiValue::UInt(56u32.into()),
            AbiValue::TokenStandard(znn_token_standard()),
            AbiValue::String("0xabc".to_string()),
            AbiValue::Bool(true),
            AbiValue::Bool(true),
            AbiValue::Bool(false),
            AbiValue::UInt(1u32.into()),
            AbiValue::UInt(10u32.into()),
            AbiValue::UInt(20u32.into()),
            AbiValue::String("metadata".to_string()),
        ]
    );
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d")
    );
}

#[test]
fn bridge_set_redeem_delay_builder_encodes_argument() {
    let api = BridgeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.set_redeem_delay(60);

    assert_eq!(
        decode_template_args(BRIDGE_DEFINITION, template.data()),
        vec![AbiValue::UInt(60u32.into())]
    );
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d")
    );
}

#[test]
fn bridge_all_builders_target_contract_and_encode_selectors() {
    let api = BridgeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let expected = address("z1qxemdeddedxdrydgexxxxxxxxxxxxxxxmqgr0d");
    assert_templates_target_and_encode(vec![
        (
            "wrap_token",
            api.wrap_token(1, 56, "0xabc", 1u32.into(), znn_token_standard()),
            expected.clone(),
            "WrapToken(uint32,uint32,string)",
        ),
        (
            "update_wrap_request",
            api.update_wrap_request(&hash(), "sig"),
            expected.clone(),
            "UpdateWrapRequest(hash,string)",
        ),
        ("halt", api.halt("sig"), expected.clone(), "Halt(string)"),
        (
            "change_tss_ecdsa_pub_key",
            api.change_tss_ecdsa_pub_key("pub", "old", "new"),
            expected.clone(),
            "ChangeTssECDSAPubKey(string,string,string)",
        ),
        (
            "redeem",
            api.redeem(&hash(), 7),
            expected.clone(),
            "Redeem(hash,uint32)",
        ),
        (
            "unwrap_token",
            api.unwrap_token(
                1,
                56,
                &hash(),
                7,
                sample_address(),
                "0xabc",
                1u32.into(),
                "sig",
            ),
            expected.clone(),
            "UnwrapToken(uint32,uint32,hash,uint32,address,string,uint256,string)",
        ),
        (
            "propose_administrator",
            api.propose_administrator(sample_address()),
            expected.clone(),
            "ProposeAdministrator(address)",
        ),
        (
            "set_network",
            api.set_network(1, 56, "n", "0xabc", "m"),
            expected.clone(),
            "SetNetwork(uint32,uint32,string,string,string)",
        ),
        (
            "remove_network",
            api.remove_network(1, 56),
            expected.clone(),
            "RemoveNetwork(uint32,uint32)",
        ),
        (
            "set_token_pair",
            api.set_token_pair(
                1,
                56,
                znn_token_standard(),
                "0xabc",
                true,
                true,
                false,
                1u32.into(),
                10,
                20,
                "m",
            ),
            expected.clone(),
            "SetTokenPair(uint32,uint32,tokenStandard,string,bool,bool,bool,uint256,uint32,uint32,string)",
        ),
        (
            "set_network_metadata",
            api.set_network_metadata(1, 56, "m"),
            expected.clone(),
            "SetNetworkMetadata(uint32,uint32,string)",
        ),
        (
            "remove_token_pair",
            api.remove_token_pair(1, 56, znn_token_standard(), "0xabc"),
            expected.clone(),
            "RemoveTokenPair(uint32,uint32,tokenStandard,string)",
        ),
        ("unhalt", api.unhalt(), expected.clone(), "Unhalt()"),
        (
            "emergency",
            api.emergency(),
            expected.clone(),
            "Emergency()",
        ),
        (
            "change_administrator",
            api.change_administrator(sample_address()),
            expected.clone(),
            "ChangeAdministrator(address)",
        ),
        (
            "set_allow_key_gen",
            api.set_allow_key_gen(true),
            expected.clone(),
            "SetAllowKeyGen(bool)",
        ),
        (
            "set_redeem_delay",
            api.set_redeem_delay(60),
            expected.clone(),
            "SetRedeemDelay(uint64)",
        ),
        (
            "set_bridge_metadata",
            api.set_bridge_metadata("m"),
            expected.clone(),
            "SetBridgeMetadata(string)",
        ),
        (
            "revoke_unwrap_request",
            api.revoke_unwrap_request(&hash(), 7),
            expected.clone(),
            "RevokeUnwrapRequest(hash,uint32)",
        ),
        (
            "nominate_guardians",
            api.nominate_guardians(&[sample_address()]),
            expected.clone(),
            "NominateGuardians(address[])",
        ),
        (
            "set_orchestrator_info",
            api.set_orchestrator_info(5, 3, 10, 5),
            expected,
            "SetOrchestratorInfo(uint64,uint32,uint32,uint32)",
        ),
    ]);
}

#[tokio::test]
async fn htlc_info_and_proxy_status_decode() {
    let info = fixture(HTLC, "htlc_info");
    let expected = HtlcInfo::from_json(&info).expect("htlc info parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.htlc.getByHash", info);
        capture_method(module, "embedded.htlc.getProxyUnlockStatus", json!(true));
    })
    .await;
    let api = HtlcApi::new(connect(&node.url).await);
    assert_eq!(api.get_by_hash(&hash()).await, Ok(expected));
    assert_eq!(
        api.get_proxy_unlock_status(&sample_address()).await,
        Ok(true)
    );
}

#[test]
fn htlc_builders_target_contract_and_selector() {
    let api = HtlcApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let token = Token::from_json(&fixture(TOKEN, "token")).expect("token parses");
    let created = api.create(
        &token,
        55u32.into(),
        sample_address(),
        123,
        0,
        32,
        vec![1, 2, 3],
    );
    assert_data_selector(created.data(), "Create(address,int64,uint8,uint8,bytes)");
    assert_eq!(
        *created.to_address(),
        address("z1qxemdeddedxhtlcxxxxxxxxxxxxxxxxxygecvw")
    );
    assert_eq!(*created.amount(), BigUint::from(55u32));
    assert_eq!(created.token_standard(), token.token_standard());

    let unlocked = api.unlock(&hash(), vec![1, 2, 3]);
    assert_data_selector(unlocked.data(), "Unlock(hash,bytes)");
}

#[test]
fn htlc_unlock_builder_targets_contract_and_selector() {
    let api = HtlcApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let unlocked = api.unlock(&hash(), vec![1, 2, 3]);
    assert_data_selector(unlocked.data(), "Unlock(hash,bytes)");
    assert_eq!(
        *unlocked.to_address(),
        address("z1qxemdeddedxhtlcxxxxxxxxxxxxxxxxxygecvw")
    );
}

#[tokio::test]
async fn liquidity_info_uncollected_and_stake_entries_decode() {
    let info = fixture(LIQUIDITY, "liquidity_info");
    let reward = fixture(COMMON, "reward_deposit");
    let list = fixture(LIQUIDITY, "liquidity_stake_list");
    let expected_info = LiquidityInfo::from_json(&info).expect("liquidity info parses");
    let expected_reward = RewardDeposit::from_json(&reward).expect("reward parses");
    let expected_list = LiquidityStakeList::from_json(&list).expect("stake list parses");
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.liquidity.getLiquidityInfo", info);
        capture_method(module, "embedded.liquidity.getUncollectedReward", reward);
        captured = Some(capture_method(
            module,
            "embedded.liquidity.getLiquidityStakeEntriesByAddress",
            list,
        ));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = LiquidityApi::new(connect(&node.url).await);
    assert_eq!(api.get_liquidity_info().await, Ok(expected_info));
    assert_eq!(
        api.get_uncollected_reward(&sample_address()).await,
        Ok(expected_reward)
    );
    assert_eq!(
        api.get_liquidity_stake_entries_by_address(&sample_address(), PageQuery::mempool())
            .await,
        Ok(expected_list)
    );
    assert_eq!(
        captured.lock().expect("params cell").clone(),
        vec![
            json!(sample_address().to_string()),
            json!(0),
            json!(znn_sdk_rust::client::constants::MEMORY_POOL_PAGE_SIZE)
        ]
    );
}

#[test]
fn liquidity_stake_carries_amount_token_and_selector() {
    let api = LiquidityApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.liquidity_stake(60, 44u32.into(), qsr_token_standard());
    assert_data_selector(template.data(), "LiquidityStake(int64)");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxlyquydytyxxxxxxxxxxxxflaaae")
    );
    assert_eq!(*template.amount(), BigUint::from(44u32));
    assert_eq!(*template.token_standard(), qsr_token_standard());
}

#[test]
fn liquidity_set_token_tuple_builder_targets_contract_and_selector() {
    let api = LiquidityApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let tuple = TokenTuple::new(znn_token_standard(), 10, 20, 1u32.into());
    let template = api.set_token_tuple(tuple);
    assert_eq!(
        decode_template_args(LIQUIDITY_DEFINITION, template.data()),
        vec![
            AbiValue::Array(vec![AbiValue::String(znn_token_standard().to_string())]),
            AbiValue::Array(vec![AbiValue::UInt(10u32.into())]),
            AbiValue::Array(vec![AbiValue::UInt(20u32.into())]),
            AbiValue::Array(vec![AbiValue::UInt(1u32.into())]),
        ]
    );
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxlyquydytyxxxxxxxxxxxxflaaae")
    );
}

#[test]
fn liquidity_collect_reward_builder_targets_contract_and_selector() {
    let api = LiquidityApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.collect_reward();
    assert_data_selector(template.data(), "CollectReward()");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxlyquydytyxxxxxxxxxxxxflaaae")
    );
}

#[test]
fn liquidity_all_builders_target_contract_and_encode_selectors() {
    let api = LiquidityApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let expected = address("z1qxemdeddedxlyquydytyxxxxxxxxxxxxflaaae");
    assert_templates_target_and_encode(vec![
        (
            "liquidity_stake",
            api.liquidity_stake(60, 1u32.into(), qsr_token_standard()),
            expected.clone(),
            "LiquidityStake(int64)",
        ),
        (
            "cancel_liquidity_stake",
            api.cancel_liquidity_stake(&hash()),
            expected.clone(),
            "CancelLiquidityStake(hash)",
        ),
        (
            "unlock_liquidity_stake_entries",
            api.unlock_liquidity_stake_entries(&[hash()]),
            expected.clone(),
            "UnlockLiquidityStakeEntries()",
        ),
        (
            "set_token_tuple",
            api.set_token_tuple(TokenTuple::new(znn_token_standard(), 10, 20, 1u32.into())),
            expected.clone(),
            "SetTokenTuple(string[],uint32[],uint32[],uint256[])",
        ),
        (
            "nominate_guardians",
            api.nominate_guardians(&[sample_address()]),
            expected.clone(),
            "NominateGuardians(address[])",
        ),
        (
            "propose_administrator",
            api.propose_administrator(sample_address()),
            expected.clone(),
            "ProposeAdministrator(address)",
        ),
        (
            "set_is_halted",
            api.set_is_halted(true),
            expected.clone(),
            "SetIsHalted(bool)",
        ),
        (
            "set_additional_reward",
            api.set_additional_reward(1u32.into(), 2u32.into()),
            expected.clone(),
            "SetAdditionalReward(uint256,uint256)",
        ),
        (
            "change_administrator",
            api.change_administrator(sample_address()),
            expected.clone(),
            "ChangeAdministrator(address)",
        ),
        (
            "collect_reward",
            api.collect_reward(),
            expected.clone(),
            "CollectReward()",
        ),
        ("emergency", api.emergency(), expected, "Emergency()"),
    ]);
}

#[tokio::test]
async fn pillar_deposited_qsr_and_nullable_reads_decode() {
    let node = MockNode::spawn(|module| {
        capture_method(
            module,
            "embedded.pillar.getDepositedQsr",
            json!("10000000000"),
        );
        capture_method(module, "embedded.pillar.checkNameAvailability", json!(true));
        capture_method(module, "embedded.pillar.getByName", Value::Null);
    })
    .await;
    let api = PillarApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_deposited_qsr(&sample_address()).await,
        Ok(BigUint::parse_bytes(b"10000000000", 10).unwrap())
    );
    assert_eq!(api.check_name_availability("pillar").await, Ok(true));
    assert_eq!(api.get_by_name("pillar").await, Ok(None::<PillarInfo>));
}

#[tokio::test]
async fn pillar_all_and_delegated_reads_decode() {
    let list = fixture(PILLAR, "pillar_info_list");
    let delegated = fixture(PILLAR, "delegation_info_active");
    let expected_list = PillarInfoList::from_json(&list).expect("pillar list parses");
    let expected_delegated = DelegationInfo::from_json(&delegated).expect("delegation parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.pillar.getAll", list);
        capture_method(module, "embedded.pillar.getDelegatedPillar", delegated);
    })
    .await;
    let api = PillarApi::new(connect(&node.url).await);
    assert_eq!(api.get_all(PageQuery::default()).await, Ok(expected_list));
    assert_eq!(
        api.get_delegated_pillar(&sample_address()).await,
        Ok(Some(expected_delegated))
    );
}

#[test]
fn pillar_register_pays_registration_amount_and_selector() {
    let api = PillarApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.register("pillar", sample_address(), sample_address(), 10, 20);
    assert_data_selector(
        template.data(),
        "Register(string,address,address,uint8,uint8)",
    );
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg")
    );
    assert_eq!(
        *template.amount(),
        BigUint::from(PILLAR_REGISTER_ZNN_AMOUNT)
    );
}

#[test]
fn pillar_collect_reward_builder_targets_contract_and_selector() {
    let api = PillarApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.collect_reward();
    assert_data_selector(template.data(), "CollectReward()");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg")
    );
}

#[test]
fn pillar_all_builders_target_contract_and_encode_selectors() {
    let api = PillarApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let expected = address("z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg");
    assert_templates_target_and_encode(vec![
        (
            "register",
            api.register("pillar", sample_address(), sample_address(), 10, 20),
            expected.clone(),
            "Register(string,address,address,uint8,uint8)",
        ),
        (
            "register_legacy",
            api.register_legacy("pillar", sample_address(), sample_address(), 10, 20),
            expected.clone(),
            "RegisterLegacy(string,address,address,uint8,uint8)",
        ),
        (
            "update_pillar",
            api.update_pillar("pillar", sample_address(), sample_address(), 10, 20),
            expected.clone(),
            "UpdatePillar(string,address,address,uint8,uint8)",
        ),
        (
            "revoke",
            api.revoke("pillar"),
            expected.clone(),
            "Revoke(string)",
        ),
        (
            "delegate",
            api.delegate("pillar"),
            expected.clone(),
            "Delegate(string)",
        ),
        (
            "undelegate",
            api.undelegate(),
            expected.clone(),
            "Undelegate()",
        ),
        (
            "collect_reward",
            api.collect_reward(),
            expected.clone(),
            "CollectReward()",
        ),
        (
            "deposit_qsr",
            api.deposit_qsr(1u32.into()),
            expected.clone(),
            "DepositQsr()",
        ),
        (
            "withdraw_qsr",
            api.withdraw_qsr(),
            expected,
            "WithdrawQsr()",
        ),
    ]);
}

#[tokio::test]
async fn plasma_info_and_required_pow_decode() {
    let info = fixture(PLASMA, "plasma_info");
    let param_value = fixture(PLASMA, "get_required_param");
    let response = fixture(PLASMA, "get_required_response");
    let expected_info = PlasmaInfo::from_json(&info).expect("plasma info parses");
    let param = GetRequiredParam::from_json(&param_value).expect("param parses");
    let expected_response = GetRequiredResponse::from_json(&response).expect("response parses");
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.plasma.get", info);
        captured = Some(capture_method(
            module,
            "embedded.plasma.getRequiredPoWForAccountBlock",
            response,
        ));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = PlasmaApi::new(connect(&node.url).await);
    assert_eq!(api.get(&sample_address()).await, Ok(expected_info));
    assert_eq!(
        api.get_required_pow_for_account_block(&param).await,
        Ok(expected_response)
    );
    assert_eq!(
        captured.lock().expect("params cell").clone(),
        vec![param.to_json()]
    );
}

#[tokio::test]
async fn plasma_entries_and_required_fusion_amount_decode() {
    let list = fixture(PLASMA, "fusion_entry_list");
    let expected_list = FusionEntryList::from_json(&list).expect("fusion list parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.plasma.getEntriesByAddress", list);
        capture_method(
            module,
            "embedded.plasma.getRequiredFusionAmount",
            json!(12345),
        );
    })
    .await;
    let api = PlasmaApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_entries_by_address(&sample_address(), PageQuery::default())
            .await,
        Ok(expected_list)
    );
    assert_eq!(api.get_required_fusion_amount(21000).await, Ok(12345));
}

#[test]
fn plasma_helper_and_fuse_are_contract_correct() {
    assert_eq!(
        PlasmaApi::<znn_sdk_rust::client::websocket::WsClient>::plasma_by_qsr(&BigUint::from(
            ONE_QSR
        )),
        BigUint::from(2100u64) * ONE_QSR
    );
    let api = PlasmaApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.fuse(sample_address(), 77u32.into());
    assert_data_selector(template.data(), "Fuse(address)");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxplasmaxxxxxxxxxxxxxxxxsctrp")
    );
    assert_eq!(*template.amount(), BigUint::from(77u32));
    assert_eq!(*template.token_standard(), qsr_token_standard());
}

#[test]
fn plasma_cancel_builder_targets_contract_and_selector() {
    let api = PlasmaApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.cancel(&hash());
    assert_data_selector(template.data(), "Cancel(hash)");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxplasmaxxxxxxxxxxxxxxxxsctrp")
    );
}

#[tokio::test]
async fn sentinel_active_and_nullable_owner_decode() {
    let list = fixture(SENTINEL, "sentinel_info_list");
    let expected = SentinelInfoList::from_json(&list).expect("sentinel list parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.sentinel.getAllActive", list);
        capture_method(module, "embedded.sentinel.getByOwner", Value::Null);
    })
    .await;
    let api = SentinelApi::new(connect(&node.url).await);
    assert_eq!(api.get_all_active(PageQuery::default()).await, Ok(expected));
    assert_eq!(api.get_by_owner(&sample_address()).await, Ok(None));
}

#[tokio::test]
async fn sentinel_uncollected_reward_decodes() {
    let reward = fixture(COMMON, "uncollected_reward");
    let expected = UncollectedReward::from_json(&reward).expect("uncollected reward parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.sentinel.getUncollectedReward", reward);
    })
    .await;
    let api = SentinelApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_uncollected_reward(&sample_address()).await,
        Ok(expected)
    );
}

#[test]
fn sentinel_register_pays_registration_amount() {
    let api = SentinelApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.register();
    assert_data_selector(template.data(), "Register()");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxsentynelxxxxxxxxxxxxxwy0r2r")
    );
    assert_eq!(
        *template.amount(),
        BigUint::from(SENTINEL_REGISTER_ZNN_AMOUNT)
    );
}

#[test]
fn sentinel_collect_reward_builder_targets_contract_and_selector() {
    let api = SentinelApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.collect_reward();
    assert_data_selector(template.data(), "CollectReward()");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxsentynelxxxxxxxxxxxxxwy0r2r")
    );
}

#[test]
fn sentinel_all_builders_target_contract_and_encode_selectors() {
    let api = SentinelApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let expected = address("z1qxemdeddedxsentynelxxxxxxxxxxxxxwy0r2r");
    assert_templates_target_and_encode(vec![
        ("register", api.register(), expected.clone(), "Register()"),
        ("revoke", api.revoke(), expected.clone(), "Revoke()"),
        (
            "collect_reward",
            api.collect_reward(),
            expected.clone(),
            "CollectReward()",
        ),
        (
            "deposit_qsr",
            api.deposit_qsr(1u32.into()),
            expected.clone(),
            "DepositQsr()",
        ),
        (
            "withdraw_qsr",
            api.withdraw_qsr(),
            expected,
            "WithdrawQsr()",
        ),
    ]);
}

#[tokio::test]
async fn spork_list_decodes() {
    let list = fixture(SPORK, "spork_list");
    let expected = SporkList::from_json(&list).expect("spork list parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.spork.getAll", list);
    })
    .await;
    let api = SporkApi::new(connect(&node.url).await);
    assert_eq!(api.get_all(PageQuery::default()).await, Ok(expected));
}

#[test]
fn spork_create_targets_contract_and_selector() {
    let api = SporkApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.create_spork("spork", "description");
    assert_data_selector(template.data(), "CreateSpork(string,string)");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxsp0rkxxxxxxxxxxxxxxxx956u48")
    );
    assert_eq!(*template.amount(), BigUint::from(0u32));
}

#[test]
fn spork_activate_targets_contract_and_selector() {
    let api = SporkApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.activate_spork(&hash());
    assert_data_selector(template.data(), "ActivateSpork(hash)");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxsp0rkxxxxxxxxxxxxxxxx956u48")
    );
}

#[tokio::test]
async fn stake_entries_decode() {
    let list = fixture(STAKE, "stake_list");
    let expected = StakeList::from_json(&list).expect("stake list parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.stake.getEntriesByAddress", list);
    })
    .await;
    let api = StakeApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_entries_by_address(&sample_address(), PageQuery::default())
            .await,
        Ok(expected)
    );
}

#[tokio::test]
async fn stake_reward_reads_decode() {
    let reward = fixture(COMMON, "uncollected_reward");
    let history = fixture(COMMON, "reward_history_list");
    let expected_reward = UncollectedReward::from_json(&reward).expect("uncollected reward parses");
    let expected_history = RewardHistoryList::from_json(&history).expect("reward history parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.stake.getUncollectedReward", reward);
        capture_method(module, "embedded.stake.getFrontierRewardByPage", history);
    })
    .await;
    let api = StakeApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_uncollected_reward(&sample_address()).await,
        Ok(expected_reward)
    );
    assert_eq!(
        api.get_frontier_reward_by_page(&sample_address(), PageQuery::default())
            .await,
        Ok(expected_history)
    );
}

#[test]
fn stake_builder_carries_amount_duration_and_selector() {
    let api = StakeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.stake(30, 11u32.into());
    assert_data_selector(template.data(), "Stake(int64)");
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxstakexxxxxxxxxxxxxxxxjv8v62")
    );
    assert_eq!(*template.amount(), BigUint::from(11u32));
    assert_eq!(*template.token_standard(), znn_token_standard());
}

#[test]
fn stake_cancel_builder_targets_contract_and_selector() {
    let api = StakeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let cancel = api.cancel(&hash());
    assert_data_selector(cancel.data(), "Cancel(hash)");
    assert_eq!(
        *cancel.to_address(),
        address("z1qxemdeddedxstakexxxxxxxxxxxxxxxxjv8v62")
    );
}

#[test]
fn stake_collect_reward_builder_targets_contract_and_selector() {
    let api = StakeApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let collect = api.collect_reward();
    assert_data_selector(collect.data(), "CollectReward()");
    assert_eq!(
        *collect.to_address(),
        address("z1qxemdeddedxstakexxxxxxxxxxxxxxxxjv8v62")
    );
}

#[tokio::test]
async fn swap_asset_by_key_id_hash_decodes() {
    let entry = fixture(SWAP, "swap_asset_entry");
    let key = Hash::parse(entry["keyIdHash"].as_str().expect("hash string")).expect("hash parses");
    let expected = SwapAssetEntry::from_json(key.clone(), &entry).expect("swap asset parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.swap.getAssetsByKeyIdHash", entry);
    })
    .await;
    let api = SwapApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_assets_by_key_id_hash(&key.to_string()).await,
        Ok(expected)
    );
}

#[tokio::test]
async fn swap_assets_and_legacy_pillars_decode() {
    let entry = fixture(SWAP, "swap_asset_entry");
    let key = Hash::parse(entry["keyIdHash"].as_str().expect("hash string")).expect("hash parses");
    let expected_entry = SwapAssetEntry::from_json(key.clone(), &entry).expect("swap asset parses");
    let legacy = fixture(SWAP, "swap_legacy_pillar_entry");
    let expected_legacy = SwapLegacyPillarEntry::from_json(&legacy).expect("legacy parses");
    let node = MockNode::spawn(|module| {
        capture_method(
            module,
            "embedded.swap.getAssets",
            json!({ key.to_string(): entry }),
        );
        capture_method(module, "embedded.swap.getLegacyPillars", json!([legacy]));
    })
    .await;
    let api = SwapApi::new(connect(&node.url).await);
    let mut expected_map = HashMap::new();
    expected_map.insert(key, expected_entry);
    assert_eq!(api.get_assets().await, Ok(expected_map));
    assert_eq!(api.get_legacy_pillars().await, Ok(vec![expected_legacy]));
}

#[test]
fn swap_decay_and_retrieve_assets_are_contract_correct() {
    const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
    assert_eq!(
        SwapApi::<znn_sdk_rust::client::websocket::WsClient>::swap_decay_percentage(
            SWAP_ASSET_DECAY_TIMESTAMP_START - 1
        ),
        0
    );
    assert_eq!(
        SwapApi::<znn_sdk_rust::client::websocket::WsClient>::swap_decay_percentage(
            GENESIS_TIMESTAMP + SWAP_ASSET_DECAY_EPOCHS_OFFSET * SECONDS_PER_DAY
        ),
        0
    );
    assert_eq!(
        SwapApi::<znn_sdk_rust::client::websocket::WsClient>::swap_decay_percentage(
            GENESIS_TIMESTAMP
                + (SWAP_ASSET_DECAY_EPOCHS_OFFSET + SWAP_ASSET_DECAY_TICK_EPOCHS) * SECONDS_PER_DAY
        ),
        SWAP_ASSET_DECAY_TICK_VALUE_PERCENTAGE
    );
    assert_eq!(
        SwapApi::<znn_sdk_rust::client::websocket::WsClient>::swap_decay_percentage(
            GENESIS_TIMESTAMP
                + (SWAP_ASSET_DECAY_EPOCHS_OFFSET + 2 * SWAP_ASSET_DECAY_TICK_EPOCHS)
                    * SECONDS_PER_DAY
        ),
        2 * SWAP_ASSET_DECAY_TICK_VALUE_PERCENTAGE
    );
    assert_eq!(
        SwapApi::<znn_sdk_rust::client::websocket::WsClient>::swap_decay_percentage(
            SWAP_ASSET_DECAY_TIMESTAMP_START + 10_000_000_000
        ),
        100
    );
    let api = SwapApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let template = api.retrieve_assets("pub-key-hex", "signature-hex");
    assert_eq!(
        decode_template_args(SWAP_DEFINITION, template.data()),
        vec![
            AbiValue::String("pub-key-hex".to_string()),
            AbiValue::String("signature-hex".to_string()),
        ]
    );
    assert_eq!(
        *template.to_address(),
        address("z1qxemdeddedxswapxxxxxxxxxxxxxxxxxxl4yww")
    );
}

#[tokio::test]
async fn token_list_reads_decode() {
    let list = fixture(TOKEN, "token_list");
    let expected_all = TokenList::from_json(&list).expect("token list parses");
    let expected_owner = expected_all.clone();
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.token.getAll", list.clone());
        capture_method(module, "embedded.token.getByOwner", list);
    })
    .await;
    let api = TokenApi::new(connect(&node.url).await);
    assert_eq!(api.get_all(PageQuery::default()).await, Ok(expected_all));
    assert_eq!(
        api.get_by_owner(&sample_address(), PageQuery::default())
            .await,
        Ok(expected_owner)
    );
}

#[tokio::test]
async fn token_by_zts_is_nullable_then_decodes() {
    let token = fixture(TOKEN, "token");
    let expected = Token::from_json(&token).expect("token parses");
    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.token.getByZts", Value::Null);
    })
    .await;
    let api = TokenApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_by_zts(&znn_token_standard()).await,
        Ok(None::<Token>)
    );

    let node = MockNode::spawn(|module| {
        capture_method(module, "embedded.token.getByZts", token);
    })
    .await;
    let api = TokenApi::new(connect(&node.url).await);
    assert_eq!(
        api.get_by_zts(&znn_token_standard()).await,
        Ok(Some(expected))
    );
}

#[test]
fn token_builders_pay_or_transfer_amounts_and_selectors() {
    let api = TokenApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let issued = api.issue_token(
        "Name",
        "SYM",
        "zenon.network",
        100u32.into(),
        100u32.into(),
        8,
        true,
        true,
        true,
    );
    assert_data_selector(
        issued.data(),
        "IssueToken(string,string,string,uint256,uint256,uint8,bool,bool,bool)",
    );
    assert_eq!(
        *issued.to_address(),
        address("z1qxemdeddedxt0kenxxxxxxxxxxxxxxxxh9amk0")
    );
    assert_eq!(*issued.amount(), BigUint::from(TOKEN_ZTS_ISSUE_FEE_IN_ZNN));
    assert_eq!(*issued.token_standard(), znn_token_standard());

    let burned = api.burn_token(qsr_token_standard(), 12u32.into());
    assert_data_selector(burned.data(), "Burn()");
    assert_eq!(*burned.token_standard(), qsr_token_standard());
    assert_eq!(*burned.amount(), BigUint::from(12u32));
}

#[test]
fn token_mint_builder_targets_contract_and_selector() {
    let api = TokenApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let minted = api.mint_token(znn_token_standard(), 12u32.into(), sample_address());
    assert_data_selector(minted.data(), "Mint(tokenStandard,uint256,address)");
    assert_eq!(
        *minted.to_address(),
        address("z1qxemdeddedxt0kenxxxxxxxxxxxxxxxxh9amk0")
    );
}

#[test]
fn token_update_builder_targets_contract_and_selector() {
    let api = TokenApi::new(std::sync::Arc::new(
        znn_sdk_rust::client::websocket::WsClient::new(),
    ));
    let updated = api.update_token(znn_token_standard(), sample_address(), true, false);
    assert_data_selector(
        updated.data(),
        "UpdateToken(tokenStandard,address,bool,bool)",
    );
    assert_eq!(
        *updated.to_address(),
        address("z1qxemdeddedxt0kenxxxxxxxxxxxxxxxxh9amk0")
    );
}

#[test]
fn token_list_fixture_sanity() {
    let list = fixture(TOKEN, "token_list");
    let parsed = TokenList::from_json(&list).expect("token list parses");
    assert_eq!(parsed.count, Some(1));
}

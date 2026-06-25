//! Integration tests for the `ledger.*` JSON-RPC namespace.
//!
//! Each scenario maps to a requirement in `add-api-ledger`. The mock server
//! harness mirrors the #51 client tests.
#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]

mod support;

use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Mutex;
use support::{MockNode, ParamsCell, connect};
use znn_sdk_rust::api::PageQuery;
use znn_sdk_rust::api::ledger::LedgerApi;
use znn_sdk_rust::client::exceptions::ClientError;
use znn_sdk_rust::client::websocket::WsClient;
use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::nom::account_block::{AccountBlock, AccountBlockList};
use znn_sdk_rust::model::nom::account_block_template::{AccountBlockTemplate, BlockType};
use znn_sdk_rust::model::nom::account_info::AccountInfo;
use znn_sdk_rust::model::nom::momentum::Momentum;
use znn_sdk_rust::primitives::address::Address;

/// Records the dispatched positional params for `method` and returns `response`.
fn capturing(
    module: &mut jsonrpsee::server::RpcModule<()>,
    method: &'static str,
    response: Value,
) -> ParamsCell {
    let cell: ParamsCell = std::sync::Arc::new(Mutex::new(Vec::new()));
    let cell2 = cell.clone();
    module
        .register_method(method, move |params, _ctx, _ext| {
            let arr: Vec<Value> = params.parse().unwrap_or_default();
            *cell2.lock().unwrap() = arr;
            let value: jsonrpsee::core::RpcResult<Value> = Ok(response.clone());
            value
        })
        .expect("mock method registers");
    cell
}

/// Returns a canonical momentum object.
fn momentum_json() -> Value {
    json!({
        "version": 1,
        "chainIdentifier": 100,
        "hash": "c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9",
        "previousHash": "0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712",
        "height": 2,
        "timestamp": 1_000_000_010,
        "data": "",
        "content": [],
        "changesHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
        "publicKey": "SAPwVIVQma3zMak169crdLkcu2B2Gm3iBCdDgfQ6IxU=",
        "signature": "qvlKN6rTQgM11/FosNazpeReViuD1GH1tIww2F0zNfXruTp3g9ULhA1mYnRYAiPJyP2NlIGhENwhzBAHJ0dYBw==",
        "producer": "z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju"
    })
}

/// A canonical one-block account-block list.
fn account_block_list_json() -> Value {
    let block = json!({
        "version": 1, "chainIdentifier": 100, "blockType": 2,
        "hash": "3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73",
        "previousHash": "598fa623dd308bec7163bb375aa7546ec4aced3b71a1c9278709903e69280dbd",
        "height": 2,
        "momentumAcknowledged": { "hash": "c37c70550e95d0c72f0924d480321976040108f29fa7530487f8dde81e713689", "height": 1 },
        "address": "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz",
        "toAddress": "z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx",
        "amount": "10000000000",
        "tokenStandard": "zts1tfjkummwyppk76twsnv50e",
        "fromBlockHash": "0000000000000000000000000000000000000000000000000000000000000000",
        "descendantBlocks": [],
        "data": "",
        "fusedPlasma": 21000, "difficulty": 0, "nonce": "0000000000000000",
        "basePlasma": 21000, "usedPlasma": 21000,
        "changesHash": "a31a31bb26f7a7ee5b5c8e83e6b47aeeab6e2330476199d93ee8ca37ac71465a",
        "publicKey": "GYyn77OXTL31zPbDBCe/eKir+VCF3hv+LxiOUF3XcJY=",
        "signature": "hrQwfpdEYTjoLV9yzEppeky2Y/9T1x760vQPL6NLgD+cn0XD1+F/dOcSwyhg8RxjHWMN6MvD2NnTAX7N+5aCBQ==",
        "token": {
            "name": "Zenon Coin", "symbol": "ZNN", "domain": "zenon.network",
            "totalSupply": "19500000000000", "decimals": 8,
            "owner": "z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg",
            "tokenStandard": "zts1tfjkummwyppk76twsnv50e",
            "maxSupply": "4611686018427387903",
            "isBurnable": true, "isMintable": true, "isUtility": true
        },
        "confirmationDetail": {
            "numConfirmations": 2, "momentumHeight": 2,
            "momentumHash": "0f92b0be5eef439be78f9d48add78288391d6723e40c7059fae0f1241a9e639f",
            "momentumTimestamp": 1_000_000_010
        },
        "pairedAccountBlock": null
    });
    json!({ "count": 1, "list": [block], "more": false })
}

#[derive(Deserialize)]
struct AccountInfoConformance {
    account_info: Value,
}

const ACCOUNT_INFO_CONFORMANCE: &str = include_str!("conformance/nom/account_info.json");

#[tokio::test]
async fn get_frontier_momentum_decodes() {
    let expected = Momentum::from_json(&momentum_json()).expect("fixture parses");
    let node = MockNode::spawn(|module| {
        module
            .register_method("ledger.getFrontierMomentum", |_params, _ctx, _ext| {
                let value: jsonrpsee::core::RpcResult<Value> = Ok(momentum_json());
                value
            })
            .expect("method registers");
    })
    .await;
    let api = LedgerApi::new(connect(&node.url).await);
    let result = api.get_frontier_momentum().await;
    assert_eq!(result, Ok(expected));
}

#[tokio::test]
async fn get_momentum_by_hash_maps_null_to_none() {
    let node = MockNode::spawn(|module| {
        module
            .register_method("ledger.getMomentumByHash", |_params, _ctx, _ext| {
                let value: jsonrpsee::core::RpcResult<Value> = Ok(Value::Null);
                value
            })
            .expect("method registers");
    })
    .await;
    let api = LedgerApi::new(connect(&node.url).await);
    let result = api
        .get_momentum_by_hash(
            &znn_sdk_rust::primitives::hash::Hash::parse(
                "c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9",
            )
            .expect("hash parses"),
        )
        .await;
    assert_eq!(result, Ok(None));
}

#[tokio::test]
async fn get_momentums_by_height_clamps_height_and_count() {
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        captured = Some(capturing(
            module,
            "ledger.getMomentumsByHeight",
            json!({ "count": 0, "list": [] }),
        ));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = LedgerApi::new(connect(&node.url).await);
    let _ = api.get_momentums_by_height(0, 9_999).await;
    let params = captured.lock().expect("params cell").clone();
    assert_eq!(
        params,
        vec![
            json!(1),
            json!(znn_sdk_rust::client::constants::RPC_MAX_PAGE_SIZE)
        ],
        "height must be floored to 1 and count capped at RPC_MAX_PAGE_SIZE"
    );
}

#[tokio::test]
async fn get_account_blocks_by_page_decodes() {
    let value = account_block_list_json();
    let expected = AccountBlockList::from_json(&value).expect("list parses");
    let response_value = value.clone();
    let node = MockNode::spawn(|module| {
        module
            .register_method(
                "ledger.getAccountBlocksByPage",
                move |_params, _ctx, _ext| {
                    let response: jsonrpsee::core::RpcResult<Value> = Ok(response_value.clone());
                    response
                },
            )
            .expect("method registers");
    })
    .await;
    let api = LedgerApi::new(connect(&node.url).await);
    let result = api
        .get_account_blocks_by_page(
            &Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").expect("address parses"),
            PageQuery::default(),
        )
        .await;
    assert_eq!(result, Ok(expected));
}

#[tokio::test]
async fn get_account_info_by_address_decodes() {
    let conformance: AccountInfoConformance =
        serde_json::from_str(ACCOUNT_INFO_CONFORMANCE).expect("conformance parses");
    let expected = AccountInfo::from_json(&conformance.account_info).expect("account info parses");
    let response_value = conformance.account_info;
    let node = MockNode::spawn(|module| {
        module
            .register_method(
                "ledger.getAccountInfoByAddress",
                move |_params, _ctx, _ext| {
                    let response: jsonrpsee::core::RpcResult<Value> = Ok(response_value.clone());
                    response
                },
            )
            .expect("method registers");
    })
    .await;
    let api = LedgerApi::new(connect(&node.url).await);
    let result = api
        .get_account_info_by_address(
            &Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").expect("address parses"),
        )
        .await;
    assert_eq!(result, Ok(expected));
}

#[tokio::test]
async fn publish_raw_transaction_forwards_the_template_json() {
    let template = AccountBlockTemplate::new(BlockType::UserSend);
    let expected_json = template.to_json();
    let mut captured = None;
    let node = MockNode::spawn(|module| {
        captured = Some(capturing(
            module,
            "ledger.publishRawTransaction",
            Value::Null,
        ));
    })
    .await;
    let captured = captured.expect("capture registered");
    let api = LedgerApi::new(connect(&node.url).await);
    let result = api.publish_raw_transaction(&template).await;
    let params = captured.lock().expect("params cell").clone();
    assert_eq!(
        params,
        vec![expected_json],
        "publishRawTransaction must forward the template JSON"
    );
    assert_eq!(result, Ok(Value::Null));
}

#[tokio::test]
async fn a_transport_error_surfaces_as_a_client_error() {
    // A fresh client with no live transport.
    let api = LedgerApi::new(std::sync::Arc::new(WsClient::new()));
    let result = api.get_frontier_momentum().await;
    assert!(
        matches!(result, Err(Error::Client(ClientError::NoConnection))),
        "a transport error must surface as Error::Client(NoConnection), got {result:?}"
    );
}

#[test]
fn page_query_defaults_match_expected_values() {
    assert_eq!(
        PageQuery::default(),
        znn_sdk_rust::api::PageQuery {
            index: 0,
            size: znn_sdk_rust::client::constants::RPC_MAX_PAGE_SIZE
        }
    );
    assert_eq!(
        PageQuery::mempool(),
        znn_sdk_rust::api::PageQuery {
            index: 0,
            size: znn_sdk_rust::client::constants::MEMORY_POOL_PAGE_SIZE
        }
    );
}

#[test]
fn account_block_list_fixture_decodes_one_block() {
    // Sanity check the fixture itself so a decode failure is not masked by the
    // API stub returning Err.
    let value = account_block_list_json();
    let list = AccountBlockList::from_json(&value).expect("fixture parses");
    assert_eq!(list.count, Some(1));
    assert_eq!(list.list.as_ref().map(Vec::len), Some(1));
    let _block: &AccountBlock = list
        .list
        .as_ref()
        .and_then(|list| list.first())
        .expect("one block");
}

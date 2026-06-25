//! Integration tests for `crate::model::nom::account_block_template` JSON
//! parsing and serialization.
#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]

use serde_json::{Value, json};
use znn_sdk_rust::Error;
use znn_sdk_rust::model::nom::account_block_template::AccountBlockTemplate;

const TEMPLATE_JSON: &str = r#"{
  "version": 1,
  "chainIdentifier": 100,
  "blockType": 2,
  "hash": "3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73",
  "previousHash": "598fa623dd308bec7163bb375aa7546ec4aced3b71a1c9278709903e69280dbd",
  "height": 2,
  "momentumAcknowledged": {
    "hash": "c37c70550e95d0c72f0924d480321976040108f29fa7530487f8dde81e713689",
    "height": 1
  },
  "address": "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz",
  "toAddress": "z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx",
  "amount": "10000000000",
  "tokenStandard": "zts1tfjkummwyppk76twsnv50e",
  "fromBlockHash": "0000000000000000000000000000000000000000000000000000000000000000",
  "data": "",
  "fusedPlasma": 21000,
  "difficulty": 0,
  "nonce": "0000000000000000",
  "publicKey": "GYyn77OXTL31zPbDBCe/eKir+VCF3hv+LxiOUF3XcJY=",
  "signature": "hrQwfpdEYTjoLV9yzEppeky2Y/9T1x760vQPL6NLgD+cn0XD1+F/dOcSwyhg8RxjHWMN6MvD2NnTAX7N+5aCBQ=="
}"#;

#[test]
fn same_json() {
    let expected: Value = serde_json::from_str(TEMPLATE_JSON).expect("fixture parses");
    let template = AccountBlockTemplate::from_json(&expected).expect("template parses");
    assert_eq!(template.to_json(), expected);
}

#[test]
fn rejects_a_malformed_template_object() {
    let mut missing = serde_json::from_str::<Value>(TEMPLATE_JSON).expect("fixture parses");
    missing
        .as_object_mut()
        .expect("fixture is an object")
        .remove("version");
    let result = AccountBlockTemplate::from_json(&missing);
    assert!(result.is_err(), "missing version must be rejected");
    assert!(matches!(result, Err(Error::InvalidInput(_))));

    let mut bad_amount = serde_json::from_str::<Value>(TEMPLATE_JSON).expect("fixture parses");
    bad_amount["amount"] = json!("not-a-number");
    let result = AccountBlockTemplate::from_json(&bad_amount);
    assert!(result.is_err(), "non-decimal amount must be rejected");
    assert!(matches!(result, Err(Error::InvalidInput(_))));

    let mut bad_block_type = serde_json::from_str::<Value>(TEMPLATE_JSON).expect("fixture parses");
    bad_block_type["blockType"] = json!(6);
    let result = AccountBlockTemplate::from_json(&bad_block_type);
    assert!(result.is_err(), "unknown blockType must be rejected");
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

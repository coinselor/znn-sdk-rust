//! Integration test for `crate::model::nom::momentum` JSON parsing and
//! serialization.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use serde_json::Value;
use znn_sdk_rust::model::nom::momentum::Momentum;

const MOMENTUM_JSON: &str = r#"{
    "version": 1,
    "chainIdentifier": 100,
    "hash": "c54f50fbd2dca9f3410b7693031b1a44d75375bfc0946410a2558895b2330db9",
    "previousHash": "0a1ec5f298fdca1402d2a88472f806b020b161896dab064ba381138d66fad712",
    "height": 2,
    "timestamp": 1000000010,
    "data": "",
    "content": [],
    "changesHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
    "publicKey": "SAPwVIVQma3zMak169crdLkcu2B2Gm3iBCdDgfQ6IxU=",
    "signature": "qvlKN6rTQgM11/FosNazpeReViuD1GH1tIww2F0zNfXruTp3g9ULhA1mYnRYAiPJyP2NlIGhENwhzBAHJ0dYBw==",
    "producer": "z1qz8v73ea2vy2rrlq7skssngu8cm8mknjjkr2ju"
  }"#;

#[test]
fn same_json() {
    let expected: Value = serde_json::from_str(MOMENTUM_JSON).expect("fixture parses");
    let momentum = Momentum::from_json(&expected).expect("momentum parses");
    assert_eq!(momentum.to_json(), expected);
}

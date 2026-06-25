//! Tests for runnable examples.
#![allow(clippy::expect_used)]

#[path = "../examples/api.rs"]
mod api_example;
#[path = "../examples/embedded.rs"]
mod embedded_example;
#[path = "../examples/send.rs"]
mod send_example;
#[path = "../examples/wallet.rs"]
mod wallet_example;

#[test]
fn wallet_derivation_matches_expected_values() {
    let report = wallet_example::derive_example_wallet().expect("wallet derivation succeeds");

    assert_eq!(
        report.entropy,
        "bc827d0a00a72354dce4c44a59485288500b49382f9ba88a016351787b7b15ca"
    );
    assert_eq!(
        report.private_key,
        "d6b01f96b566d7df9b5b53b1971e4baeb74cc64167a9843f82d04b2194ca4863"
    );
    assert_eq!(
        report.public_key,
        "3e13d7238d0e768a567dce84b54915f2323f2dcd0ef9a716d9c61abed631ba10"
    );
    assert_eq!(report.address, "z1qqjnwjjpnue8xmmpanz6csze6tcmtzzdtfsww7");
    assert_eq!(
        report.address_core,
        "0025374a419f32736f61ecc5ac4059d2f1b5884d"
    );
    assert_eq!(report.chain_identifier, 1);
}

#[tokio::test]
async fn api_example_runs_offline_without_panicking() {
    let message = api_example::run_with_url("ws://127.0.0.1:1").await;

    assert!(
        message.contains("could not connect") || message.contains("No connection"),
        "offline api example must print a connection message, got {message:?}"
    );
}

#[test]
fn send_example_builds_a_self_send_template() {
    let template =
        send_example::build_self_send_template(wallet_example::MNEMONIC).expect("template builds");

    assert_eq!(
        template.amount(),
        &num_bigint::BigUint::from(send_example::EXAMPLE_AMOUNT)
    );
    assert_eq!(
        template.to_address().to_string(),
        "z1qqjnwjjpnue8xmmpanz6csze6tcmtzzdtfsww7"
    );
}

#[test]
fn embedded_example_builds_templates() {
    let templates = embedded_example::build_embedded_templates().expect("templates build");

    assert_eq!(templates.len(), 3);
    assert!(templates.iter().all(|template| !template.data().is_empty()));
}

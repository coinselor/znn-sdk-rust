//! Embedded contract builder examples.
//!
//! These examples build account-block templates locally. Publishing them with
//! `Zenon::send` still requires a connected node, balance/plasma, and any
//! protocol-specific privileges.

use num_bigint::BigUint;
use std::sync::Arc;
use znn_sdk_rust::api::embedded::EmbeddedApi;
use znn_sdk_rust::client::websocket::WsClient;
use znn_sdk_rust::embedded::constants::{FUSE_MIN_QSR_AMOUNT, STAKE_TIME_MIN_SEC};
use znn_sdk_rust::error::Error;
use znn_sdk_rust::model::nom::account_block_template::AccountBlockTemplate;
use znn_sdk_rust::primitives::address::Address;

/// Returns sample embedded contract templates: stake, plasma fuse, and token issue.
pub fn build_embedded_templates() -> Result<Vec<AccountBlockTemplate>, Error> {
    let embedded = EmbeddedApi::new(Arc::new(WsClient::new()));
    let beneficiary = Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx")?;

    let stake = embedded
        .stake
        .stake(STAKE_TIME_MIN_SEC, BigUint::from(1_000_000_000u64));
    let fuse = embedded
        .plasma
        .fuse(beneficiary, BigUint::from(FUSE_MIN_QSR_AMOUNT));
    let issue = embedded.token.issue_token(
        "ExampleToken",
        "EXT",
        "example.com",
        BigUint::from(1_000_000u64),
        BigUint::from(1_000_000u64),
        8,
        true,
        true,
        false,
    );

    Ok(vec![stake, fuse, issue])
}

#[allow(dead_code)]
fn main() -> Result<(), Error> {
    for (index, template) in build_embedded_templates()?.iter().enumerate() {
        println!(
            "template[{index}] to={} amount={} data_len={}",
            template.to_address(),
            template.amount(),
            template.data().len()
        );
    }
    Ok(())
}

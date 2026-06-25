//! Spork embedded contract API.

use crate::abi::AbiValue;
use crate::api::PageQuery;
use crate::api::embedded::{dispatch, embedded_address, encode_call, page_params};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::definitions::SPORK_DEFINITION;
use crate::error::Error;
use crate::model::embedded::spork::SporkList;
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::znn_token_standard;
use num_bigint::BigUint;
use std::sync::Arc;

/// Spork API root.
pub struct SporkApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> SporkApi<C> {
    /// Creates a spork API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns all sporks.
    pub async fn get_all(&self, page: PageQuery) -> Result<SporkList, Error> {
        let response = dispatch(&*self.client, "embedded.spork.getAll", &page_params(page)).await?;
        SporkList::from_json(&response)
    }

    /// Builds a create-spork template.
    pub fn create_spork(&self, name: &str, description: &str) -> AccountBlockTemplate {
        let data = encode_call(
            SPORK_DEFINITION,
            "CreateSpork",
            &[
                AbiValue::String(name.to_string()),
                AbiValue::String(description.to_string()),
            ],
        );
        AccountBlockTemplate::call_contract(
            spork_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds an activate-spork template.
    pub fn activate_spork(&self, id: &Hash) -> AccountBlockTemplate {
        let data = encode_call(
            SPORK_DEFINITION,
            "ActivateSpork",
            &[AbiValue::Hash(id.clone())],
        );
        AccountBlockTemplate::call_contract(
            spork_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the spork contract address.
pub fn spork_address() -> Address {
    embedded_address("z1qxemdeddedxsp0rkxxxxxxxxxxxxxxxx956u48")
}

//! Token embedded contract API.

use crate::abi::AbiValue;
use crate::api::PageQuery;
use crate::api::embedded::{
    address_page_params, dispatch, embedded_address, encode_call, optional, page_params,
};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::constants::TOKEN_ZTS_ISSUE_FEE_IN_ZNN;
use crate::embedded::definitions::TOKEN_DEFINITION;
use crate::error::Error;
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::model::nom::token::{Token, TokenList};
use crate::primitives::address::Address;
use crate::primitives::token_standard::{TokenStandard, znn_token_standard};
use num_bigint::BigUint;
use serde_json::json;
use std::sync::Arc;

/// Token API root.
pub struct TokenApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> TokenApi<C> {
    /// Creates a token API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns all tokens.
    pub async fn get_all(&self, page: PageQuery) -> Result<TokenList, Error> {
        let response = dispatch(&*self.client, "embedded.token.getAll", &page_params(page)).await?;
        TokenList::from_json(&response)
    }

    /// Returns tokens by owner.
    pub async fn get_by_owner(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<TokenList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.token.getByOwner",
            &address_page_params(address, page),
        )
        .await?;
        TokenList::from_json(&response)
    }

    /// Returns the token for `zts`, or `None`.
    pub async fn get_by_zts(&self, zts: &TokenStandard) -> Result<Option<Token>, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.token.getByZts",
            &[json!(zts.to_string())],
        )
        .await?;
        optional(&response, Token::from_json)
    }

    /// Builds an issue-token template.
    #[allow(clippy::too_many_arguments)]
    pub fn issue_token(
        &self,
        name: &str,
        symbol: &str,
        domain: &str,
        total_supply: BigUint,
        max_supply: BigUint,
        decimals: u8,
        mintable: bool,
        burnable: bool,
        utility: bool,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            TOKEN_DEFINITION,
            "IssueToken",
            &[
                AbiValue::String(name.to_string()),
                AbiValue::String(symbol.to_string()),
                AbiValue::String(domain.to_string()),
                AbiValue::UInt(total_supply),
                AbiValue::UInt(max_supply),
                AbiValue::UInt(BigUint::from(decimals)),
                AbiValue::Bool(mintable),
                AbiValue::Bool(burnable),
                AbiValue::Bool(utility),
            ],
        );
        AccountBlockTemplate::call_contract(
            token_address(),
            znn_token_standard(),
            BigUint::from(TOKEN_ZTS_ISSUE_FEE_IN_ZNN),
            data,
        )
    }

    /// Builds a mint-token template.
    pub fn mint_token(
        &self,
        zts: TokenStandard,
        amount: BigUint,
        to_address: Address,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            TOKEN_DEFINITION,
            "Mint",
            &[
                AbiValue::TokenStandard(zts),
                AbiValue::UInt(amount),
                AbiValue::Address(to_address),
            ],
        );
        AccountBlockTemplate::call_contract(
            token_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a burn-token template.
    pub fn burn_token(&self, zts: TokenStandard, amount: BigUint) -> AccountBlockTemplate {
        let data = encode_call(TOKEN_DEFINITION, "Burn", &[]);
        AccountBlockTemplate::call_contract(token_address(), zts, amount, data)
    }

    /// Builds an update-token template.
    pub fn update_token(
        &self,
        zts: TokenStandard,
        owner: Address,
        mintable: bool,
        burnable: bool,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            TOKEN_DEFINITION,
            "UpdateToken",
            &[
                AbiValue::TokenStandard(zts),
                AbiValue::Address(owner),
                AbiValue::Bool(mintable),
                AbiValue::Bool(burnable),
            ],
        );
        AccountBlockTemplate::call_contract(
            token_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the token contract address.
pub fn token_address() -> Address {
    embedded_address("z1qxemdeddedxt0kenxxxxxxxxxxxxxxxxh9amk0")
}

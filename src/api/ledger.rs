//! Ledger API (`ledger.*` JSON-RPC methods).
//!
//! [`LedgerApi`] wraps a shared [`Client`] and dispatches the `ledger.*`
//! namespace: account-block, momentum, and account-info queries plus
//! [`publish_raw_transaction`]. Reads are `async` and return [`Error`] on
//! failure, mapping a transport failure to [`Error::Client`].
//!
//! [`publish_raw_transaction`]: LedgerApi::publish_raw_transaction

use crate::api::PageQuery;
use crate::api::{address_page_params, optional, page_params};
use crate::client::constants::RPC_MAX_PAGE_SIZE;
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::error::Error;
use crate::model::nom::account_block::{AccountBlock, AccountBlockList};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::model::nom::account_info::AccountInfo;
use crate::model::nom::detailed_momentum::DetailedMomentumList;
use crate::model::nom::momentum::{Momentum, MomentumList};
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use serde_json::{Value, json};
use std::sync::Arc;

/// The `ledger.*` JSON-RPC namespace.
pub struct LedgerApi<C: Client = WsClient> {
    client: Arc<C>,
}

/// Re-export of the typed publish result for ergonomic access alongside the
/// publish methods. The type itself is defined in [`crate::publish`] so it
/// remains available in reduced-core builds that omit the websocket transport.
pub use crate::publish::{PublishError, PublishResult};

impl<C: Client> LedgerApi<C> {
    /// Creates a ledger API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns the frontier momentum.
    pub async fn get_frontier_momentum(&self) -> Result<Momentum, Error> {
        let response = self.dispatch("ledger.getFrontierMomentum", &[]).await?;
        Momentum::from_json(&response)
    }

    /// Returns the momentum at or before `time`, or `None`.
    pub async fn get_momentum_before_time(&self, time: u64) -> Result<Option<Momentum>, Error> {
        let response = self
            .dispatch("ledger.getMomentumBeforeTime", &[json!(time)])
            .await?;
        optional(&response, Momentum::from_json)
    }

    /// Returns the momentum with `hash`, or `None`.
    pub async fn get_momentum_by_hash(&self, hash: &Hash) -> Result<Option<Momentum>, Error> {
        let response = self
            .dispatch("ledger.getMomentumByHash", &[json!(hash.to_string())])
            .await?;
        optional(&response, Momentum::from_json)
    }

    /// Returns up to `count` momentums starting at `height`, clamping `height`
    /// to a minimum of `1` and `count` to [`RPC_MAX_PAGE_SIZE`].
    pub async fn get_momentums_by_height(
        &self,
        height: u64,
        count: u64,
    ) -> Result<MomentumList, Error> {
        let (height, count) = clamp_range(height, count);
        let response = self
            .dispatch(
                "ledger.getMomentumsByHeight",
                &[json!(height), json!(count)],
            )
            .await?;
        MomentumList::from_json(&response)
    }

    /// Returns a page of momentums.
    pub async fn get_momentums_by_page(&self, page: PageQuery) -> Result<MomentumList, Error> {
        let response = self
            .dispatch("ledger.getMomentumsByPage", &page_params(page))
            .await?;
        MomentumList::from_json(&response)
    }

    /// Returns up to `count` detailed momentums starting at `height`, clamping
    /// `height` to a minimum of `1` and `count` to [`RPC_MAX_PAGE_SIZE`].
    pub async fn get_detailed_momentums_by_height(
        &self,
        height: u64,
        count: u64,
    ) -> Result<DetailedMomentumList, Error> {
        let (height, count) = clamp_range(height, count);
        let response = self
            .dispatch(
                "ledger.getDetailedMomentumsByHeight",
                &[json!(height), json!(count)],
            )
            .await?;
        DetailedMomentumList::from_json(&response)
    }

    /// Returns the unconfirmed (memory-pool) blocks for `address`.
    pub async fn get_unconfirmed_blocks_by_address(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<AccountBlockList, Error> {
        let response = self
            .dispatch(
                "ledger.getUnconfirmedBlocksByAddress",
                &address_page_params(address, page),
            )
            .await?;
        AccountBlockList::from_json(&response)
    }

    /// Returns the unreceived blocks for `address`.
    pub async fn get_unreceived_blocks_by_address(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<AccountBlockList, Error> {
        let response = self
            .dispatch(
                "ledger.getUnreceivedBlocksByAddress",
                &address_page_params(address, page),
            )
            .await?;
        AccountBlockList::from_json(&response)
    }

    /// Returns the frontier account block for `address`, or `None`.
    pub async fn get_frontier_account_block(
        &self,
        address: &Address,
    ) -> Result<Option<AccountBlock>, Error> {
        let response = self
            .dispatch(
                "ledger.getFrontierAccountBlock",
                &[json!(address.to_string())],
            )
            .await?;
        optional(&response, AccountBlock::from_json)
    }

    /// Returns the account block with `hash`, or `None`.
    pub async fn get_account_block_by_hash(
        &self,
        hash: &Hash,
    ) -> Result<Option<AccountBlock>, Error> {
        let response = self
            .dispatch("ledger.getAccountBlockByHash", &[json!(hash.to_string())])
            .await?;
        optional(&response, AccountBlock::from_json)
    }

    /// Returns account blocks for `address` at `height`.
    pub async fn get_account_blocks_by_height(
        &self,
        address: &Address,
        height: u64,
        count: u64,
    ) -> Result<AccountBlockList, Error> {
        let response = self
            .dispatch(
                "ledger.getAccountBlocksByHeight",
                &[json!(address.to_string()), json!(height), json!(count)],
            )
            .await?;
        AccountBlockList::from_json(&response)
    }

    /// Returns a page of account blocks for `address`.
    pub async fn get_account_blocks_by_page(
        &self,
        address: &Address,
        page: PageQuery,
    ) -> Result<AccountBlockList, Error> {
        let response = self
            .dispatch(
                "ledger.getAccountBlocksByPage",
                &address_page_params(address, page),
            )
            .await?;
        AccountBlockList::from_json(&response)
    }

    /// Returns the account info for `address`.
    pub async fn get_account_info_by_address(
        &self,
        address: &Address,
    ) -> Result<AccountInfo, Error> {
        let response = self
            .dispatch(
                "ledger.getAccountInfoByAddress",
                &[json!(address.to_string())],
            )
            .await?;
        AccountInfo::from_json(&response)
    }

    /// Publishes a signed `template`, returning the node's raw response.
    pub async fn publish_raw_transaction(
        &self,
        template: &AccountBlockTemplate,
    ) -> Result<Value, Error> {
        self.dispatch("ledger.publishRawTransaction", &[template.to_json()])
            .await
    }

    /// Publishes a signed `template`, returning a typed [`PublishResult`] on
    /// acceptance (the node's `null` response) or surfacing a non-null response
    /// as a typed [`Error::Publish`] carrying the decoded [`PublishError`].
    ///
    /// Delegates to [`Self::publish_raw_transaction`]: a `null` response yields a
    /// [`PublishResult`] carrying the template's computed transaction hash, while
    /// any non-null response is decoded by [`PublishError::from_response`] into a
    /// rejection string ([`PublishError::Rejected`]) or an unexpected shape
    /// ([`PublishError::Unexpected`]).
    pub async fn publish_transaction(
        &self,
        template: &AccountBlockTemplate,
    ) -> Result<PublishResult, Error> {
        let response = self.publish_raw_transaction(template).await?;
        if response.is_null() {
            let hash = crate::utils::block::get_transaction_hash(template)?;
            return Ok(PublishResult::new(hash, template.clone()));
        }
        Err(Error::Publish(PublishError::from_response(&response)))
    }

    async fn dispatch(&self, method: &str, params: &[Value]) -> Result<Value, Error> {
        self.client
            .send_request(method, params)
            .await
            .map_err(Error::from)
    }
}

/// Floors `height` to a minimum of `1` and caps `count` at [`RPC_MAX_PAGE_SIZE`].
fn clamp_range(height: u64, count: u64) -> (u64, u64) {
    let height = height.max(1);
    let count = count.min(u64::from(RPC_MAX_PAGE_SIZE));
    (height, count)
}

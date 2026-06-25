//! High-level SDK entry point: client, wallet, and send orchestration.

use crate::api::embedded::EmbeddedApi;
use crate::api::ledger::{LedgerApi, PublishResult};
use crate::api::stats::StatsApi;
use crate::api::subscribe::SubscribeApi;
use crate::client::exceptions::ClientError;
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::error::Error;
use crate::model::embedded::plasma::{GetRequiredParam, GetRequiredResponse};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::pow;
use crate::pow::provider::{NO_PROVIDER_CONFIGURED, PowProvider, default_pow_provider};
use crate::primitives::hash::Hash;
use crate::primitives::hash_height::HashHeight;
use crate::utils::block;
use crate::wallet::keypair::KeyPair;
use serde_json::Value;
use std::sync::Arc;

/// Canonical hex nonce for zero-difficulty account blocks.
const ZERO_NONCE_HEX: &str = "0000000000000000";

/// High-level SDK entry point exposing client-backed API roots over one shared client.
pub struct Zenon<C: Client = WsClient> {
    client: Arc<C>,
    /// Ledger API root.
    pub ledger: LedgerApi<C>,
    /// Stats API root.
    pub stats: StatsApi<C>,
    /// Subscribe API root.
    pub subscribe: SubscribeApi<C>,
    /// Embedded contract API root.
    pub embedded: EmbeddedApi<C>,
    default_key_pair: Option<KeyPair>,
    pow_provider: Option<Box<dyn PowProvider>>,
}

impl<C: Client> Zenon<C> {
    /// Builds an SDK entry point from a pre-built shared client.
    pub fn from_client(client: Arc<C>) -> Self {
        Self {
            ledger: LedgerApi::new(Arc::clone(&client)),
            stats: StatsApi::new(Arc::clone(&client)),
            subscribe: SubscribeApi::new(Arc::clone(&client)),
            embedded: EmbeddedApi::new(Arc::clone(&client)),
            client,
            default_key_pair: None,
            pow_provider: None,
        }
    }

    /// Returns the shared client.
    pub fn client(&self) -> &Arc<C> {
        &self.client
    }

    /// Returns the default key pair used by [`Self::send`] when no explicit key pair is passed.
    pub fn default_key_pair(&self) -> Option<&KeyPair> {
        self.default_key_pair.as_ref()
    }

    /// Sets the default key pair used by [`Self::send`] when no explicit key pair is passed.
    pub fn set_default_key_pair(&mut self, key_pair: KeyPair) {
        self.default_key_pair = Some(key_pair);
    }

    /// Clears the default key pair.
    pub fn clear_default_key_pair(&mut self) {
        self.default_key_pair = None;
    }

    /// Sets the proof-of-work provider used by block preparation for non-zero
    /// required difficulty. When unset, preparation falls back to native
    /// in-process generation (when the `native-pow` feature is enabled).
    pub fn set_pow_provider(&mut self, provider: Box<dyn PowProvider>) {
        self.pow_provider = Some(provider);
    }

    /// Prepares `template` for publishing without publishing it: completes the
    /// mutable account-block fields, applies the required proof-of-work,
    /// computes the transaction hash, signs it, and returns the prepared
    /// template.
    ///
    /// `keypair` selects the signer: an explicit pair takes precedence,
    /// otherwise the SDK entry point's default key pair is used. Returns the existing
    /// missing-key error when neither is available.
    pub async fn prepare_block(
        &self,
        template: &AccountBlockTemplate,
        keypair: Option<&KeyPair>,
    ) -> Result<AccountBlockTemplate, Error> {
        let selected = self.select_key_pair(keypair)?;
        let mut transaction = template.clone();
        self.check_and_set_fields(&mut transaction, selected)
            .await?;
        self.apply_required_pow(&mut transaction).await?;
        set_hash_and_signature(&mut transaction, selected)?;
        Ok(transaction)
    }

    /// Completes, signs, and publishes `template` using an explicit or default key pair.
    ///
    /// This prepares the template through [`Self::prepare_block`] and then
    /// publishes the resulting signed template. The returned value is the raw
    /// `ledger.publishRawTransaction` node response.
    pub async fn send(
        &self,
        template: &AccountBlockTemplate,
        keypair: Option<&KeyPair>,
    ) -> Result<Value, Error> {
        let prepared = self.prepare_block(template, keypair).await?;
        self.ledger.publish_raw_transaction(&prepared).await
    }

    /// Typed send: prepares, signs, and publishes `template`, returning a
    /// [`PublishResult`] on acceptance or decoding a non-null node response into
    /// a typed error.
    ///
    /// Prepares and signs `template` via [`Self::prepare_block`], then publishes
    /// through [`LedgerApi::publish_transaction`], which returns the typed
    /// [`PublishResult`] on acceptance or surfaces a non-null node response as a
    /// typed [`Error::Publish`].
    pub async fn send_typed(
        &self,
        template: &AccountBlockTemplate,
        keypair: Option<&KeyPair>,
    ) -> Result<PublishResult, Error> {
        let prepared = self.prepare_block(template, keypair).await?;
        self.ledger.publish_transaction(&prepared).await
    }

    /// Returns whether sending `template` with `keypair` would require proof-of-work.
    pub async fn requires_pow(
        &self,
        template: &AccountBlockTemplate,
        keypair: &KeyPair,
    ) -> Result<bool, Error> {
        let address = keypair.address()?;
        let param = required_pow_param(template, address);
        let response = self
            .embedded
            .plasma
            .get_required_pow_for_account_block(&param)
            .await?;
        Ok(response.required_difficulty() != 0)
    }

    fn select_key_pair<'a>(&'a self, keypair: Option<&'a KeyPair>) -> Result<&'a KeyPair, Error> {
        keypair
            .or(self.default_key_pair.as_ref())
            .ok_or_else(|| Error::generic("No default wallet account selected"))
    }

    async fn check_and_set_fields(
        &self,
        transaction: &mut AccountBlockTemplate,
        keypair: &KeyPair,
    ) -> Result<(), Error> {
        transaction.set_address(keypair.address()?);
        transaction.set_public_key(keypair.public_key().to_vec());
        self.autofill_transaction_parameters(transaction).await?;

        if block::is_send_block(transaction.block_type()) {
            return Ok(());
        }

        validate_receive_transaction(self, transaction).await
    }

    async fn autofill_transaction_parameters(
        &self,
        transaction: &mut AccountBlockTemplate,
    ) -> Result<(), Error> {
        let frontier = self
            .ledger
            .get_frontier_account_block(transaction.address())
            .await?;
        if let Some(block) = frontier {
            let template = block.template();
            let height = template
                .height()
                .checked_add(1)
                .ok_or_else(|| Error::InvalidInput("account height overflows u64".to_string()))?;
            transaction.set_height(height);
            transaction.set_previous_hash(template.hash().clone());
        } else {
            transaction.set_height(1);
            transaction.set_previous_hash(Hash::empty());
        }

        let frontier_momentum = self.ledger.get_frontier_momentum().await?;
        transaction.set_momentum_acknowledged(HashHeight::new(
            frontier_momentum.hash().clone(),
            frontier_momentum.height(),
        ));
        Ok(())
    }

    async fn apply_required_pow(
        &self,
        transaction: &mut AccountBlockTemplate,
    ) -> Result<(), Error> {
        let param = required_pow_param(transaction, transaction.address().clone());
        let response = self
            .embedded
            .plasma
            .get_required_pow_for_account_block(&param)
            .await?;
        apply_required_pow_response(self, transaction, &response).await
    }

    /// Resolves the proof-of-work nonce for a non-zero difficulty.
    ///
    /// A configured provider wins; otherwise the default factory resolves the
    /// provider for the current build (native when `native-pow` is enabled,
    /// otherwise none). The `native-pow` feature gate lives entirely in
    /// [`default_pow_provider`], so this body is `cfg`-free.
    async fn resolve_nonce(&self, data_hash: &Hash, difficulty: u64) -> Result<[u8; 8], Error> {
        match self.pow_provider.as_ref() {
            Some(provider) => provider.generate_pow(data_hash, difficulty).await,
            None => {
                default_pow_provider()
                    .ok_or_else(|| Error::generic(NO_PROVIDER_CONFIGURED))?
                    .generate_pow(data_hash, difficulty)
                    .await
            }
        }
    }
}

impl Zenon<WsClient> {
    /// Creates an unconnected SDK entry point.
    pub fn new() -> Self {
        Self::from_client(Arc::new(WsClient::new()))
    }

    /// Connects a WebSocket client before sharing it across all API roots.
    pub async fn connect(url: &str, retry: bool) -> Result<Self, ClientError> {
        let mut client = WsClient::new();
        client.initialize(url, retry).await?;
        Ok(Self::from_client(Arc::new(client)))
    }
}

impl Default for Zenon<WsClient> {
    fn default() -> Self {
        Self::new()
    }
}

async fn validate_receive_transaction<C: Client>(
    zenon: &Zenon<C>,
    transaction: &AccountBlockTemplate,
) -> Result<(), Error> {
    if transaction.from_block_hash() == &Hash::empty() {
        return Err(Error::InvalidInput(
            "receive transaction requires fromBlockHash".to_string(),
        ));
    }
    let send_block = zenon
        .ledger
        .get_account_block_by_hash(transaction.from_block_hash())
        .await?
        .ok_or_else(|| {
            Error::InvalidInput("fromBlockHash does not reference a block".to_string())
        })?;
    if send_block.template().to_address() != transaction.address() {
        return Err(Error::InvalidInput(
            "receive transaction is not addressed to signer".to_string(),
        ));
    }
    if !transaction.data().is_empty() {
        return Err(Error::InvalidInput(
            "receive transaction data must be empty".to_string(),
        ));
    }
    Ok(())
}

fn required_pow_param(
    template: &AccountBlockTemplate,
    address: crate::primitives::address::Address,
) -> GetRequiredParam {
    GetRequiredParam::new(
        address,
        template.block_type_enum(),
        Some(template.to_address().clone()),
        template.data().to_vec(),
    )
}

async fn apply_required_pow_response<C: Client>(
    zenon: &Zenon<C>,
    transaction: &mut AccountBlockTemplate,
    response: &GetRequiredResponse,
) -> Result<(), Error> {
    if response.required_difficulty() == 0 {
        transaction.set_fused_plasma(u64_to_u32("basePlasma", response.base_plasma())?);
        transaction.set_difficulty(0);
        transaction.set_nonce(ZERO_NONCE_HEX);
        return Ok(());
    }

    let difficulty = response.required_difficulty();
    transaction.set_fused_plasma(u64_to_u32("availablePlasma", response.available_plasma())?);
    transaction.set_difficulty(u64_to_u32("requiredDifficulty", difficulty)?);
    let data_hash =
        pow::account_block_data_hash(transaction.address(), transaction.previous_hash());
    let nonce = zenon.resolve_nonce(&data_hash, difficulty).await?;
    if !pow::verify_pow(&data_hash, &nonce, difficulty) {
        return Err(Error::InvalidInput(
            "proof-of-work provider returned a nonce that does not satisfy the required difficulty"
                .to_string(),
        ));
    }
    transaction.set_nonce(const_hex::encode(nonce));
    Ok(())
}

fn set_hash_and_signature(
    transaction: &mut AccountBlockTemplate,
    keypair: &KeyPair,
) -> Result<(), Error> {
    let hash = block::get_transaction_hash(transaction)?;
    let signature = keypair.sign(hash.bytes())?;
    transaction.set_hash(hash);
    transaction.set_signature(signature.to_vec());
    Ok(())
}

fn u64_to_u32(field: &str, value: u64) -> Result<u32, Error> {
    u32::try_from(value)
        .map_err(|_| Error::InvalidInput(format!("{field} value {value} overflows u32")))
}

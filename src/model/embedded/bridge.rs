//! Bridge contract models.

use crate::error::Error;
use crate::model::embedded::common::TimeChallengeInfo;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::TokenStandard;
use num_bigint::BigUint;
use serde_json::{Map, Value, json};

/// Bridge administrator info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeInfo {
    administrator: Address,
    compressed_tss_ecdsa_pub_key: String,
    decompressed_tss_ecdsa_pub_key: String,
    allow_key_gen: bool,
    halted: bool,
    unhalted_at: u64,
    unhalt_duration_in_momentums: u64,
    tss_nonce: u64,
    metadata: String,
}

impl BridgeInfo {
    /// Creates bridge info.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        administrator: Address,
        compressed_tss_ecdsa_pub_key: String,
        decompressed_tss_ecdsa_pub_key: String,
        allow_key_gen: bool,
        halted: bool,
        unhalted_at: u64,
        unhalt_duration_in_momentums: u64,
        tss_nonce: u64,
        metadata: String,
    ) -> Self {
        Self {
            administrator,
            compressed_tss_ecdsa_pub_key,
            decompressed_tss_ecdsa_pub_key,
            allow_key_gen,
            halted,
            unhalted_at,
            unhalt_duration_in_momentums,
            tss_nonce,
            metadata,
        }
    }

    /// Returns the administrator.
    pub fn administrator(&self) -> &Address {
        &self.administrator
    }
    /// Returns the compressed TSS ECDSA public key.
    pub fn compressed_tss_ecdsa_pub_key(&self) -> &str {
        &self.compressed_tss_ecdsa_pub_key
    }
    /// Returns the decompressed TSS ECDSA public key.
    pub fn decompressed_tss_ecdsa_pub_key(&self) -> &str {
        &self.decompressed_tss_ecdsa_pub_key
    }
    /// Returns whether key generation is allowed.
    pub fn allow_key_gen(&self) -> bool {
        self.allow_key_gen
    }
    /// Returns whether the bridge is halted.
    pub fn halted(&self) -> bool {
        self.halted
    }
    /// Returns the unhalted-at height.
    pub fn unhalted_at(&self) -> u64 {
        self.unhalted_at
    }
    /// Returns the unhalt duration in momentums.
    pub fn unhalt_duration_in_momentums(&self) -> u64 {
        self.unhalt_duration_in_momentums
    }
    /// Returns the TSS nonce.
    pub fn tss_nonce(&self) -> u64 {
        self.tss_nonce
    }
    /// Returns the metadata.
    pub fn metadata(&self) -> &str {
        &self.metadata
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "administrator": self.administrator.to_string(),
            "compressedTssECDSAPubKey": self.compressed_tss_ecdsa_pub_key,
            "decompressedTssECDSAPubKey": self.decompressed_tss_ecdsa_pub_key,
            "allowKeyGen": self.allow_key_gen,
            "halted": self.halted,
            "unhaltedAt": self.unhalted_at,
            "unhaltDurationInMomentums": self.unhalt_duration_in_momentums,
            "tssNonce": self.tss_nonce,
            "metadata": self.metadata,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "bridge info")?;
        Ok(Self::new(
            Address::parse(required_str(object, "administrator")?)?,
            required_str(object, "compressedTssECDSAPubKey")?.to_string(),
            required_str(object, "decompressedTssECDSAPubKey")?.to_string(),
            required_bool(object, "allowKeyGen")?,
            required_bool(object, "halted")?,
            required_u64(object, "unhaltedAt")?,
            required_u64(object, "unhaltDurationInMomentums")?,
            required_u64(object, "tssNonce")?,
            required_str(object, "metadata")?.to_string(),
        ))
    }
}

/// Orchestrator info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratorInfo {
    window_size: u64,
    key_gen_threshold: u64,
    confirmations_to_finality: u64,
    estimated_momentum_time: u64,
    allow_key_gen_height: u64,
}

impl OrchestratorInfo {
    /// Creates orchestrator info.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window_size: u64,
        key_gen_threshold: u64,
        confirmations_to_finality: u64,
        estimated_momentum_time: u64,
        allow_key_gen_height: u64,
    ) -> Self {
        Self {
            window_size,
            key_gen_threshold,
            confirmations_to_finality,
            estimated_momentum_time,
            allow_key_gen_height,
        }
    }

    /// Returns the window size.
    pub fn window_size(&self) -> u64 {
        self.window_size
    }
    /// Returns the key-generation threshold.
    pub fn key_gen_threshold(&self) -> u64 {
        self.key_gen_threshold
    }
    /// Returns the confirmations to finality.
    pub fn confirmations_to_finality(&self) -> u64 {
        self.confirmations_to_finality
    }
    /// Returns the estimated momentum time.
    pub fn estimated_momentum_time(&self) -> u64 {
        self.estimated_momentum_time
    }
    /// Returns the allow-key-gen height.
    pub fn allow_key_gen_height(&self) -> u64 {
        self.allow_key_gen_height
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "windowSize": self.window_size,
            "keyGenThreshold": self.key_gen_threshold,
            "confirmationsToFinality": self.confirmations_to_finality,
            "estimatedMomentumTime": self.estimated_momentum_time,
            "allowKeyGenHeight": self.allow_key_gen_height,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "orchestrator info")?;
        Ok(Self::new(
            required_u64(object, "windowSize")?,
            required_u64(object, "keyGenThreshold")?,
            required_u64(object, "confirmationsToFinality")?,
            required_u64(object, "estimatedMomentumTime")?,
            required_u64(object, "allowKeyGenHeight")?,
        ))
    }
}

/// A bridgeable token pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPair {
    token_standard: TokenStandard,
    token_address: String,
    bridgeable: bool,
    redeemable: bool,
    owned: bool,
    min_amount: BigUint,
    fee_percentage: u64,
    redeem_delay: u64,
    metadata: String,
}

impl TokenPair {
    /// Creates a token pair.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        token_standard: TokenStandard,
        token_address: String,
        bridgeable: bool,
        redeemable: bool,
        owned: bool,
        min_amount: BigUint,
        fee_percentage: u64,
        redeem_delay: u64,
        metadata: String,
    ) -> Self {
        Self {
            token_standard,
            token_address,
            bridgeable,
            redeemable,
            owned,
            min_amount,
            fee_percentage,
            redeem_delay,
            metadata,
        }
    }

    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }
    /// Returns the token address.
    pub fn token_address(&self) -> &str {
        &self.token_address
    }
    /// Returns whether the pair is bridgeable.
    pub fn bridgeable(&self) -> bool {
        self.bridgeable
    }
    /// Returns whether the pair is redeemable.
    pub fn redeemable(&self) -> bool {
        self.redeemable
    }
    /// Returns whether the pair is owned.
    pub fn owned(&self) -> bool {
        self.owned
    }
    /// Returns the minimum amount.
    pub fn min_amount(&self) -> &BigUint {
        &self.min_amount
    }
    /// Returns the fee percentage.
    pub fn fee_percentage(&self) -> u64 {
        self.fee_percentage
    }
    /// Returns the redeem delay.
    pub fn redeem_delay(&self) -> u64 {
        self.redeem_delay
    }
    /// Returns the metadata.
    pub fn metadata(&self) -> &str {
        &self.metadata
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "tokenStandard": self.token_standard.to_string(),
            "tokenAddress": self.token_address,
            "bridgeable": self.bridgeable,
            "redeemable": self.redeemable,
            "owned": self.owned,
            "minAmount": self.min_amount.to_string(),
            "feePercentage": self.fee_percentage,
            "redeemDelay": self.redeem_delay,
            "metadata": self.metadata,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "token pair")?;
        Ok(Self::new(
            TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            required_str(object, "tokenAddress")?.to_string(),
            required_bool(object, "bridgeable")?,
            required_bool(object, "redeemable")?,
            required_bool(object, "owned")?,
            required_big_uint(object, "minAmount")?,
            required_u64(object, "feePercentage")?,
            required_u64(object, "redeemDelay")?,
            required_str(object, "metadata")?.to_string(),
        ))
    }
}

/// Bridge network info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNetworkInfo {
    network_class: u64,
    chain_id: u64,
    name: String,
    contract_address: String,
    metadata: String,
    token_pairs: Vec<TokenPair>,
}

impl BridgeNetworkInfo {
    /// Creates bridge network info.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_class: u64,
        chain_id: u64,
        name: String,
        contract_address: String,
        metadata: String,
        token_pairs: Vec<TokenPair>,
    ) -> Self {
        Self {
            network_class,
            chain_id,
            name,
            contract_address,
            metadata,
            token_pairs,
        }
    }

    /// Returns the network class.
    pub fn network_class(&self) -> u64 {
        self.network_class
    }
    /// Returns the chain id.
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
    /// Returns the network name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the contract address.
    pub fn contract_address(&self) -> &str {
        &self.contract_address
    }
    /// Returns the metadata.
    pub fn metadata(&self) -> &str {
        &self.metadata
    }
    /// Returns the token pairs.
    pub fn token_pairs(&self) -> &[TokenPair] {
        &self.token_pairs
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "networkClass": self.network_class,
            "chainId": self.chain_id,
            "name": self.name,
            "contractAddress": self.contract_address,
            "metadata": self.metadata,
            "tokenPairs": self.token_pairs.iter().map(TokenPair::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "bridge network info")?;
        Ok(Self::new(
            required_u64(object, "networkClass")?,
            required_u64(object, "chainId")?,
            required_str(object, "name")?.to_string(),
            required_str(object, "contractAddress")?.to_string(),
            required_str(object, "metadata")?.to_string(),
            null_safe_array(object, "tokenPairs", TokenPair::from_json)?,
        ))
    }
}

/// A paged list of bridge network info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNetworkInfoList {
    count: u64,
    list: Vec<BridgeNetworkInfo>,
}

impl BridgeNetworkInfoList {
    /// Creates a bridge network info list.
    pub fn new(count: u64, list: Vec<BridgeNetworkInfo>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }
    /// Returns the list.
    pub fn list(&self) -> &[BridgeNetworkInfo] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(BridgeNetworkInfo::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "bridge network info list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", BridgeNetworkInfo::from_json)?,
        ))
    }
}

/// A wrap-token request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapTokenRequest {
    network_class: u64,
    chain_id: u64,
    id: Hash,
    to_address: String,
    token_standard: TokenStandard,
    token_address: String,
    amount: BigUint,
    fee: BigUint,
    signature: String,
    creation_momentum_height: u64,
    confirmations_to_finality: u64,
}

impl WrapTokenRequest {
    /// Creates a wrap-token request.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_class: u64,
        chain_id: u64,
        id: Hash,
        to_address: String,
        token_standard: TokenStandard,
        token_address: String,
        amount: BigUint,
        fee: BigUint,
        signature: String,
        creation_momentum_height: u64,
        confirmations_to_finality: u64,
    ) -> Self {
        Self {
            network_class,
            chain_id,
            id,
            to_address,
            token_standard,
            token_address,
            amount,
            fee,
            signature,
            creation_momentum_height,
            confirmations_to_finality,
        }
    }

    /// Returns the network class.
    pub fn network_class(&self) -> u64 {
        self.network_class
    }
    /// Returns the chain id.
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }
    /// Returns the destination address.
    pub fn to_address(&self) -> &str {
        &self.to_address
    }
    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }
    /// Returns the token address.
    pub fn token_address(&self) -> &str {
        &self.token_address
    }
    /// Returns the amount.
    pub fn amount(&self) -> &BigUint {
        &self.amount
    }
    /// Returns the fee.
    pub fn fee(&self) -> &BigUint {
        &self.fee
    }
    /// Returns the signature.
    pub fn signature(&self) -> &str {
        &self.signature
    }
    /// Returns the creation momentum height.
    pub fn creation_momentum_height(&self) -> u64 {
        self.creation_momentum_height
    }
    /// Returns the confirmations to finality.
    pub fn confirmations_to_finality(&self) -> u64 {
        self.confirmations_to_finality
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "networkClass": self.network_class,
            "chainId": self.chain_id,
            "id": self.id.to_string(),
            "toAddress": self.to_address,
            "tokenStandard": self.token_standard.to_string(),
            "tokenAddress": self.token_address,
            "amount": self.amount.to_string(),
            "fee": self.fee.to_string(),
            "signature": self.signature,
            "creationMomentumHeight": self.creation_momentum_height,
            "confirmationsToFinality": self.confirmations_to_finality,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "wrap token request")?;
        Ok(Self::new(
            required_u64(object, "networkClass")?,
            required_u64(object, "chainId")?,
            Hash::parse(required_str(object, "id")?)?,
            required_str(object, "toAddress")?.to_string(),
            TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            required_str(object, "tokenAddress")?.to_string(),
            required_big_uint(object, "amount")?,
            required_big_uint(object, "fee")?,
            required_str(object, "signature")?.to_string(),
            required_u64(object, "creationMomentumHeight")?,
            required_u64(object, "confirmationsToFinality")?,
        ))
    }
}

/// A paged list of wrap-token requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapTokenRequestList {
    count: u64,
    list: Vec<WrapTokenRequest>,
}

impl WrapTokenRequestList {
    /// Creates a wrap-token request list.
    pub fn new(count: u64, list: Vec<WrapTokenRequest>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }
    /// Returns the list.
    pub fn list(&self) -> &[WrapTokenRequest] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(WrapTokenRequest::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "wrap token request list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            null_safe_array(object, "list", WrapTokenRequest::from_json)?,
        ))
    }
}

/// An unwrap-token request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwrapTokenRequest {
    registration_momentum_height: u64,
    network_class: u64,
    chain_id: u64,
    transaction_hash: Hash,
    log_index: u64,
    to_address: Address,
    token_address: String,
    token_standard: TokenStandard,
    amount: BigUint,
    signature: String,
    redeemed: u64,
    revoked: u64,
    redeemable_in: u64,
}

impl UnwrapTokenRequest {
    /// Creates an unwrap-token request.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registration_momentum_height: u64,
        network_class: u64,
        chain_id: u64,
        transaction_hash: Hash,
        log_index: u64,
        to_address: Address,
        token_address: String,
        token_standard: TokenStandard,
        amount: BigUint,
        signature: String,
        redeemed: u64,
        revoked: u64,
        redeemable_in: u64,
    ) -> Self {
        Self {
            registration_momentum_height,
            network_class,
            chain_id,
            transaction_hash,
            log_index,
            to_address,
            token_address,
            token_standard,
            amount,
            signature,
            redeemed,
            revoked,
            redeemable_in,
        }
    }

    /// Returns the registration momentum height.
    pub fn registration_momentum_height(&self) -> u64 {
        self.registration_momentum_height
    }
    /// Returns the network class.
    pub fn network_class(&self) -> u64 {
        self.network_class
    }
    /// Returns the chain id.
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
    /// Returns the transaction hash.
    pub fn transaction_hash(&self) -> &Hash {
        &self.transaction_hash
    }
    /// Returns the log index.
    pub fn log_index(&self) -> u64 {
        self.log_index
    }
    /// Returns the destination address.
    pub fn to_address(&self) -> &Address {
        &self.to_address
    }
    /// Returns the token address.
    pub fn token_address(&self) -> &str {
        &self.token_address
    }
    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }
    /// Returns the amount.
    pub fn amount(&self) -> &BigUint {
        &self.amount
    }
    /// Returns the signature.
    pub fn signature(&self) -> &str {
        &self.signature
    }
    /// Returns the redeemed flag.
    pub fn redeemed(&self) -> u64 {
        self.redeemed
    }
    /// Returns the revoked flag.
    pub fn revoked(&self) -> u64 {
        self.revoked
    }
    /// Returns the redeemable-in value.
    pub fn redeemable_in(&self) -> u64 {
        self.redeemable_in
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "registrationMomentumHeight": self.registration_momentum_height,
            "networkClass": self.network_class,
            "chainId": self.chain_id,
            "transactionHash": self.transaction_hash.to_string(),
            "logIndex": self.log_index,
            "toAddress": self.to_address.to_string(),
            "tokenAddress": self.token_address,
            "tokenStandard": self.token_standard.to_string(),
            "amount": self.amount.to_string(),
            "signature": self.signature,
            "redeemed": self.redeemed,
            "revoked": self.revoked,
            "redeemableIn": self.redeemable_in,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "unwrap token request")?;
        Ok(Self::new(
            required_u64(object, "registrationMomentumHeight")?,
            required_u64(object, "networkClass")?,
            required_u64(object, "chainId")?,
            Hash::parse(required_str(object, "transactionHash")?)?,
            required_u64(object, "logIndex")?,
            Address::parse(required_str(object, "toAddress")?)?,
            required_str(object, "tokenAddress")?.to_string(),
            TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            required_big_uint(object, "amount")?,
            required_str(object, "signature")?.to_string(),
            required_u64(object, "redeemed")?,
            required_u64(object, "revoked")?,
            required_u64(object, "redeemableIn")?,
        ))
    }
}

/// A paged list of unwrap-token requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwrapTokenRequestList {
    count: u64,
    list: Vec<UnwrapTokenRequest>,
}

impl UnwrapTokenRequestList {
    /// Creates an unwrap-token request list.
    pub fn new(count: u64, list: Vec<UnwrapTokenRequest>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }
    /// Returns the list.
    pub fn list(&self) -> &[UnwrapTokenRequest] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(UnwrapTokenRequest::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "unwrap token request list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            null_safe_array(object, "list", UnwrapTokenRequest::from_json)?,
        ))
    }
}

/// Accumulated fees for a token standard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZtsFeesInfo {
    token_standard: TokenStandard,
    accumulated_fee: BigUint,
}

impl ZtsFeesInfo {
    /// Creates zts fees info.
    pub fn new(token_standard: TokenStandard, accumulated_fee: BigUint) -> Self {
        Self {
            token_standard,
            accumulated_fee,
        }
    }

    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }
    /// Returns the accumulated fee.
    pub fn accumulated_fee(&self) -> &BigUint {
        &self.accumulated_fee
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "tokenStandard": self.token_standard.to_string(),
            "accumulatedFee": self.accumulated_fee.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "zts fees info")?;
        Ok(Self::new(
            TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            required_big_uint(object, "accumulatedFee")?,
        ))
    }
}

/// A paged list of time challenges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeChallengesList {
    count: u64,
    list: Vec<TimeChallengeInfo>,
}

impl TimeChallengesList {
    /// Creates a time challenges list.
    pub fn new(count: u64, list: Vec<TimeChallengeInfo>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }
    /// Returns the list.
    pub fn list(&self) -> &[TimeChallengeInfo] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(TimeChallengeInfo::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "time challenges list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", TimeChallengeInfo::from_json)?,
        ))
    }
}

fn null_safe_array<T, F>(object: &Map<String, Value>, field: &str, map: F) -> Result<Vec<T>, Error>
where
    F: Fn(&Value) -> Result<T, Error>,
{
    match object.get(field) {
        Some(Value::Array(values)) => values.iter().map(map).collect(),
        Some(Value::Null) => Ok(Vec::new()),
        None => Err(Error::InvalidInput(format!("missing {field} field"))),
        Some(_) => Err(Error::InvalidInput(format!("{field} must be an array"))),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Conformance {
        #[allow(dead_code)]
        description: String,
        bridge_info: Value,
        orchestrator_info: Value,
        token_pair: Value,
        bridge_network_info: Value,
        bridge_network_info_list: Value,
        wrap_token_request: Value,
        wrap_token_request_list: Value,
        unwrap_token_request: Value,
        unwrap_token_request_list: Value,
        zts_fees_info: Value,
        time_challenges_list: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/bridge.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid bridge conformance")
    }

    fn znn_ts() -> TokenStandard {
        TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap()
    }

    #[test]
    fn bridge_info_round_trip() {
        let value = conf().bridge_info;
        let info = BridgeInfo::from_json(&value).expect("conformance parses");
        assert_eq!(info.to_json(), value);
    }

    #[test]
    fn orchestrator_info_round_trip() {
        let value = conf().orchestrator_info;
        let info = OrchestratorInfo::from_json(&value).expect("conformance parses");
        assert_eq!(info.to_json(), value);
    }

    #[test]
    fn token_pair_round_trip() {
        let original = TokenPair::new(
            znn_ts(),
            "0xabc".to_string(),
            true,
            true,
            false,
            BigUint::from(1_000_000u64),
            10,
            20,
            String::new(),
        );
        let round_trip = TokenPair::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn bridge_network_info_round_trip() {
        let value = conf().bridge_network_info;
        let info = BridgeNetworkInfo::from_json(&value).expect("conformance parses");
        assert_eq!(info.to_json(), value);
    }

    #[test]
    fn bridge_network_info_null_token_pairs_defaults_empty() {
        let mut nulled = conf().bridge_network_info;
        nulled["tokenPairs"] = Value::Null;
        let info = BridgeNetworkInfo::from_json(&nulled).expect("null tokenPairs parses");
        assert!(info.token_pairs().is_empty());

        // A populated tokenPairs must parse its elements.
        assert_eq!(
            BridgeNetworkInfo::from_json(&conf().bridge_network_info)
                .expect("conformance parses")
                .token_pairs()
                .len(),
            1
        );
    }

    #[test]
    fn bridge_network_info_rejects_non_array_token_pairs() {
        let mut bad = conf().bridge_network_info;
        bad["tokenPairs"] = json!("not-an-array");
        let result = BridgeNetworkInfo::from_json(&bad);
        assert!(result.is_err(), "non-array tokenPairs must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut missing = conf().bridge_network_info;
        missing.as_object_mut().unwrap().remove("tokenPairs");
        let result = BridgeNetworkInfo::from_json(&missing);
        assert!(result.is_err(), "missing tokenPairs must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn bridge_network_info_list_round_trip() {
        let value = conf().bridge_network_info_list;
        let list = BridgeNetworkInfoList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
    }

    #[test]
    fn wrap_token_request_round_trip() {
        let original = WrapTokenRequest::new(
            1,
            1,
            Hash::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
            "0xrecipient".to_string(),
            znn_ts(),
            "0xabc".to_string(),
            BigUint::from(100_000_000u64),
            BigUint::from(1_000_000u64),
            "0xsig".to_string(),
            50,
            10,
        );
        let round_trip =
            WrapTokenRequest::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn wrap_token_request_list_null_list_defaults_empty() {
        let mut nulled = conf().wrap_token_request_list;
        nulled["list"] = Value::Null;
        let list = WrapTokenRequestList::from_json(&nulled).expect("null list parses");
        assert!(list.list().is_empty());

        // A populated list must parse its elements.
        let populated = json!({"count": 1, "list": [conf().wrap_token_request]});
        assert_eq!(
            WrapTokenRequestList::from_json(&populated)
                .expect("populated parses")
                .list()
                .len(),
            1
        );

        let missing = json!({"count": 1});
        let result = WrapTokenRequestList::from_json(&missing);
        assert!(result.is_err(), "missing list must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn unwrap_token_request_round_trip() {
        let original = UnwrapTokenRequest::new(
            40,
            1,
            1,
            Hash::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
            0,
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            "0xabc".to_string(),
            znn_ts(),
            BigUint::from(100_000_000u64),
            "0xsig".to_string(),
            0,
            0,
            5,
        );
        let round_trip =
            UnwrapTokenRequest::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn unwrap_token_request_list_null_list_defaults_empty() {
        let mut nulled = conf().unwrap_token_request_list;
        nulled["list"] = Value::Null;
        let list = UnwrapTokenRequestList::from_json(&nulled).expect("null list parses");
        assert!(list.list().is_empty());

        // A populated list must parse its elements.
        let populated = json!({"count": 1, "list": [conf().unwrap_token_request]});
        assert_eq!(
            UnwrapTokenRequestList::from_json(&populated)
                .expect("populated parses")
                .list()
                .len(),
            1
        );

        let missing = json!({"count": 1});
        let result = UnwrapTokenRequestList::from_json(&missing);
        assert!(result.is_err(), "missing list must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn zts_fees_info_round_trip() {
        let original = ZtsFeesInfo::new(znn_ts(), BigUint::from(500_000_000u64));
        let round_trip = ZtsFeesInfo::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn time_challenges_list_round_trip_preserves_pascalcase_keys() {
        let value = conf().time_challenges_list;
        let list = TimeChallengesList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
    }
}

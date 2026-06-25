//! Zenon `AccountBlockTemplate` ledger model and `BlockType` enum.
//!
//! [`AccountBlockTemplate`] is the unsigned account-block shape assembled before
//! signing. Its JSON form encodes the `data`, `public_key`, and `signature` byte
//! fields as standard base64.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::{
    Address, CORE_SIZE as ADDRESS_CORE_SIZE, PREFIX as ADDRESS_PREFIX,
};
use crate::primitives::hash::Hash;
use crate::primitives::hash_height::HashHeight;
use crate::primitives::token_standard::{TokenStandard, empty_token_standard};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use num_bigint::BigUint;
use serde_json::{Value, json};

/// Default chain identifier for a freshly assembled template.
pub const DEFAULT_CHAIN_IDENTIFIER: u32 = 1;

/// Account-block type ordinals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BlockType {
    /// Unknown block type.
    Unknown = 0,
    /// A genesis receive.
    GenesisReceive = 1,
    /// A user send.
    UserSend = 2,
    /// A user receive.
    UserReceive = 3,
    /// A contract send.
    ContractSend = 4,
    /// A contract receive.
    ContractReceive = 5,
}

impl BlockType {
    /// Returns the ordinal of this block type.
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Parses a block-type ordinal, returning `None` for an out-of-range value.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::GenesisReceive),
            2 => Some(Self::UserSend),
            3 => Some(Self::UserReceive),
            4 => Some(Self::ContractSend),
            5 => Some(Self::ContractReceive),
            _ => None,
        }
    }
}

/// An unsigned account-block template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountBlockTemplate {
    version: u32,
    chain_identifier: u32,
    block_type: BlockType,
    hash: Hash,
    previous_hash: Hash,
    height: u64,
    momentum_acknowledged: HashHeight,
    address: Address,
    to_address: Address,
    amount: BigUint,
    token_standard: TokenStandard,
    from_block_hash: Hash,
    data: Vec<u8>,
    fused_plasma: u32,
    difficulty: u32,
    nonce: String,
    public_key: Vec<u8>,
    signature: Vec<u8>,
}

impl AccountBlockTemplate {
    /// Creates a template for `block_type`, defaulting every other field.
    pub fn new(block_type: BlockType) -> Self {
        Self {
            version: 1,
            chain_identifier: DEFAULT_CHAIN_IDENTIFIER,
            block_type,
            hash: Hash::empty(),
            previous_hash: Hash::empty(),
            height: 0,
            momentum_acknowledged: HashHeight::empty(),
            address: empty_address(),
            to_address: empty_address(),
            amount: BigUint::from(0u32),
            token_standard: empty_token_standard(),
            from_block_hash: Hash::empty(),
            data: Vec::new(),
            fused_plasma: 0,
            difficulty: 0,
            nonce: String::new(),
            public_key: Vec::new(),
            signature: Vec::new(),
        }
    }

    /// Creates a `userReceive` template for the given from-block hash.
    pub fn receive(from_block_hash: Hash) -> Self {
        let mut template = Self::new(BlockType::UserReceive);
        template.from_block_hash = from_block_hash;
        template
    }

    /// Creates a `userSend` template transferring `amount` of `token_standard`
    /// to `to_address`, with optional `data`.
    pub fn send(
        to_address: Address,
        token_standard: TokenStandard,
        amount: BigUint,
        data: Option<Vec<u8>>,
    ) -> Self {
        let mut template = Self::new(BlockType::UserSend);
        template.to_address = to_address;
        template.token_standard = token_standard;
        template.amount = amount;
        template.data = data.unwrap_or_default();
        template
    }

    /// Creates a `userSend` template calling the contract at `to_address` with
    /// `amount` and `data`.
    pub fn call_contract(
        to_address: Address,
        token_standard: TokenStandard,
        amount: BigUint,
        data: Vec<u8>,
    ) -> Self {
        Self::send(to_address, token_standard, amount, Some(data))
    }

    /// Builder that sets the proof-of-work `nonce` as a hex string and returns
    /// the template for chaining.
    #[must_use]
    pub fn with_nonce(mut self, nonce: impl Into<String>) -> Self {
        self.nonce = nonce.into();
        self
    }

    /// Sets the template hash.
    pub fn set_hash(&mut self, hash: Hash) {
        self.hash = hash;
    }

    /// Sets the previous account-block hash.
    pub fn set_previous_hash(&mut self, previous_hash: Hash) {
        self.previous_hash = previous_hash;
    }

    /// Sets the account-block height.
    pub fn set_height(&mut self, height: u64) {
        self.height = height;
    }

    /// Sets the acknowledged momentum.
    pub fn set_momentum_acknowledged(&mut self, momentum_acknowledged: HashHeight) {
        self.momentum_acknowledged = momentum_acknowledged;
    }

    /// Sets the signer address.
    pub fn set_address(&mut self, address: Address) {
        self.address = address;
    }

    /// Sets the fused plasma value.
    pub fn set_fused_plasma(&mut self, fused_plasma: u32) {
        self.fused_plasma = fused_plasma;
    }

    /// Sets the proof-of-work difficulty.
    pub fn set_difficulty(&mut self, difficulty: u32) {
        self.difficulty = difficulty;
    }

    /// Sets the proof-of-work nonce as a hex string.
    pub fn set_nonce(&mut self, nonce: impl Into<String>) {
        self.nonce = nonce.into();
    }

    /// Sets the public key bytes.
    pub fn set_public_key(&mut self, public_key: Vec<u8>) {
        self.public_key = public_key;
    }

    /// Sets the signature bytes.
    pub fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = signature;
    }

    /// Returns the version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns the chain identifier.
    pub fn chain_identifier(&self) -> u32 {
        self.chain_identifier
    }

    /// Returns the block type ordinal.
    pub fn block_type(&self) -> u32 {
        self.block_type.as_u32()
    }

    /// Returns the block type enum.
    pub fn block_type_enum(&self) -> BlockType {
        self.block_type
    }

    /// Returns the hash.
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    /// Returns the previous hash.
    pub fn previous_hash(&self) -> &Hash {
        &self.previous_hash
    }

    /// Returns the height.
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Returns the acknowledged momentum.
    pub fn momentum_acknowledged(&self) -> &HashHeight {
        &self.momentum_acknowledged
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the destination address.
    pub fn to_address(&self) -> &Address {
        &self.to_address
    }

    /// Returns the amount.
    pub fn amount(&self) -> &BigUint {
        &self.amount
    }

    /// Returns the token standard.
    pub fn token_standard(&self) -> &TokenStandard {
        &self.token_standard
    }

    /// Returns the from-block hash.
    pub fn from_block_hash(&self) -> &Hash {
        &self.from_block_hash
    }

    /// Returns the data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the fused plasma.
    pub fn fused_plasma(&self) -> u32 {
        self.fused_plasma
    }

    /// Returns the difficulty.
    pub fn difficulty(&self) -> u32 {
        self.difficulty
    }

    /// Returns the nonce.
    pub fn nonce(&self) -> &str {
        &self.nonce
    }

    /// Returns the public key.
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Returns the signature.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Serializes the template to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "chainIdentifier": self.chain_identifier,
            "blockType": self.block_type.as_u32(),
            "hash": self.hash.to_string(),
            "previousHash": self.previous_hash.to_string(),
            "height": self.height,
            "momentumAcknowledged": self.momentum_acknowledged.to_json(),
            "address": self.address.to_string(),
            "toAddress": self.to_address.to_string(),
            "amount": self.amount.to_string(),
            "tokenStandard": self.token_standard.to_string(),
            "fromBlockHash": self.from_block_hash.to_string(),
            "data": STANDARD.encode(&self.data),
            "fusedPlasma": self.fused_plasma,
            "difficulty": self.difficulty,
            "nonce": self.nonce,
            "publicKey": STANDARD.encode(&self.public_key),
            "signature": STANDARD.encode(&self.signature),
        })
    }

    /// Deserializes a template from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "account block template")?;
        let block_type = BlockType::from_u32(required_u32(object, "blockType")?)
            .ok_or_else(|| Error::InvalidInput("blockType is out of range".into()))?;

        Ok(Self {
            version: required_u32(object, "version")?,
            chain_identifier: required_u32(object, "chainIdentifier")?,
            block_type,
            hash: Hash::parse(required_str(object, "hash")?)?,
            previous_hash: Hash::parse(required_str(object, "previousHash")?)?,
            height: required_u64(object, "height")?,
            momentum_acknowledged: HashHeight::from_json(required_value(
                object,
                "momentumAcknowledged",
            )?)?,
            address: Address::parse(required_str(object, "address")?)?,
            to_address: Address::parse(required_str(object, "toAddress")?)?,
            amount: required_big_uint(object, "amount")?,
            token_standard: TokenStandard::parse(required_str(object, "tokenStandard")?)?,
            from_block_hash: Hash::parse(required_str(object, "fromBlockHash")?)?,
            data: required_base64(object, "data")?,
            fused_plasma: required_u32(object, "fusedPlasma")?,
            difficulty: required_u32(object, "difficulty")?,
            nonce: required_str(object, "nonce")?.to_string(),
            public_key: required_base64(object, "publicKey")?,
            signature: required_base64(object, "signature")?,
        })
    }
}

#[allow(clippy::expect_used)]
fn empty_address() -> Address {
    Address::new(ADDRESS_PREFIX, &[0u8; ADDRESS_CORE_SIZE])
        .expect("20 zero bytes form a valid address core")
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn block_type_ordinals_match_expected_values() {
        assert_eq!(BlockType::Unknown.as_u32(), 0);
        assert_eq!(BlockType::GenesisReceive.as_u32(), 1);
        assert_eq!(BlockType::UserSend.as_u32(), 2);
        assert_eq!(BlockType::UserReceive.as_u32(), 3);
        assert_eq!(BlockType::ContractSend.as_u32(), 4);
        assert_eq!(BlockType::ContractReceive.as_u32(), 5);
    }

    #[test]
    fn block_type_from_u32_round_trips_known_ordinals() {
        for ordinal in 0..=5 {
            let variant = BlockType::from_u32(ordinal).expect("known ordinal maps to a variant");
            assert_eq!(variant.as_u32(), ordinal);
        }
    }

    #[test]
    fn block_type_from_u32_rejects_unknown_ordinals() {
        assert!(BlockType::from_u32(6).is_none());
        assert!(BlockType::from_u32(u32::MAX).is_none());
    }

    #[test]
    fn new_sets_the_default_fields() {
        let template = AccountBlockTemplate::new(BlockType::UserSend);
        assert_eq!(template.version(), 1);
        assert_eq!(template.chain_identifier(), DEFAULT_CHAIN_IDENTIFIER);
        assert_eq!(template.block_type(), 2);
        assert_eq!(*template.hash(), Hash::empty());
        assert_eq!(*template.previous_hash(), Hash::empty());
        assert_eq!(template.height(), 0);
        assert_eq!(template.momentum_acknowledged(), &HashHeight::empty());
        assert_eq!(*template.amount(), BigUint::from(0u32));
        assert_eq!(
            template.token_standard().to_string(),
            "zts1qqqqqqqqqqqqqqqqtq587y"
        );
        assert!(template.data().is_empty());
        assert!(template.public_key().is_empty());
        assert!(template.signature().is_empty());
        assert_eq!(template.nonce(), "");
        assert_eq!(template.fused_plasma(), 0);
        assert_eq!(template.difficulty(), 0);
    }

    #[test]
    fn receive_factory_sets_user_receive_and_from_block_hash() {
        let from = Hash::parse("3835082b4afb76971d58d6ad510e7e91f3bb0d41912fac4ec4cfef7bd7bbea73")
            .unwrap();
        let template = AccountBlockTemplate::receive(from.clone());
        assert_eq!(template.block_type(), 3);
        assert_eq!(*template.from_block_hash(), from);
    }

    #[test]
    fn send_factory_sets_user_send_and_send_fields() {
        let to = Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap();
        let ts = TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap();
        let template =
            AccountBlockTemplate::send(to.clone(), ts.clone(), BigUint::from(100u64), None);
        assert_eq!(template.block_type(), 2);
        assert_eq!(*template.to_address(), to);
        assert_eq!(*template.token_standard(), ts);
        assert_eq!(*template.amount(), BigUint::from(100u64));
        assert!(template.data().is_empty());
    }

    #[test]
    fn call_contract_factory_sets_user_send_and_data() {
        let to = Address::parse("z1qxemdeddedxswapxxxxxxxxxxxxxxxxxxl4yww").unwrap();
        let ts = TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap();
        let template = AccountBlockTemplate::call_contract(
            to.clone(),
            ts.clone(),
            BigUint::from(0u32),
            vec![1, 2, 3],
        );
        assert_eq!(template.block_type(), 2);
        assert_eq!(*template.to_address(), to);
        assert_eq!(template.data(), &[1, 2, 3]);
    }
}

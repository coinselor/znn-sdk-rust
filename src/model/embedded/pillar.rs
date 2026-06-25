//! Pillar contract models.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use num_bigint::BigUint;
use serde_json::{Value, json};

/// Pillar type ordinal: an unknown pillar.
pub const UNKNOWN_TYPE: u64 = 0;
/// Pillar type ordinal: a legacy pillar.
pub const LEGACY_PILLAR_TYPE: u64 = 1;
/// Pillar type ordinal: a regular pillar.
pub const REGULAR_PILLAR_TYPE: u64 = 2;

/// Pillar epoch statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PillarEpochStats {
    produced_momentums: u64,
    expected_momentums: u64,
}

impl PillarEpochStats {
    /// Creates epoch stats.
    pub fn new(produced_momentums: u64, expected_momentums: u64) -> Self {
        Self {
            produced_momentums,
            expected_momentums,
        }
    }

    /// Returns produced momentums.
    pub fn produced_momentums(&self) -> u64 {
        self.produced_momentums
    }
    /// Returns expected momentums.
    pub fn expected_momentums(&self) -> u64 {
        self.expected_momentums
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "producedMomentums": self.produced_momentums,
            "expectedMomentums": self.expected_momentums,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "pillar epoch stats")?;
        Ok(Self::new(
            required_u64(object, "producedMomentums")?,
            required_u64(object, "expectedMomentums")?,
        ))
    }
}

/// Pillar info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PillarInfo {
    name: String,
    rank: u64,
    pillar_type: u64,
    owner_address: Address,
    producer_address: Address,
    withdraw_address: Address,
    give_momentum_reward_percentage: u64,
    give_delegate_reward_percentage: u64,
    is_revocable: bool,
    revoke_cooldown: u64,
    revoke_timestamp: u64,
    current_stats: PillarEpochStats,
    weight: BigUint,
    produced_momentums: u64,
    expected_momentums: u64,
}

impl PillarInfo {
    /// Creates pillar info.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        rank: u64,
        pillar_type: u64,
        owner_address: Address,
        producer_address: Address,
        withdraw_address: Address,
        give_momentum_reward_percentage: u64,
        give_delegate_reward_percentage: u64,
        is_revocable: bool,
        revoke_cooldown: u64,
        revoke_timestamp: u64,
        current_stats: PillarEpochStats,
        weight: BigUint,
        produced_momentums: u64,
        expected_momentums: u64,
    ) -> Self {
        Self {
            name,
            rank,
            pillar_type,
            owner_address,
            producer_address,
            withdraw_address,
            give_momentum_reward_percentage,
            give_delegate_reward_percentage,
            is_revocable,
            revoke_cooldown,
            revoke_timestamp,
            current_stats,
            weight,
            produced_momentums,
            expected_momentums,
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the rank.
    pub fn rank(&self) -> u64 {
        self.rank
    }
    /// Returns the pillar type ordinal.
    pub fn pillar_type(&self) -> u64 {
        self.pillar_type
    }
    /// Returns the owner address.
    pub fn owner_address(&self) -> &Address {
        &self.owner_address
    }
    /// Returns the producer address.
    pub fn producer_address(&self) -> &Address {
        &self.producer_address
    }
    /// Returns the withdraw address.
    pub fn withdraw_address(&self) -> &Address {
        &self.withdraw_address
    }
    /// Returns the momentum reward percentage.
    pub fn give_momentum_reward_percentage(&self) -> u64 {
        self.give_momentum_reward_percentage
    }
    /// Returns the delegate reward percentage.
    pub fn give_delegate_reward_percentage(&self) -> u64 {
        self.give_delegate_reward_percentage
    }
    /// Returns whether revocable.
    pub fn is_revocable(&self) -> bool {
        self.is_revocable
    }
    /// Returns the revoke cooldown.
    pub fn revoke_cooldown(&self) -> u64 {
        self.revoke_cooldown
    }
    /// Returns the revoke timestamp.
    pub fn revoke_timestamp(&self) -> u64 {
        self.revoke_timestamp
    }
    /// Returns the current stats.
    pub fn current_stats(&self) -> &PillarEpochStats {
        &self.current_stats
    }
    /// Returns the weight.
    pub fn weight(&self) -> &BigUint {
        &self.weight
    }
    /// Returns produced momentums.
    pub fn produced_momentums(&self) -> u64 {
        self.produced_momentums
    }
    /// Returns expected momentums.
    pub fn expected_momentums(&self) -> u64 {
        self.expected_momentums
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "rank": self.rank,
            "type": self.pillar_type,
            "ownerAddress": self.owner_address.to_string(),
            "producerAddress": self.producer_address.to_string(),
            "withdrawAddress": self.withdraw_address.to_string(),
            "giveMomentumRewardPercentage": self.give_momentum_reward_percentage,
            "giveDelegateRewardPercentage": self.give_delegate_reward_percentage,
            "isRevocable": self.is_revocable,
            "revokeCooldown": self.revoke_cooldown,
            "revokeTimestamp": self.revoke_timestamp,
            "currentStats": self.current_stats.to_json(),
            "weight": self.weight.to_string(),
            "producedMomentums": self.produced_momentums,
            "expectedMomentums": self.expected_momentums,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "pillar info")?;
        let stats_value = required_value(object, "currentStats")?;
        let stats_object = json_object(stats_value, "currentStats")?;
        Ok(Self {
            name: required_str(object, "name")?.to_string(),
            rank: required_u64(object, "rank")?,
            pillar_type: optional_u64(object, "type")?.unwrap_or(UNKNOWN_TYPE),
            owner_address: Address::parse(required_str(object, "ownerAddress")?)?,
            producer_address: Address::parse(required_str(object, "producerAddress")?)?,
            withdraw_address: Address::parse(required_str(object, "withdrawAddress")?)?,
            give_momentum_reward_percentage: required_u64(object, "giveMomentumRewardPercentage")?,
            give_delegate_reward_percentage: required_u64(object, "giveDelegateRewardPercentage")?,
            is_revocable: required_bool(object, "isRevocable")?,
            revoke_cooldown: required_u64(object, "revokeCooldown")?,
            revoke_timestamp: required_u64(object, "revokeTimestamp")?,
            current_stats: PillarEpochStats::from_json(stats_value)?,
            weight: required_big_uint(object, "weight")?,
            produced_momentums: required_u64(stats_object, "producedMomentums")?,
            expected_momentums: required_u64(stats_object, "expectedMomentums")?,
        })
    }
}

/// A paged list of pillar info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PillarInfoList {
    count: u64,
    list: Vec<PillarInfo>,
}

impl PillarInfoList {
    /// Creates a pillar info list.
    pub fn new(count: u64, list: Vec<PillarInfo>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Returns the list.
    pub fn list(&self) -> &[PillarInfo] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(PillarInfo::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "pillar info list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", PillarInfo::from_json)?,
        ))
    }
}

/// A pillar epoch history entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PillarEpochHistory {
    name: String,
    epoch: u64,
    give_block_reward_percentage: u64,
    give_delegate_reward_percentage: u64,
    produced_block_num: u64,
    expected_block_num: u64,
    weight: BigUint,
}

impl PillarEpochHistory {
    /// Creates an epoch history entry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        epoch: u64,
        give_block_reward_percentage: u64,
        give_delegate_reward_percentage: u64,
        produced_block_num: u64,
        expected_block_num: u64,
        weight: BigUint,
    ) -> Self {
        Self {
            name,
            epoch,
            give_block_reward_percentage,
            give_delegate_reward_percentage,
            produced_block_num,
            expected_block_num,
            weight,
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Returns the block reward percentage.
    pub fn give_block_reward_percentage(&self) -> u64 {
        self.give_block_reward_percentage
    }
    /// Returns the delegate reward percentage.
    pub fn give_delegate_reward_percentage(&self) -> u64 {
        self.give_delegate_reward_percentage
    }
    /// Returns produced blocks.
    pub fn produced_block_num(&self) -> u64 {
        self.produced_block_num
    }
    /// Returns expected blocks.
    pub fn expected_block_num(&self) -> u64 {
        self.expected_block_num
    }
    /// Returns the weight.
    pub fn weight(&self) -> &BigUint {
        &self.weight
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "epoch": self.epoch,
            "giveBlockRewardPercentage": self.give_block_reward_percentage,
            "giveDelegateRewardPercentage": self.give_delegate_reward_percentage,
            "producedBlockNum": self.produced_block_num,
            "expectedBlockNum": self.expected_block_num,
            "weight": self.weight.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "pillar epoch history")?;
        Ok(Self::new(
            required_str(object, "name")?.to_string(),
            required_u64(object, "epoch")?,
            required_u64(object, "giveBlockRewardPercentage")?,
            required_u64(object, "giveDelegateRewardPercentage")?,
            required_u64(object, "producedBlockNum")?,
            required_u64(object, "expectedBlockNum")?,
            required_big_uint(object, "weight")?,
        ))
    }
}

/// A paged list of pillar epoch history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PillarEpochHistoryList {
    count: u64,
    list: Vec<PillarEpochHistory>,
}

impl PillarEpochHistoryList {
    /// Creates an epoch history list.
    pub fn new(count: u64, list: Vec<PillarEpochHistory>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Returns the list.
    pub fn list(&self) -> &[PillarEpochHistory] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(PillarEpochHistory::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "pillar epoch history list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", PillarEpochHistory::from_json)?,
        ))
    }
}

/// Delegation info for an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationInfo {
    name: String,
    status: u64,
    weight: BigUint,
}

impl DelegationInfo {
    /// Creates delegation info.
    pub fn new(name: String, status: u64, weight: BigUint) -> Self {
        Self {
            name,
            status,
            weight,
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the status.
    pub fn status(&self) -> u64 {
        self.status
    }

    /// Returns the weight.
    pub fn weight(&self) -> &BigUint {
        &self.weight
    }

    /// Returns whether the delegated pillar is active.
    pub fn is_pillar_active(&self) -> bool {
        self.status == 1
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "status": self.status,
            "weight": self.weight.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "delegation info")?;
        Ok(Self::new(
            required_str(object, "name")?.to_string(),
            required_u64(object, "status")?,
            required_big_uint(object, "weight")?,
        ))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Conformance {
        #[allow(dead_code)]
        description: String,
        pillar_epoch_stats: Value,
        pillar_info: Value,
        pillar_info_list: Value,
        pillar_epoch_history: Value,
        pillar_epoch_history_list: Value,
        delegation_info_active: Value,
        delegation_info_inactive: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/pillar.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid pillar conformance")
    }

    #[test]
    fn pillar_info_round_trip() {
        let original = PillarInfo::new(
            "Pillar1".to_string(),
            1,
            REGULAR_PILLAR_TYPE,
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap(),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            25,
            25,
            false,
            0,
            0,
            PillarEpochStats::new(10, 12),
            BigUint::from(100_000_000_000u64),
            10,
            12,
        );
        let round_trip = PillarInfo::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn pillar_info_type_defaults_to_unknown_when_absent() {
        let mut missing = conf().pillar_info;
        missing.as_object_mut().unwrap().remove("type");
        let info = PillarInfo::from_json(&missing).expect("missing type parses");
        assert_eq!(info.pillar_type(), UNKNOWN_TYPE);

        // A present type must be read.
        assert_eq!(
            PillarInfo::from_json(&conf().pillar_info)
                .expect("conformance parses")
                .pillar_type(),
            REGULAR_PILLAR_TYPE
        );
    }

    #[test]
    fn pillar_info_current_stats_nesting() {
        let info = PillarInfo::from_json(&conf().pillar_info).expect("conformance parses");
        assert_eq!(info.produced_momentums(), 10);
        assert_eq!(info.expected_momentums(), 12);
        assert_eq!(info.name(), "Pillar1");
        assert_eq!(info.pillar_type(), REGULAR_PILLAR_TYPE);
    }

    #[test]
    fn pillar_info_list_round_trip() {
        let value = conf().pillar_info_list;
        let list = PillarInfoList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
        assert_eq!(list.count(), 1);
    }

    #[test]
    fn pillar_epoch_history_round_trip() {
        let value = conf().pillar_epoch_history;
        let entry = PillarEpochHistory::from_json(&value).expect("conformance parses");
        assert_eq!(entry.to_json(), value);
    }

    #[test]
    fn pillar_epoch_history_list_round_trip() {
        let value = conf().pillar_epoch_history_list;
        let list = PillarEpochHistoryList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
    }

    #[test]
    fn delegation_info_is_pillar_active_reflects_status() {
        let active = DelegationInfo::new("P".to_string(), 1, BigUint::from(0u32));
        assert!(active.is_pillar_active(), "status 1 is active");

        let inactive = DelegationInfo::new("P".to_string(), 0, BigUint::from(0u32));
        assert!(!inactive.is_pillar_active(), "status 0 is not active");
    }

    #[test]
    fn delegation_info_equal_when_name_status_weight_match() {
        let a = DelegationInfo::new("Pillar1".to_string(), 1, BigUint::from(100u64));
        let b = DelegationInfo::new("Pillar1".to_string(), 1, BigUint::from(100u64));
        assert_eq!(a, b);
    }

    #[test]
    fn delegation_info_not_equal_when_weight_differs() {
        let a = DelegationInfo::new("Pillar1".to_string(), 1, BigUint::from(100u64));
        let b = DelegationInfo::new("Pillar1".to_string(), 1, BigUint::from(99u64));
        assert_ne!(a, b);
    }

    #[test]
    fn delegation_info_round_trip() {
        let original = DelegationInfo::new("Pillar1".to_string(), 1, BigUint::from(100u64));
        let round_trip = DelegationInfo::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn pillar_epoch_stats_round_trip() {
        let original = PillarEpochStats::new(10, 12);
        let round_trip =
            PillarEpochStats::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }
}

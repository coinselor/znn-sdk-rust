//! Embedded-contract shared models reused across the embedded contract
//! types: rewards, votes, security, deposits, and challenge info.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use num_bigint::BigUint;
use serde_json::{Value, json};

/// Uncollected reward for an address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncollectedReward {
    address: Address,
    znn_amount: BigUint,
    qsr_amount: BigUint,
}

impl UncollectedReward {
    /// Creates an uncollected reward.
    pub fn new(address: Address, znn_amount: BigUint, qsr_amount: BigUint) -> Self {
        Self {
            address,
            znn_amount,
            qsr_amount,
        }
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the ZNN amount.
    pub fn znn_amount(&self) -> &BigUint {
        &self.znn_amount
    }

    /// Returns the QSR amount.
    pub fn qsr_amount(&self) -> &BigUint {
        &self.qsr_amount
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "address": self.address.to_string(),
            "znnAmount": self.znn_amount.to_string(),
            "qsrAmount": self.qsr_amount.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "uncollected reward")?;
        Ok(Self::new(
            Address::parse(required_str(object, "address")?)?,
            required_big_uint(object, "znnAmount")?,
            required_big_uint(object, "qsrAmount")?,
        ))
    }
}

/// A single reward history entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardHistoryEntry {
    epoch: u64,
    znn_amount: BigUint,
    qsr_amount: BigUint,
}

impl RewardHistoryEntry {
    /// Creates a reward history entry.
    pub fn new(epoch: u64, znn_amount: BigUint, qsr_amount: BigUint) -> Self {
        Self {
            epoch,
            znn_amount,
            qsr_amount,
        }
    }

    /// Returns the epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Returns the ZNN amount.
    pub fn znn_amount(&self) -> &BigUint {
        &self.znn_amount
    }

    /// Returns the QSR amount.
    pub fn qsr_amount(&self) -> &BigUint {
        &self.qsr_amount
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "epoch": self.epoch,
            "znnAmount": self.znn_amount.to_string(),
            "qsrAmount": self.qsr_amount.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "reward history entry")?;
        Ok(Self::new(
            required_u64(object, "epoch")?,
            required_big_uint(object, "znnAmount")?,
            required_big_uint(object, "qsrAmount")?,
        ))
    }
}

/// A paged list of reward history entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardHistoryList {
    count: u64,
    list: Vec<RewardHistoryEntry>,
}

impl RewardHistoryList {
    /// Creates a reward history list.
    pub fn new(count: u64, list: Vec<RewardHistoryEntry>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Returns the list.
    pub fn list(&self) -> &[RewardHistoryEntry] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(RewardHistoryEntry::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "reward history list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", RewardHistoryEntry::from_json)?,
        ))
    }
}

/// A vote tally for a hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteBreakdown {
    id: Hash,
    yes: u64,
    no: u64,
    total: u64,
}

impl VoteBreakdown {
    /// Creates a vote breakdown.
    pub fn new(id: Hash, yes: u64, no: u64, total: u64) -> Self {
        Self { id, yes, no, total }
    }

    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }

    /// Returns the yes count.
    pub fn yes(&self) -> u64 {
        self.yes
    }

    /// Returns the no count.
    pub fn no(&self) -> u64 {
        self.no
    }

    /// Returns the total.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id.to_string(),
            "yes": self.yes,
            "no": self.no,
            "total": self.total,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "vote breakdown")?;
        Ok(Self::new(
            Hash::parse(required_str(object, "id")?)?,
            required_u64(object, "yes")?,
            required_u64(object, "no")?,
            required_u64(object, "total")?,
        ))
    }
}

/// A pillar's vote on a hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PillarVote {
    id: Hash,
    name: String,
    vote: u64,
}

impl PillarVote {
    /// Creates a pillar vote.
    pub fn new(id: Hash, name: String, vote: u64) -> Self {
        Self { id, name, vote }
    }

    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the vote.
    pub fn vote(&self) -> u64 {
        self.vote
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id.to_string(),
            "name": self.name,
            "vote": self.vote,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "pillar vote")?;
        Ok(Self::new(
            Hash::parse(required_str(object, "id")?)?,
            required_str(object, "name")?.to_string(),
            required_u64(object, "vote")?,
        ))
    }
}

/// Security info for a guarded account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityInfo {
    guardians: Vec<Address>,
    guardians_votes: Vec<Address>,
    administrator_delay: u64,
    soft_delay: u64,
}

impl SecurityInfo {
    /// Creates security info.
    pub fn new(
        guardians: Vec<Address>,
        guardians_votes: Vec<Address>,
        administrator_delay: u64,
        soft_delay: u64,
    ) -> Self {
        Self {
            guardians,
            guardians_votes,
            administrator_delay,
            soft_delay,
        }
    }

    /// Returns the guardians.
    pub fn guardians(&self) -> &[Address] {
        &self.guardians
    }

    /// Returns the guardian votes.
    pub fn guardians_votes(&self) -> &[Address] {
        &self.guardians_votes
    }

    /// Returns the administrator delay.
    pub fn administrator_delay(&self) -> u64 {
        self.administrator_delay
    }

    /// Returns the soft delay.
    pub fn soft_delay(&self) -> u64 {
        self.soft_delay
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "guardians": self.guardians.iter().map(Address::to_string).collect::<Vec<_>>(),
            "guardiansVotes": self.guardians_votes.iter().map(Address::to_string).collect::<Vec<_>>(),
            "administratorDelay": self.administrator_delay,
            "softDelay": self.soft_delay,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "security info")?;
        Ok(Self::new(
            required_array(object, "guardians", value_to_address)?,
            required_array(object, "guardiansVotes", value_to_address)?,
            required_u64(object, "administratorDelay")?,
            required_u64(object, "softDelay")?,
        ))
    }
}

/// A reward deposit for an address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardDeposit {
    address: Address,
    znn_amount: BigUint,
    qsr_amount: BigUint,
}

impl RewardDeposit {
    /// Creates a reward deposit.
    pub fn new(address: Address, znn_amount: BigUint, qsr_amount: BigUint) -> Self {
        Self {
            address,
            znn_amount,
            qsr_amount,
        }
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the ZNN amount.
    pub fn znn_amount(&self) -> &BigUint {
        &self.znn_amount
    }

    /// Returns the QSR amount.
    pub fn qsr_amount(&self) -> &BigUint {
        &self.qsr_amount
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "address": self.address.to_string(),
            "znnAmount": self.znn_amount.to_string(),
            "qsrAmount": self.qsr_amount.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "reward deposit")?;
        Ok(Self::new(
            Address::parse(required_str(object, "address")?)?,
            required_big_uint(object, "znnAmount")?,
            required_big_uint(object, "qsrAmount")?,
        ))
    }
}

/// Info for a time-locked challenge (`PascalCase` JSON keys).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeChallengeInfo {
    method_name: String,
    params_hash: Hash,
    challenge_start_height: u64,
}

impl TimeChallengeInfo {
    /// Creates time challenge info.
    pub fn new(method_name: String, params_hash: Hash, challenge_start_height: u64) -> Self {
        Self {
            method_name,
            params_hash,
            challenge_start_height,
        }
    }

    /// Returns the method name.
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// Returns the params hash.
    pub fn params_hash(&self) -> &Hash {
        &self.params_hash
    }

    /// Returns the challenge start height.
    pub fn challenge_start_height(&self) -> u64 {
        self.challenge_start_height
    }

    /// Serializes to a JSON object with `PascalCase` keys.
    pub fn to_json(&self) -> Value {
        json!({
            "MethodName": self.method_name,
            "ParamsHash": self.params_hash.to_string(),
            "ChallengeStartHeight": self.challenge_start_height,
        })
    }

    /// Deserializes from a JSON object with `PascalCase` keys.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "time challenge info")?;
        Ok(Self::new(
            required_str(object, "MethodName")?.to_string(),
            Hash::parse(required_str(object, "ParamsHash")?)?,
            required_u64(object, "ChallengeStartHeight")?,
        ))
    }
}

fn value_to_address(value: &Value) -> Result<Address, Error> {
    let s = value
        .as_str()
        .ok_or_else(|| Error::InvalidInput("address must be a string".into()))?;
    Address::parse(s)
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
        uncollected_reward: Value,
        reward_history_entry: Value,
        reward_history_list: Value,
        vote_breakdown: Value,
        pillar_vote: Value,
        security_info: Value,
        reward_deposit: Value,
        time_challenge_info: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/common.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid embedded common conformance")
    }

    fn znn() -> BigUint {
        BigUint::from(15_000_000_000u64)
    }

    fn qsr() -> BigUint {
        BigUint::from(3_750_000_000u64)
    }

    fn hash_value() -> &'static str {
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }

    #[test]
    fn uncollected_reward_new_and_accessors() {
        let r = UncollectedReward::new(
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            znn(),
            qsr(),
        );
        assert_eq!(
            r.address().to_string(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(*r.znn_amount(), znn());
        assert_eq!(*r.qsr_amount(), qsr());
    }

    #[test]
    fn uncollected_reward_round_trip() {
        let original = UncollectedReward::new(
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            znn(),
            qsr(),
        );
        let round_trip =
            UncollectedReward::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn uncollected_reward_to_json_matches_conformance() {
        let value = conf().uncollected_reward;
        let reward = UncollectedReward::from_json(&value).expect("conformance parses");
        assert_eq!(reward.to_json(), value);
    }

    #[test]
    fn uncollected_reward_from_json_reads_conformance() {
        let value = conf().uncollected_reward;
        let reward = UncollectedReward::from_json(&value).expect("conformance parses");
        assert_eq!(
            reward.address().to_string(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(*reward.znn_amount(), znn());
        assert_eq!(*reward.qsr_amount(), qsr());
    }

    #[test]
    fn uncollected_reward_rejects_malformed() {
        let mut bad = conf().uncollected_reward;
        bad["znnAmount"] = json!("not-a-number");
        let result = UncollectedReward::from_json(&bad);
        assert!(result.is_err(), "non-decimal znnAmount must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn reward_history_entry_round_trip() {
        let original = RewardHistoryEntry::new(
            100,
            BigUint::from(5_000_000_000u64),
            BigUint::from(1_250_000_000u64),
        );
        let round_trip =
            RewardHistoryEntry::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn reward_history_entry_from_json_reads_conformance() {
        let entry = RewardHistoryEntry::from_json(&conf().reward_history_entry).expect("parses");
        assert_eq!(entry.epoch(), 100);
        assert_eq!(*entry.znn_amount(), BigUint::from(5_000_000_000u64));
    }

    #[test]
    fn reward_history_list_round_trip() {
        let value = conf().reward_history_list;
        let list = RewardHistoryList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
        assert_eq!(list.count(), 2);
        assert_eq!(list.list().len(), 2);
    }

    #[test]
    fn reward_history_list_rejects_non_array_list() {
        let mut bad = conf().reward_history_list;
        bad["list"] = json!("not-an-array");
        let result = RewardHistoryList::from_json(&bad);
        assert!(result.is_err(), "non-array list must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn vote_breakdown_round_trip() {
        let original = VoteBreakdown::new(Hash::parse(hash_value()).unwrap(), 12, 3, 15);
        let round_trip = VoteBreakdown::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn vote_breakdown_from_json_reads_conformance() {
        let v = VoteBreakdown::from_json(&conf().vote_breakdown).expect("parses");
        assert_eq!(v.yes(), 12);
        assert_eq!(v.no(), 3);
        assert_eq!(v.total(), 15);
        assert_eq!(v.id().to_string(), hash_value());
    }

    #[test]
    fn pillar_vote_round_trip() {
        let original =
            PillarVote::new(Hash::parse(hash_value()).unwrap(), "Pillar1".to_string(), 1);
        let round_trip = PillarVote::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn pillar_vote_from_json_reads_conformance() {
        let v = PillarVote::from_json(&conf().pillar_vote).expect("parses");
        assert_eq!(v.name(), "Pillar1");
        assert_eq!(v.vote(), 1);
        assert_eq!(v.id().to_string(), hash_value());
    }

    #[test]
    fn security_info_round_trip() {
        let value = conf().security_info;
        let info = SecurityInfo::from_json(&value).expect("conformance parses");
        assert_eq!(info.to_json(), value);
        assert_eq!(info.guardians().len(), 2);
        assert_eq!(info.guardians_votes().len(), 1);
        assert_eq!(info.administrator_delay(), 1000);
        assert_eq!(info.soft_delay(), 500);
    }

    #[test]
    fn security_info_rejects_non_array_guardians() {
        let mut bad = conf().security_info;
        bad["guardians"] = json!("not-an-array");
        let result = SecurityInfo::from_json(&bad);
        assert!(result.is_err(), "non-array guardians must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn reward_deposit_round_trip() {
        let original = RewardDeposit::new(
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            BigUint::from(7_000_000_000u64),
            BigUint::from(1_750_000_000u64),
        );
        let round_trip = RewardDeposit::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn reward_deposit_from_json_reads_conformance() {
        let d = RewardDeposit::from_json(&conf().reward_deposit).expect("parses");
        assert_eq!(
            d.address().to_string(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(*d.znn_amount(), BigUint::from(7_000_000_000u64));
    }

    #[test]
    fn time_challenge_info_round_trip_preserves_pascalcase_keys() {
        let value = conf().time_challenge_info;
        let info = TimeChallengeInfo::from_json(&value).expect("conformance parses");
        assert_eq!(info.to_json(), value);
        assert_eq!(info.method_name(), "VoteFor");
        assert_eq!(info.challenge_start_height(), 42);
    }

    #[test]
    fn time_challenge_info_rejects_missing_pascalcase_key() {
        let mut bad = conf().time_challenge_info;
        bad.as_object_mut().unwrap().remove("MethodName");
        let result = TimeChallengeInfo::from_json(&bad);
        assert!(result.is_err(), "missing MethodName must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }
}

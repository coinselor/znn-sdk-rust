//! Accelerator contract models.

use crate::error::Error;
use crate::model::embedded::common::VoteBreakdown;
use crate::model::json::*;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use num_bigint::BigUint;
use serde_json::{Map, Value, json};

/// Accelerator project status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum AcceleratorProjectStatus {
    /// Under vote.
    Voting = 0,
    /// Active.
    Active = 1,
    /// Paid.
    Paid = 2,
    /// Closed.
    Closed = 3,
    /// Completed.
    Completed = 4,
}

impl AcceleratorProjectStatus {
    /// Returns the ordinal.
    pub fn as_index(self) -> u64 {
        self as u64
    }

    /// Parses an ordinal, returning `None` for an out-of-range value.
    pub fn from_index(index: u64) -> Option<Self> {
        match index {
            0 => Some(Self::Voting),
            1 => Some(Self::Active),
            2 => Some(Self::Paid),
            3 => Some(Self::Closed),
            4 => Some(Self::Completed),
            _ => None,
        }
    }
}

/// Accelerator project vote choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum AcceleratorProjectVote {
    /// Yes.
    Yes = 0,
    /// No.
    No = 1,
    /// Abstain.
    Abstain = 2,
}

impl AcceleratorProjectVote {
    /// Returns the ordinal.
    pub fn as_index(self) -> u64 {
        self as u64
    }

    /// Parses an ordinal, returning `None` for an out-of-range value.
    pub fn from_index(index: u64) -> Option<Self> {
        match index {
            0 => Some(Self::Yes),
            1 => Some(Self::No),
            2 => Some(Self::Abstain),
            _ => None,
        }
    }
}

/// An accelerator phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase {
    id: Hash,
    project_id: Hash,
    name: String,
    description: String,
    url: String,
    znn_funds_needed: BigUint,
    qsr_funds_needed: BigUint,
    creation_timestamp: u64,
    accepted_timestamp: u64,
    status: AcceleratorProjectStatus,
    vote_breakdown: VoteBreakdown,
}

impl Phase {
    /// Creates a phase.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Hash,
        project_id: Hash,
        name: String,
        description: String,
        url: String,
        znn_funds_needed: BigUint,
        qsr_funds_needed: BigUint,
        creation_timestamp: u64,
        accepted_timestamp: u64,
        status: AcceleratorProjectStatus,
        vote_breakdown: VoteBreakdown,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            description,
            url,
            znn_funds_needed,
            qsr_funds_needed,
            creation_timestamp,
            accepted_timestamp,
            status,
            vote_breakdown,
        }
    }

    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }
    /// Returns the project id.
    pub fn project_id(&self) -> &Hash {
        &self.project_id
    }
    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }
    /// Returns the url.
    pub fn url(&self) -> &str {
        &self.url
    }
    /// Returns the znn funds needed.
    pub fn znn_funds_needed(&self) -> &BigUint {
        &self.znn_funds_needed
    }
    /// Returns the qsr funds needed.
    pub fn qsr_funds_needed(&self) -> &BigUint {
        &self.qsr_funds_needed
    }
    /// Returns the creation timestamp.
    pub fn creation_timestamp(&self) -> u64 {
        self.creation_timestamp
    }
    /// Returns the accepted timestamp.
    pub fn accepted_timestamp(&self) -> u64 {
        self.accepted_timestamp
    }
    /// Returns the status.
    pub fn status(&self) -> AcceleratorProjectStatus {
        self.status
    }
    /// Returns the vote breakdown.
    pub fn vote_breakdown(&self) -> &VoteBreakdown {
        &self.vote_breakdown
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "phase": {
                "id": self.id.to_string(),
                "projectID": self.project_id.to_string(),
                "name": self.name,
                "description": self.description,
                "url": self.url,
                "znnFundsNeeded": self.znn_funds_needed.to_string(),
                "qsrFundsNeeded": self.qsr_funds_needed.to_string(),
                "creationTimestamp": self.creation_timestamp,
                "acceptedTimestamp": self.accepted_timestamp,
                "status": self.status.as_index(),
            },
            "votes": self.vote_breakdown.to_json(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let outer = json_object(value, "phase")?;
        let object = json_object(required_value(outer, "phase")?, "phase body")?;
        Ok(Self::new(
            Hash::parse(required_str(object, "id")?)?,
            Hash::parse(required_str(object, "projectID")?)?,
            required_str(object, "name")?.to_string(),
            required_str(object, "description")?.to_string(),
            required_str(object, "url")?.to_string(),
            required_big_uint(object, "znnFundsNeeded")?,
            required_big_uint(object, "qsrFundsNeeded")?,
            required_u64(object, "creationTimestamp")?,
            required_u64(object, "acceptedTimestamp")?,
            required_status(object, "status")?,
            VoteBreakdown::from_json(required_value(outer, "votes")?)?,
        ))
    }
}

/// An accelerator project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    id: Hash,
    name: String,
    description: String,
    url: String,
    owner: Address,
    znn_funds_needed: BigUint,
    qsr_funds_needed: BigUint,
    creation_timestamp: u64,
    last_update_timestamp: u64,
    status: AcceleratorProjectStatus,
    phase_ids: Vec<Hash>,
    phases: Vec<Phase>,
    vote_breakdown: VoteBreakdown,
}

impl Project {
    /// Creates a project.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Hash,
        name: String,
        description: String,
        url: String,
        owner: Address,
        znn_funds_needed: BigUint,
        qsr_funds_needed: BigUint,
        creation_timestamp: u64,
        last_update_timestamp: u64,
        status: AcceleratorProjectStatus,
        phase_ids: Vec<Hash>,
        phases: Vec<Phase>,
        vote_breakdown: VoteBreakdown,
    ) -> Self {
        Self {
            id,
            name,
            description,
            url,
            owner,
            znn_funds_needed,
            qsr_funds_needed,
            creation_timestamp,
            last_update_timestamp,
            status,
            phase_ids,
            phases,
            vote_breakdown,
        }
    }

    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }
    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }
    /// Returns the url.
    pub fn url(&self) -> &str {
        &self.url
    }
    /// Returns the owner.
    pub fn owner(&self) -> &Address {
        &self.owner
    }
    /// Returns the znn funds needed.
    pub fn znn_funds_needed(&self) -> &BigUint {
        &self.znn_funds_needed
    }
    /// Returns the qsr funds needed.
    pub fn qsr_funds_needed(&self) -> &BigUint {
        &self.qsr_funds_needed
    }
    /// Returns the creation timestamp.
    pub fn creation_timestamp(&self) -> u64 {
        self.creation_timestamp
    }
    /// Returns the last update timestamp.
    pub fn last_update_timestamp(&self) -> u64 {
        self.last_update_timestamp
    }
    /// Returns the status.
    pub fn status(&self) -> AcceleratorProjectStatus {
        self.status
    }
    /// Returns the phase ids.
    pub fn phase_ids(&self) -> &[Hash] {
        &self.phase_ids
    }
    /// Returns the phases.
    pub fn phases(&self) -> &[Phase] {
        &self.phases
    }
    /// Returns the vote breakdown.
    pub fn vote_breakdown(&self) -> &VoteBreakdown {
        &self.vote_breakdown
    }

    /// Sum of znn funds of paid phases.
    pub fn get_paid_znn_funds(&self) -> BigUint {
        self.phases
            .iter()
            .filter(|phase| phase.status() == AcceleratorProjectStatus::Paid)
            .fold(BigUint::from(0u32), |sum, phase| {
                sum + phase.znn_funds_needed()
            })
    }

    /// Pending znn fund of the active last phase.
    pub fn get_pending_znn_funds(&self) -> BigUint {
        match self.phases.last() {
            Some(last) if last.status() == AcceleratorProjectStatus::Active => {
                last.znn_funds_needed().clone()
            }
            _ => BigUint::from(0u32),
        }
    }

    /// Remaining znn fund.
    pub fn get_remaining_znn_funds(&self) -> BigUint {
        if self.phases.is_empty() {
            self.znn_funds_needed.clone()
        } else {
            saturating_sub_biguint(&self.znn_funds_needed, self.get_paid_znn_funds())
        }
    }

    /// Total znn fund.
    pub fn get_total_znn_funds(&self) -> BigUint {
        self.znn_funds_needed.clone()
    }

    /// Sum of qsr funds of paid phases.
    pub fn get_paid_qsr_funds(&self) -> BigUint {
        self.phases
            .iter()
            .filter(|phase| phase.status() == AcceleratorProjectStatus::Paid)
            .fold(BigUint::from(0u32), |sum, phase| {
                sum + phase.qsr_funds_needed()
            })
    }

    /// Pending qsr fund of the active last phase.
    pub fn get_pending_qsr_funds(&self) -> BigUint {
        match self.phases.last() {
            Some(last) if last.status() == AcceleratorProjectStatus::Active => {
                last.qsr_funds_needed().clone()
            }
            _ => BigUint::from(0u32),
        }
    }

    /// Remaining qsr fund.
    pub fn get_remaining_qsr_funds(&self) -> BigUint {
        if self.phases.is_empty() {
            self.qsr_funds_needed.clone()
        } else {
            saturating_sub_biguint(&self.qsr_funds_needed, self.get_paid_qsr_funds())
        }
    }

    /// Total qsr fund.
    pub fn get_total_qsr_funds(&self) -> BigUint {
        self.qsr_funds_needed.clone()
    }

    /// Returns the last phase, or `None` when there are none.
    pub fn get_last_phase(&self) -> Option<&Phase> {
        self.phases.last()
    }

    /// Returns the phase whose id matches, or `None`.
    pub fn find_phase_by_id(&self, id: &Hash) -> Option<&Phase> {
        self.phases.iter().find(|phase| phase.id() == id)
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id.to_string(),
            "owner": self.owner.to_string(),
            "name": self.name,
            "description": self.description,
            "url": self.url,
            "znnFundsNeeded": self.znn_funds_needed.to_string(),
            "qsrFundsNeeded": self.qsr_funds_needed.to_string(),
            "creationTimestamp": self.creation_timestamp,
            "lastUpdateTimestamp": self.last_update_timestamp,
            "status": self.status.as_index(),
            "phaseIds": self.phase_ids.iter().map(Hash::to_string).collect::<Vec<_>>(),
            "phases": self.phases.iter().map(Phase::to_json).collect::<Vec<_>>(),
            "votes": self.vote_breakdown.to_json(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "project")?;
        Ok(Self::new(
            Hash::parse(required_str(object, "id")?)?,
            required_str(object, "name")?.to_string(),
            required_str(object, "description")?.to_string(),
            required_str(object, "url")?.to_string(),
            Address::parse(required_str(object, "owner")?)?,
            required_big_uint(object, "znnFundsNeeded")?,
            required_big_uint(object, "qsrFundsNeeded")?,
            required_u64(object, "creationTimestamp")?,
            required_u64(object, "lastUpdateTimestamp")?,
            required_status(object, "status")?,
            required_array(object, "phaseIds", value_to_hash)?,
            required_array(object, "phases", Phase::from_json)?,
            VoteBreakdown::from_json(required_value(object, "votes")?)?,
        ))
    }
}

/// A paged list of accelerator projects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectList {
    count: u64,
    list: Vec<Project>,
}

impl ProjectList {
    /// Creates a project list.
    pub fn new(count: u64, list: Vec<Project>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Returns the list.
    pub fn list(&self) -> &[Project] {
        &self.list
    }

    /// Returns the project whose id matches, or `None`.
    pub fn find_id(&self, id: &Hash) -> Option<&Project> {
        self.list.iter().find(|project| project.id() == id)
    }

    /// Returns the project containing the phase id, or `None`.
    pub fn find_project_by_phase_id(&self, phase_id: &Hash) -> Option<&Project> {
        self.list
            .iter()
            .find(|project| project.phase_ids().iter().any(|pid| pid == phase_id))
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(Project::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "project list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", Project::from_json)?,
        ))
    }
}

fn required_status(
    object: &Map<String, Value>,
    field: &str,
) -> Result<AcceleratorProjectStatus, Error> {
    AcceleratorProjectStatus::from_index(required_u64(object, field)?)
        .ok_or_else(|| Error::InvalidInput(format!("{field} is out of range")))
}

fn saturating_sub_biguint(left: &BigUint, right: BigUint) -> BigUint {
    if left <= &right {
        BigUint::from(0u32)
    } else {
        left.clone() - right
    }
}

fn value_to_hash(value: &Value) -> Result<Hash, Error> {
    let s = value
        .as_str()
        .ok_or_else(|| Error::InvalidInput("hash must be a string".into()))?;
    Hash::parse(s)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn empty_breakdown() -> VoteBreakdown {
        VoteBreakdown::new(Hash::empty(), 0, 0, 0)
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Conformance {
        #[allow(dead_code)]
        description: String,
        phase: Value,
        project: Value,
        project_list: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/accelerator.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid accelerator conformance")
    }

    fn hash(suffix: u8) -> Hash {
        let mut bytes = [0u8; 32];
        bytes[31] = suffix;
        Hash::from_bytes(&bytes).unwrap()
    }

    fn phase(id_suffix: u8, znn: u64, qsr: u64, status: AcceleratorProjectStatus) -> Phase {
        Phase::new(
            hash(id_suffix),
            hash(0),
            format!("Phase {id_suffix}"),
            String::new(),
            String::new(),
            BigUint::from(znn),
            BigUint::from(qsr),
            0,
            0,
            status,
            empty_breakdown(),
        )
    }

    fn fund_project() -> Project {
        // Order: Paid, Closed, Active (last is Active so pending applies).
        let phases = vec![
            phase(1, 100, 40, AcceleratorProjectStatus::Paid),
            phase(2, 10, 5, AcceleratorProjectStatus::Closed),
            phase(3, 50, 25, AcceleratorProjectStatus::Active),
        ];
        let phase_ids = vec![hash(1), hash(2), hash(3)];
        Project::new(
            hash(0),
            "Project A".to_string(),
            String::new(),
            String::new(),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            BigUint::from(300u64),
            BigUint::from(150u64),
            0,
            0,
            AcceleratorProjectStatus::Active,
            phase_ids,
            phases,
            empty_breakdown(),
        )
    }

    #[test]
    fn status_from_index_round_trips_ordinals_and_rejects_out_of_range() {
        assert_eq!(
            AcceleratorProjectStatus::from_index(0),
            Some(AcceleratorProjectStatus::Voting)
        );
        assert_eq!(
            AcceleratorProjectStatus::from_index(4),
            Some(AcceleratorProjectStatus::Completed)
        );
        assert_eq!(AcceleratorProjectStatus::from_index(5), None);
    }

    #[test]
    fn vote_from_index_maps_each_ordinal() {
        assert_eq!(
            AcceleratorProjectVote::from_index(0),
            Some(AcceleratorProjectVote::Yes)
        );
        assert_eq!(
            AcceleratorProjectVote::from_index(2),
            Some(AcceleratorProjectVote::Abstain)
        );
    }

    #[test]
    fn phase_round_trip() {
        let original = phase(1, 100, 50, AcceleratorProjectStatus::Paid);
        let round_trip = Phase::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn phase_reads_nested_object() {
        let phase = Phase::from_json(&conf().phase).expect("conformance parses");
        assert_eq!(phase.name(), "Phase 1");
        assert_eq!(*phase.znn_funds_needed(), BigUint::from(100_000_000u64));
        assert_eq!(phase.status(), AcceleratorProjectStatus::Paid);
    }

    #[test]
    fn project_round_trip() {
        let original = fund_project();
        let round_trip = Project::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn paid_znn_sums_only_paid_phases() {
        let project = fund_project();
        assert_eq!(project.get_paid_znn_funds(), BigUint::from(100u64));
    }

    #[test]
    fn pending_znn_returns_active_last_phase_fund() {
        let project = fund_project();
        assert_eq!(project.get_pending_znn_funds(), BigUint::from(50u64));
    }

    #[test]
    fn pending_znn_is_zero_when_last_phase_not_active() {
        // Positive control: an active last phase reports its fund.
        assert_eq!(fund_project().get_pending_znn_funds(), BigUint::from(50u64));

        // A non-empty project whose last phase is not active reports zero pending.
        let project = Project::new(
            hash(0),
            "x".to_string(),
            String::new(),
            String::new(),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            BigUint::from(300u64),
            BigUint::from(150u64),
            0,
            0,
            AcceleratorProjectStatus::Active,
            vec![hash(1), hash(2)],
            vec![
                phase(1, 100, 40, AcceleratorProjectStatus::Paid),
                phase(2, 10, 5, AcceleratorProjectStatus::Closed),
            ],
            empty_breakdown(),
        );
        assert!(!project.phases().is_empty());
        assert_eq!(project.get_pending_znn_funds(), BigUint::from(0u32));
    }

    #[test]
    fn remaining_znn_subtracts_paid_from_needed() {
        let project = fund_project();
        assert_eq!(project.get_remaining_znn_funds(), BigUint::from(200u64));
    }

    #[test]
    fn remaining_funds_saturate_to_zero_when_paid_exceeds_needed() {
        let project = Project::new(
            hash(0),
            "x".to_string(),
            String::new(),
            String::new(),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            BigUint::from(100u64),
            BigUint::from(50u64),
            0,
            0,
            AcceleratorProjectStatus::Active,
            vec![hash(1)],
            vec![phase(1, 150, 75, AcceleratorProjectStatus::Paid)],
            empty_breakdown(),
        );
        assert_eq!(project.get_remaining_znn_funds(), BigUint::from(0u32));
        assert_eq!(project.get_remaining_qsr_funds(), BigUint::from(0u32));
    }

    #[test]
    fn total_znn_returns_needed() {
        let project = fund_project();
        assert_eq!(project.get_total_znn_funds(), BigUint::from(300u64));
    }

    #[test]
    fn qsr_fund_helpers_mirror_znn() {
        let project = fund_project();
        assert_eq!(project.get_paid_qsr_funds(), BigUint::from(40u64));
        assert_eq!(project.get_pending_qsr_funds(), BigUint::from(25u64));
        assert_eq!(project.get_remaining_qsr_funds(), BigUint::from(110u64));
        assert_eq!(project.get_total_qsr_funds(), BigUint::from(150u64));
    }

    #[test]
    fn remaining_and_pending_when_no_phases() {
        let project = Project::new(
            hash(0),
            "x".to_string(),
            String::new(),
            String::new(),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            BigUint::from(300u64),
            BigUint::from(150u64),
            0,
            0,
            AcceleratorProjectStatus::Active,
            Vec::new(),
            Vec::new(),
            empty_breakdown(),
        );
        assert_eq!(project.get_remaining_znn_funds(), BigUint::from(300u64));
        assert_eq!(project.get_pending_znn_funds(), BigUint::from(0u32));
    }

    #[test]
    fn get_last_phase_returns_final_phase() {
        let project = fund_project();
        assert_eq!(
            project.get_last_phase().map(Phase::status),
            Some(AcceleratorProjectStatus::Active)
        );
    }

    #[test]
    fn find_phase_by_id_reflects_id() {
        let project = fund_project();
        assert_eq!(
            project.find_phase_by_id(&hash(1)).map(Phase::status),
            Some(AcceleratorProjectStatus::Paid)
        );
        assert!(project.find_phase_by_id(&hash(99)).is_none());
    }

    #[test]
    fn find_phase_by_id_uses_phase_ids_on_phases_not_project_index_list() {
        let project = Project::new(
            hash(0),
            "x".to_string(),
            String::new(),
            String::new(),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            BigUint::from(300u64),
            BigUint::from(150u64),
            0,
            0,
            AcceleratorProjectStatus::Active,
            vec![hash(99)],
            vec![phase(1, 100, 40, AcceleratorProjectStatus::Paid)],
            empty_breakdown(),
        );
        assert_eq!(
            project.find_phase_by_id(&hash(1)).map(Phase::status),
            Some(AcceleratorProjectStatus::Paid)
        );
        assert!(project.find_phase_by_id(&hash(99)).is_none());
    }

    #[test]
    fn project_list_find_id_reflects_id() {
        let list = ProjectList::new(1, vec![fund_project()]);
        assert!(list.find_id(&hash(0)).is_some());
        assert!(list.find_id(&hash(99)).is_none());
    }

    #[test]
    fn project_list_find_by_phase_id_reflects_id() {
        let list = ProjectList::new(1, vec![fund_project()]);
        assert!(list.find_project_by_phase_id(&hash(2)).is_some());
        assert!(list.find_project_by_phase_id(&hash(99)).is_none());
    }

    #[test]
    fn project_list_round_trip() {
        let value = conf().project_list;
        let list = ProjectList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
    }
}

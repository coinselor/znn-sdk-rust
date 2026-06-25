//! Sentinel contract models.

use crate::error::Error;
use crate::model::json::*;
use crate::primitives::address::Address;
use serde_json::{Value, json};

/// Sentinel registration info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelInfo {
    owner: Address,
    registration_timestamp: u64,
    is_revocable: bool,
    revoke_cooldown: u64,
    active: bool,
}

impl SentinelInfo {
    /// Creates sentinel info.
    pub fn new(
        owner: Address,
        registration_timestamp: u64,
        is_revocable: bool,
        revoke_cooldown: u64,
        active: bool,
    ) -> Self {
        Self {
            owner,
            registration_timestamp,
            is_revocable,
            revoke_cooldown,
            active,
        }
    }

    /// Returns the owner.
    pub fn owner(&self) -> &Address {
        &self.owner
    }
    /// Returns the registration timestamp.
    pub fn registration_timestamp(&self) -> u64 {
        self.registration_timestamp
    }
    /// Returns whether revocable.
    pub fn is_revocable(&self) -> bool {
        self.is_revocable
    }
    /// Returns the revoke cooldown.
    pub fn revoke_cooldown(&self) -> u64 {
        self.revoke_cooldown
    }
    /// Returns whether active.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "owner": self.owner.to_string(),
            "registrationTimestamp": self.registration_timestamp,
            "isRevocable": self.is_revocable,
            "revokeCooldown": self.revoke_cooldown,
            "active": self.active,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "sentinel info")?;
        Ok(Self::new(
            Address::parse(required_str(object, "owner")?)?,
            required_u64(object, "registrationTimestamp")?,
            required_bool(object, "isRevocable")?,
            required_u64(object, "revokeCooldown")?,
            required_bool(object, "active")?,
        ))
    }
}

/// A paged list of sentinel info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelInfoList {
    count: u64,
    list: Vec<SentinelInfo>,
}

impl SentinelInfoList {
    /// Creates a sentinel info list.
    pub fn new(count: u64, list: Vec<SentinelInfo>) -> Self {
        Self { count, list }
    }

    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Returns the list.
    pub fn list(&self) -> &[SentinelInfo] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(SentinelInfo::to_json).collect::<Vec<_>>(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "sentinel info list")?;
        Ok(Self::new(
            required_u64(object, "count")?,
            required_array(object, "list", SentinelInfo::from_json)?,
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
        sentinel_info: Value,
        sentinel_info_list: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/sentinel.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid sentinel conformance")
    }

    fn sample() -> SentinelInfo {
        SentinelInfo::new(
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            1_700_000_000,
            false,
            0,
            true,
        )
    }

    #[test]
    fn sentinel_info_round_trip() {
        let original = sample();
        let round_trip = SentinelInfo::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn sentinel_info_from_json_reads_conformance() {
        let info = SentinelInfo::from_json(&conf().sentinel_info).expect("parses");
        assert_eq!(
            info.owner().to_string(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(info.registration_timestamp(), 1_700_000_000);
        assert!(info.active());
        assert!(!info.is_revocable());
    }

    #[test]
    fn sentinel_info_rejects_malformed() {
        let mut bad = conf().sentinel_info;
        bad.as_object_mut().unwrap().remove("owner");
        let result = SentinelInfo::from_json(&bad);
        assert!(result.is_err(), "missing owner must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn sentinel_info_list_round_trip() {
        let value = conf().sentinel_info_list;
        let list = SentinelInfoList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
        assert_eq!(list.count(), 1);
        assert_eq!(list.list().len(), 1);
    }
}

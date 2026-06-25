//! Node stats and sync models.
//!
//! Read-only types returned by `stats.*` JSON-RPC methods: a peer entry, the
//! node's network info, its process info, its OS info, and the chain sync
//! state/info. `SyncState` encodes by ordinal; every type round-trips through
//! JSON.

use crate::error::Error;
use crate::model::json::*;
use serde_json::{Value, json};

/// A network peer observed by the node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    public_key: String,
    ip: String,
}

impl Peer {
    /// Creates a peer from its public key and IP.
    pub fn new(public_key: impl Into<String>, ip: impl Into<String>) -> Self {
        Self {
            public_key: public_key.into(),
            ip: ip.into(),
        }
    }

    /// Returns the peer's public key.
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// Returns the peer's IP address.
    pub fn ip(&self) -> &str {
        &self.ip
    }

    /// Serializes the peer to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({ "publicKey": self.public_key, "ip": self.ip })
    }

    /// Deserializes a peer from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "peer")?;
        Ok(Self::new(
            required_str(object, "publicKey")?.to_string(),
            required_str(object, "ip")?.to_string(),
        ))
    }
}

/// The node's view of the network: peer count, the local peer, and the peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInfo {
    num_peers: u64,
    self_peer: Peer,
    peers: Vec<Peer>,
}

impl NetworkInfo {
    /// Creates network info from its parts.
    pub fn new(num_peers: u64, self_peer: Peer, peers: Vec<Peer>) -> Self {
        Self {
            num_peers,
            self_peer,
            peers,
        }
    }

    /// Returns the peer count.
    pub fn num_peers(&self) -> u64 {
        self.num_peers
    }

    /// Returns the local peer.
    pub fn self_peer(&self) -> &Peer {
        &self.self_peer
    }

    /// Returns the peer list.
    pub fn peers(&self) -> &[Peer] {
        &self.peers
    }

    /// Serializes the network info to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "numPeers": self.num_peers,
            "self": self.self_peer.to_json(),
            "peers": self.peers.iter().map(Peer::to_json).collect::<Vec<_>>()
        })
    }

    /// Deserializes network info from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "network info")?;
        let num_peers = required_u64(object, "numPeers")?;
        let self_peer = Peer::from_json(required_value(object, "self")?)?;
        let peers = required_array_ref(object, "peers")?
            .iter()
            .map(Peer::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(num_peers, self_peer, peers))
    }
}

/// Build information for the running node process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    commit: String,
    version: String,
}

impl ProcessInfo {
    /// Creates process info from its commit and version.
    pub fn new(commit: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            commit: commit.into(),
            version: version.into(),
        }
    }

    /// Returns the commit hash.
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// Returns the version string.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Serializes the process info to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({ "commit": self.commit, "version": self.version })
    }

    /// Deserializes process info from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "process info")?;
        Ok(Self::new(
            required_str(object, "commit")?.to_string(),
            required_str(object, "version")?.to_string(),
        ))
    }
}

/// Operating-system metrics for the node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsInfo {
    os: String,
    platform: String,
    platform_family: String,
    platform_version: String,
    kernel_version: String,
    memory_total: u64,
    memory_free: u64,
    num_cpu: u32,
    num_goroutine: u32,
}

impl OsInfo {
    /// Creates OS info from all fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        os: impl Into<String>,
        platform: impl Into<String>,
        platform_family: impl Into<String>,
        platform_version: impl Into<String>,
        kernel_version: impl Into<String>,
        memory_total: u64,
        memory_free: u64,
        num_cpu: u32,
        num_goroutine: u32,
    ) -> Self {
        Self {
            os: os.into(),
            platform: platform.into(),
            platform_family: platform_family.into(),
            platform_version: platform_version.into(),
            kernel_version: kernel_version.into(),
            memory_total,
            memory_free,
            num_cpu,
            num_goroutine,
        }
    }

    /// Returns the OS name.
    pub fn os(&self) -> &str {
        &self.os
    }

    /// Returns the platform name.
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// Returns the platform family.
    pub fn platform_family(&self) -> &str {
        &self.platform_family
    }

    /// Returns the platform version.
    pub fn platform_version(&self) -> &str {
        &self.platform_version
    }

    /// Returns the kernel version.
    pub fn kernel_version(&self) -> &str {
        &self.kernel_version
    }

    /// Returns the total memory in bytes.
    pub fn memory_total(&self) -> u64 {
        self.memory_total
    }

    /// Returns the free memory in bytes.
    pub fn memory_free(&self) -> u64 {
        self.memory_free
    }

    /// Returns the CPU count.
    pub fn num_cpu(&self) -> u32 {
        self.num_cpu
    }

    /// Returns the goroutine count.
    pub fn num_goroutine(&self) -> u32 {
        self.num_goroutine
    }

    /// Serializes the OS info to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "os": self.os,
            "platform": self.platform,
            "platformFamily": self.platform_family,
            "platformVersion": self.platform_version,
            "kernelVersion": self.kernel_version,
            "memoryTotal": self.memory_total,
            "memoryFree": self.memory_free,
            "numCPU": self.num_cpu,
            "numGoroutine": self.num_goroutine
        })
    }

    /// Deserializes OS info from a JSON object.
    //
    // `platformVersion` is intentionally derived from the `platform` key; the
    // wire `platformVersion` value is ignored.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "os info")?;
        let os = required_str(object, "os")?.to_string();
        let platform = required_str(object, "platform")?.to_string();
        let platform_family = required_str(object, "platformFamily")?.to_string();
        let kernel_version = required_str(object, "kernelVersion")?.to_string();
        let memory_total = required_u64(object, "memoryTotal")?;
        let memory_free = required_u64(object, "memoryFree")?;
        let num_cpu = required_u32(object, "numCPU")?;
        let num_goroutine = required_u32(object, "numGoroutine")?;
        Ok(Self::new(
            os,
            platform.clone(),
            platform_family,
            platform,
            kernel_version,
            memory_total,
            memory_free,
            num_cpu,
            num_goroutine,
        ))
    }
}

/// Chain synchronization state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SyncState {
    /// The state is unknown.
    Unknown = 0,
    /// The node is syncing.
    Syncing = 1,
    /// The node has finished syncing.
    SyncDone = 2,
    /// The node lacks enough peers to sync.
    NotEnoughPeers = 3,
}

impl SyncState {
    /// Returns the ordinal of this state.
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Parses a state ordinal, returning `None` for an out-of-range value.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Syncing),
            2 => Some(Self::SyncDone),
            3 => Some(Self::NotEnoughPeers),
            _ => None,
        }
    }
}

/// The node's chain sync progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncInfo {
    state: SyncState,
    current_height: u64,
    target_height: u64,
}

impl SyncInfo {
    /// Creates sync info from its state and heights.
    pub fn new(state: SyncState, current_height: u64, target_height: u64) -> Self {
        Self {
            state,
            current_height,
            target_height,
        }
    }

    /// Returns the sync state.
    pub fn state(&self) -> SyncState {
        self.state
    }

    /// Returns the current height.
    pub fn current_height(&self) -> u64 {
        self.current_height
    }

    /// Returns the target height.
    pub fn target_height(&self) -> u64 {
        self.target_height
    }

    /// Serializes the sync info to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "state": self.state.as_u32(),
            "currentHeight": self.current_height,
            "targetHeight": self.target_height
        })
    }

    /// Deserializes sync info from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "sync info")?;
        let state = match object.get("state") {
            Some(value) => SyncState::from_u32(required_u32_of(value, "state")?)
                .ok_or_else(|| Error::InvalidInput("state is out of range".into()))?,
            None => SyncState::Unknown,
        };
        let current_height = required_u64(object, "currentHeight")?;
        let target_height = required_u64(object, "targetHeight")?;
        Ok(Self::new(state, current_height, target_height))
    }
}

fn required_u32_of(value: &Value, field: &str) -> Result<u32, Error> {
    let raw = value
        .as_u64()
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be an unsigned integer")))?;
    u32::try_from(raw).map_err(|_| Error::InvalidInput(format!("{field} is out of range")))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Deserialize)]
    struct Vectors {
        peer: Value,
        network_info: Value,
        process_info: Value,
        os_info: Value,
        sync_info: Value,
    }

    const SYNC: &str = include_str!("../../../tests/conformance/stats/sync.json");

    fn vectors() -> Vectors {
        serde_json::from_str(SYNC).expect("valid stats conformance")
    }

    #[test]
    fn sync_state_round_trips_each_index() {
        for index in 0..=3_u32 {
            let variant = SyncState::from_u32(index).expect("in-range index maps to a variant");
            assert_eq!(variant.as_u32(), index, "index {index} must round-trip");
        }
    }

    #[test]
    fn sync_state_rejects_out_of_range_index() {
        assert!(SyncState::from_u32(4).is_none());
        assert!(SyncState::from_u32(u32::MAX).is_none());
    }

    #[test]
    fn peer_round_trips_the_conformance_object() {
        let v = vectors();
        let peer = Peer::from_json(&v.peer).expect("peer parses");
        assert_eq!(
            peer.to_json(),
            v.peer,
            "peer must round-trip the conformance"
        );
        assert_eq!(
            peer.public_key(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
        assert_eq!(peer.ip(), "192.0.2.10");
    }

    #[test]
    fn peer_rejects_a_non_object() {
        let arr = serde_json::json!([1, 2, 3]);
        let err = Peer::from_json(&arr).expect_err("array is not a peer");
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn network_info_round_trips_the_conformance_object() {
        let v = vectors();
        let info = NetworkInfo::from_json(&v.network_info).expect("network info parses");
        assert_eq!(
            info.to_json(),
            v.network_info,
            "network info must round-trip"
        );
        assert_eq!(info.num_peers(), 3);
        assert_eq!(info.peers().len(), 2);
        assert_eq!(
            info.self_peer().public_key(),
            "z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz"
        );
    }

    #[test]
    fn network_info_rejects_a_non_array_peers_field() {
        let bad = serde_json::json!({ "numPeers": 1, "self": { "publicKey": "k", "ip": "i" }, "peers": "nope" });
        let err = NetworkInfo::from_json(&bad).expect_err("peers must be an array");
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn process_info_round_trips_the_conformance_object() {
        let v = vectors();
        let info = ProcessInfo::from_json(&v.process_info).expect("process info parses");
        assert_eq!(
            info.to_json(),
            v.process_info,
            "process info must round-trip"
        );
        assert_eq!(info.commit(), "0123abcd");
        assert_eq!(info.version(), "0.0.1");
    }

    #[test]
    fn os_info_platform_version_mirrors_the_platform_key() {
        let v = vectors();
        let os = OsInfo::from_json(&v.os_info).expect("os info parses");
        assert_eq!(os.platform(), "ubuntu");
        assert_eq!(
            os.platform_version(),
            "ubuntu",
            "platformVersion must mirror the platform key"
        );
        assert_eq!(os.platform_family(), "debian");
        assert_eq!(os.memory_total(), 17_179_869_184);
        assert_eq!(os.num_cpu(), 8);
        assert_eq!(os.num_goroutine(), 1024);
    }

    #[test]
    fn os_info_rejects_a_missing_num_cpu() {
        let bad = serde_json::json!({
            "os": "linux", "platform": "ubuntu", "platformFamily": "debian",
            "kernelVersion": "5.15.0", "memoryTotal": 1, "memoryFree": 1, "numGoroutine": 1
        });
        let err = OsInfo::from_json(&bad).expect_err("numCPU is required");
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn sync_info_round_trips_the_conformance_object() {
        let v = vectors();
        let info = SyncInfo::from_json(&v.sync_info).expect("sync info parses");
        assert_eq!(info.to_json(), v.sync_info, "sync info must round-trip");
        assert_eq!(info.state(), SyncState::Syncing);
        assert_eq!(info.current_height(), 123_456);
        assert_eq!(info.target_height(), 123_457);
    }

    #[test]
    fn sync_info_defaults_state_to_unknown_when_absent() {
        let missing_state = serde_json::json!({ "currentHeight": 10, "targetHeight": 20 });
        let result = SyncInfo::from_json(&missing_state);
        assert!(
            result.is_ok(),
            "absent state must default to Unknown, got {result:?}"
        );
        let info = result.expect("checked ok above");
        assert_eq!(info.state(), SyncState::Unknown);
        assert_eq!(info.current_height(), 10);
        assert_eq!(info.target_height(), 20);
    }

    #[test]
    fn sync_info_rejects_an_out_of_range_state() {
        let bad = serde_json::json!({ "state": 9, "currentHeight": 1, "targetHeight": 2 });
        let err = SyncInfo::from_json(&bad).expect_err("state is out of range");
        assert!(matches!(err, Error::InvalidInput(_)));
    }
}

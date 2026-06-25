//! Stats API (`stats.*` JSON-RPC methods).
//!
//! [`StatsApi`] wraps a shared [`Client`] and dispatches the read-only `stats.*`
//! namespace: [`os_info`], [`process_info`], [`network_info`], and
//! [`sync_info`].
//!
//! [`os_info`]: StatsApi::os_info
//! [`process_info`]: StatsApi::process_info
//! [`network_info`]: StatsApi::network_info
//! [`sync_info`]: StatsApi::sync_info

use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::error::Error;
use crate::model::stats::{NetworkInfo, OsInfo, ProcessInfo, SyncInfo};
use serde_json::Value;
use std::sync::Arc;

/// The `stats.*` JSON-RPC namespace.
pub struct StatsApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> StatsApi<C> {
    /// Creates a stats API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns the node's operating-system metrics.
    pub async fn os_info(&self) -> Result<OsInfo, Error> {
        let response = self.dispatch("stats.osInfo", &[]).await?;
        OsInfo::from_json(&response)
    }

    /// Returns the node process build information.
    pub async fn process_info(&self) -> Result<ProcessInfo, Error> {
        let response = self.dispatch("stats.processInfo", &[]).await?;
        ProcessInfo::from_json(&response)
    }

    /// Returns the node's network view.
    pub async fn network_info(&self) -> Result<NetworkInfo, Error> {
        let response = self.dispatch("stats.networkInfo", &[]).await?;
        NetworkInfo::from_json(&response)
    }

    /// Returns the node's chain sync progress.
    pub async fn sync_info(&self) -> Result<SyncInfo, Error> {
        let response = self.dispatch("stats.syncInfo", &[]).await?;
        SyncInfo::from_json(&response)
    }

    async fn dispatch(&self, method: &str, params: &[Value]) -> Result<Value, Error> {
        self.client
            .send_request(method, params)
            .await
            .map_err(Error::from)
    }
}

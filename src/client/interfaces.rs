//! The async JSON-RPC client trait.

use crate::client::exceptions::ClientError;
use serde_json::Value;

/// Sends JSON-RPC requests to a Zenon full node.
pub trait Client: Send + Sync {
    /// Sends `method` with positional `params`, returning the JSON result or a
    /// client error.
    fn send_request<'a>(
        &'a self,
        method: &'a str,
        params: &'a [Value],
    ) -> impl std::future::Future<Output = Result<Value, ClientError>> + Send + 'a;
}

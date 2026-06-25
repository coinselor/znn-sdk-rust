//! Client error types.

use thiserror::Error;

/// Errors returned by the JSON-RPC client.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientError {
    /// There is no live transport to the Zenon full node.
    #[error("No connection to the Zenon full node")]
    NoConnection,
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn no_connection_renders_expected_message() {
        assert_eq!(
            ClientError::NoConnection.to_string(),
            "No connection to the Zenon full node"
        );
    }
}

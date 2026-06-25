//! Shared error types for the Zenon Rust SDK.

use crate::client::exceptions::ClientError;
use crate::publish::PublishError;
use thiserror::Error;

/// Errors returned by the Zenon Rust SDK.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// A generic error for modules not yet specialized.
    #[error("{0}")]
    Generic(String),

    /// Invalid user input such as a malformed address or hex string.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// JSON serialization or deserialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// A keystore decryption failed authentication (wrong password).
    #[error("incorrect password")]
    IncorrectPassword,

    /// A JSON-RPC client error surfaced through an API method.
    #[error("{0}")]
    Client(ClientError),

    /// A typed publish error decoded from a non-null `ledger.publishRawTransaction`
    /// response. Carries the structured [`PublishError`] so callers can match on
    /// the rejection kind without re-parsing the response.
    #[error("{0}")]
    Publish(PublishError),
}

/// Converts a [`ClientError`] into the crate [`enum@Error`].
impl From<ClientError> for Error {
    fn from(value: ClientError) -> Self {
        Self::Client(value)
    }
}

impl Error {
    /// Creates a generic error with the given message.
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_displays_message() {
        assert_eq!(Error::generic("scaffold ok").to_string(), "scaffold ok");
    }

    #[test]
    fn invalid_input_displays_message() {
        let err = Error::InvalidInput("bad hex".into());
        assert_eq!(err.to_string(), "invalid input: bad hex");
    }

    #[test]
    fn serialization_displays_message() {
        let err = Error::Serialization("bad json".into());
        assert_eq!(err.to_string(), "serialization error: bad json");
    }

    #[test]
    fn incorrect_password_displays_message() {
        assert_eq!(Error::IncorrectPassword.to_string(), "incorrect password");
    }
}

//! Typed publish result and error.
//!
//! [`PublishResult`] and [`PublishError`] model the outcome of publishing a
//! prepared account block. They live in an always-compiled module (independent
//! of the `client-ws` feature) so the crate [`Error`](crate::error::Error) can
//! carry a [`Error::Publish`](crate::error::Error::Publish) variant in
//! reduced-core builds, where the websocket
//! transport is excluded. The `ledger.*` API re-exports both types for
//! ergonomic access alongside the publish methods.

use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::hash::Hash;
use serde_json::Value;

/// Result of publishing a prepared account block.
///
/// The node accepts a valid publish with a `null` response; the SDK surfaces the
/// computed hash and the prepared template as the receipt. The result represents
/// publish-acceptance (mempool admission), not confirmation. Confirmation is a
/// separate, later concern. The `hash` is SDK-computed during block preparation
/// (see [`crate::utils::block::get_transaction_hash`]), not returned by the node,
/// so it identifies the submitted block rather than confirming its application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishResult {
    hash: Hash,
    template: AccountBlockTemplate,
}

impl PublishResult {
    /// Creates a publish result from its accepted hash and prepared template.
    pub fn new(hash: Hash, template: AccountBlockTemplate) -> Self {
        Self { hash, template }
    }

    /// Returns the SDK-computed hash of the published block.
    ///
    /// This hash is derived from the prepared template, not returned by the node.
    /// Callers can use it to track the block (e.g. via `getAccountBlockByHash`),
    /// but it does not by itself prove the block was confirmed.
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    /// Returns the fully prepared, signed template that was published.
    pub fn template(&self) -> &AccountBlockTemplate {
        &self.template
    }
}

/// Typed error decoded from a non-null `ledger.publishRawTransaction` response.
///
/// The node's non-null rejection body is not field-documented, so the decoder
/// is defensive: a recognizable rejection string becomes
/// [`PublishError::Rejected`], and any other non-null shape becomes
/// [`PublishError::Unexpected`] (rendered to a string so the type stays `Eq`).
/// The enum is `#[non_exhaustive]` so richer variants can be added later without
/// a breaking change.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishError {
    /// The node rejected the block with a reason message.
    Rejected {
        /// The rejection reason.
        message: String,
    },
    /// An unexpected non-null, non-rejection response shape, rendered as JSON.
    Unexpected(String),
}

impl PublishError {
    /// Decodes a non-null node response into a typed error.
    ///
    /// A rejection string becomes [`Self::Rejected`]; any other non-null shape
    /// becomes [`Self::Unexpected`] rendered to a string so the type stays `Eq`.
    pub fn from_response(value: &Value) -> Self {
        match value {
            Value::String(message) => Self::Rejected {
                message: message.clone(),
            },
            _ => Self::Unexpected(value.to_string()),
        }
    }
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected { message } => {
                write!(f, "publish rejected: {message}")
            }
            Self::Unexpected(rendered) => {
                write!(f, "publish rejected: unexpected response {rendered}")
            }
        }
    }
}

impl std::error::Error for PublishError {}

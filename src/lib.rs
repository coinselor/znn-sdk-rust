//! Zenon SDK for Rust.
//!
//! Recommended imports are available directly from `znn_sdk_rust`: the
//! [`Zenon`] SDK entry point, the [`Error`] type, the JSON-RPC API roots ([`LedgerApi`],
//! [`StatsApi`], [`SubscribeApi`], [`EmbeddedApi`]), core primitives
//! ([`Address`], [`struct@Hash`], [`HashHeight`], [`TokenStandard`]), the NOM
//! [`AccountBlockTemplate`] and [`BlockType`], and wallet key types ([`KeyPair`],
//! [`KeyStore`]). Lower-level modules (`abi`, `embedded`, `model`, `utils`,
//! `pow`, `wallet`, `client`) remain available for specialized use. Hosts that
//! need custom proof-of-work can import [`PowProvider`] and [`PowFuture`]
//! directly from this crate.
//!
//! # Feature layout
//!
//! - `default = ["client-ws", "wallet-fs", "native-pow"]` keeps the full SDK
//!   surface source-compatible.
//! - `client-ws` gates the WebSocket transport, the JSON-RPC API roots, and the
//!   `Zenon` SDK entry point.
//! - `wallet-fs` gates the filesystem keystore manager and on-disk wallet path
//!   helpers.
//! - `native-pow` gates the randomized in-process proof-of-work generator.
//!
//! With `--no-default-features` the crate compiles the reduced core: primitives,
//! models, ABI, deterministic crypto helpers, bytes/amount utilities, block
//! hashing, embedded definitions/builders, and proof-of-work
//! verification/data-hash helpers.

// Core protocol modules: always available in every feature configuration.
pub mod abi;
pub mod crypto;
pub mod embedded;
pub mod error;
pub mod model;
pub mod pow;
pub mod primitives;
pub mod publish;
pub mod utils;

// Wallet key/mnemonic derivation is core; filesystem keystore persistence is
// gated behind the `wallet-fs` feature inside the module.
pub mod wallet;

/// On-disk wallet path helpers and directory management.
#[cfg(feature = "wallet-fs")]
pub mod global;

/// JSON-RPC client trait and error types. The WebSocket transport is gated
/// behind the `client-ws` feature inside the module.
pub mod client;

/// JSON-RPC API roots (ledger, stats, subscribe, embedded). Requires a client
/// transport, so gated behind `client-ws`.
#[cfg(feature = "client-ws")]
pub mod api;

/// High-level SDK entry point orchestrating client, wallet, and send/prepare flows.
#[cfg(feature = "client-ws")]
pub mod zenon;

// Recommended imports for common SDK workflows.
pub use error::Error;
pub use model::nom::account_block_template::AccountBlockTemplate;
pub use model::nom::account_block_template::BlockType;
#[cfg(feature = "native-pow")]
pub use pow::provider::NativePowProvider;
pub use pow::provider::{PowFuture, PowProvider};
pub use primitives::address::Address;
pub use primitives::hash::Hash;
pub use primitives::hash_height::HashHeight;
pub use primitives::token_standard::TokenStandard;
pub use wallet::keypair::KeyPair;
pub use wallet::keystore::KeyStore;

#[cfg(feature = "client-ws")]
pub use api::PageQuery;
#[cfg(feature = "client-ws")]
pub use api::embedded::EmbeddedApi;
#[cfg(feature = "client-ws")]
pub use api::ledger::LedgerApi;
#[cfg(feature = "client-ws")]
pub use api::stats::StatsApi;
#[cfg(feature = "client-ws")]
pub use api::subscribe::SubscribeApi;
#[cfg(feature = "client-ws")]
pub use client::factory::{ClientTransport, new_client};
#[cfg(feature = "client-ws")]
pub use client::http::HttpClient;
#[cfg(feature = "client-ws")]
pub use zenon::Zenon;

#[cfg(all(test, feature = "client-ws"))]
mod tests {
    #[test]
    fn public_module_tree_resolves() {
        #[allow(unused_imports)]
        use crate::{
            abi, api, client, crypto, embedded, global, model, pow, primitives, utils, wallet,
            zenon,
        };
    }
}

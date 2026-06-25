//! JSON-RPC client surface.
//!
//! The [`Client`](crate::client::interfaces::Client) trait and
//! [`ClientError`](crate::client::exceptions::ClientError) are core protocol
//! contracts and are always available. The WebSocket and HTTP transports that
//! depend on `jsonrpsee` are gated behind the `client-ws` feature.

pub mod constants;
pub mod exceptions;
pub mod interfaces;

#[cfg(feature = "client-ws")]
pub(crate) mod dial;

#[cfg(feature = "client-ws")]
pub use dial::ConnectionState;

#[cfg(feature = "client-ws")]
pub mod websocket;

#[cfg(feature = "client-ws")]
pub mod http;

#[cfg(feature = "client-ws")]
pub mod factory;

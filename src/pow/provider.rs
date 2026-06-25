//! Pluggable proof-of-work provider abstraction.
//!
//! Account-block preparation routes non-zero-difficulty proof-of-work through a
//! [`PowProvider`] so a host can delegate generation (for example to a worker,
//! an external service, or a future WASM binding) instead of always running the
//! native in-process search.

use crate::error::Error;
use crate::primitives::hash::Hash;
use std::future::Future;
use std::pin::Pin;

/// Error surfaced when nonce resolution has no provider: no provider is
/// configured and the default factory resolved none (the `native-pow` feature is
/// disabled).
///
/// Defined here so the default-provider factory owns the single message the
/// `native-pow`-disabled path returns, keeping the wording pinned in one place.
pub(crate) const NO_PROVIDER_CONFIGURED: &str = "no proof-of-work provider configured; enable the `native-pow` feature or call Zenon::set_pow_provider";

/// The boxed future returned by [`PowProvider::generate_pow`].
///
/// The future is `Send` so native SDK entry point futures remain spawnable on Tokio's
/// multi-threaded runtime.
pub type PowFuture<'a> = Pin<Box<dyn Future<Output = Result<[u8; 8], Error>> + Send + 'a>>;

/// Generates an 8-byte account-block proof-of-work nonce.
///
/// Implementations receive the canonical account-block proof-of-work data hash
/// (see [`crate::pow::account_block_data_hash`]) and the required difficulty,
/// and return a nonce that satisfies [`crate::pow::verify_pow`] for that
/// difficulty. The SDK entry point verifies the returned nonce before placing it on the
/// prepared template.
///
/// Providers are `Send + Sync` so configuring one does not make the native
/// [`Zenon`](crate::zenon::Zenon) SDK entry point non-sendable.
pub trait PowProvider: Send + Sync {
    /// Returns a nonce satisfying `difficulty` for `data_hash`.
    fn generate_pow<'a>(&'a self, data_hash: &'a Hash, difficulty: u64) -> PowFuture<'a>;
}

/// Native in-process proof-of-work provider wrapping
/// [`crate::pow::generate_pow`].
///
/// This is the default provider used by [`crate::zenon::Zenon`] when no
/// provider is configured and the `native-pow` feature is enabled. Hosts
/// without a native RNG should inject their own [`PowProvider`] instead.
#[cfg(feature = "native-pow")]
pub struct NativePowProvider;

#[cfg(feature = "native-pow")]
impl Default for NativePowProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "native-pow")]
impl NativePowProvider {
    /// Creates a native provider.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "native-pow")]
impl PowProvider for NativePowProvider {
    #[cfg(feature = "client-ws")]
    fn generate_pow<'a>(&'a self, data_hash: &'a Hash, difficulty: u64) -> PowFuture<'a> {
        let data_hash = data_hash.clone();
        Box::pin(async move {
            if tokio::runtime::Handle::try_current().is_err() {
                return Ok(crate::pow::generate_pow(&data_hash, difficulty));
            }
            tokio::task::spawn_blocking(move || crate::pow::generate_pow(&data_hash, difficulty))
                .await
                .map_err(|err| Error::generic(format!("native proof-of-work task failed: {err}")))
        })
    }

    #[cfg(not(feature = "client-ws"))]
    fn generate_pow<'a>(&'a self, data_hash: &'a Hash, difficulty: u64) -> PowFuture<'a> {
        Box::pin(async move { Ok(crate::pow::generate_pow(data_hash, difficulty)) })
    }
}

/// Resolves the default proof-of-work provider for the current build.
///
/// This is the single place the `native-pow` feature gate lives for the default
/// provider. When `native-pow` is enabled it returns the native in-process
/// provider; otherwise it returns `None`, and the caller surfaces the
/// "no proof-of-work provider configured" error. Centralizing the gate here keeps
/// `Zenon::resolve_nonce` free of any `#[cfg]` branch.
#[cfg(feature = "native-pow")]
// `Option` is required to share a signature with the `#[cfg(not(...))]` arm,
// which resolves to `None` so `resolve_nonce` can surface the configured
// "no provider" error.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn default_pow_provider() -> Option<Box<dyn PowProvider>> {
    Some(Box::new(NativePowProvider::new()))
}

/// Resolves the default proof-of-work provider for the current build.
///
/// Reduced-core counterpart: with no native generation available, no default
/// provider exists.
#[cfg(not(feature = "native-pow"))]
pub(crate) fn default_pow_provider() -> Option<Box<dyn PowProvider>> {
    None
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;

    /// With `native-pow` enabled, the default factory resolves a provider rather
    /// than `None`, so the default resolution path is identical to a configured
    /// provider's.
    #[cfg(feature = "native-pow")]
    #[test]
    fn default_factory_resolves_a_provider_under_native_pow() {
        let provider = default_pow_provider();
        assert!(
            provider.is_some(),
            "default_pow_provider must return Some under the native-pow feature"
        );
    }

    /// With `native-pow` disabled, the default factory resolves no provider, and
    /// nonce resolution surfaces the configured error. Pins the error-path
    /// behavior for the reduced build.
    #[cfg(not(feature = "native-pow"))]
    #[test]
    fn default_factory_resolves_no_provider_without_native_pow() {
        assert!(
            default_pow_provider().is_none(),
            "default_pow_provider must return None when native-pow is disabled"
        );
    }

    /// Pins the exact wording surfaced when no provider is available, so the
    /// green refactor of `resolve_nonce` preserves the message the reduced build
    /// returns today. The factory is the single owner of this message; the
    /// reduced build's `resolve_nonce` must surface it verbatim.
    #[test]
    fn no_provider_error_wording_is_pinned() {
        assert_eq!(
            NO_PROVIDER_CONFIGURED,
            "no proof-of-work provider configured; enable the `native-pow` feature or call Zenon::set_pow_provider",
        );
    }
}

//! Public API import and documentation checks.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

#[test]
fn recommended_imports_compile() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/public_api_release_imports.rs");
}

/// `SubscriptionEvent` is `#[non_exhaustive]`: a cross-crate match that lists
/// every known variant but omits the `_` wildcard must fail to compile, proving
/// callers cannot match exhaustively and must remain forward-compatible.
#[test]
fn subscription_event_non_exhaustive_requires_wildcard() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/subscription_event_non_exhaustive_requires_wildcard.rs");
}

const LIB_RS: &str = include_str!("../src/lib.rs");

/// The library should re-export common consumer types directly instead of
/// requiring every application to use deep module paths.
#[test]
fn top_level_reexports_are_declared() {
    for needle in [
        "pub use zenon::Zenon",
        "pub use api::ledger::LedgerApi",
        "pub use api::stats::StatsApi",
        "pub use api::subscribe::SubscribeApi",
        "pub use api::embedded::EmbeddedApi",
        "pub use primitives::address::Address",
        "pub use primitives::hash::Hash",
        "pub use primitives::hash_height::HashHeight",
        "pub use primitives::token_standard::TokenStandard",
        "pub use pow::provider::{PowFuture, PowProvider}",
        "pub use pow::provider::NativePowProvider",
        "pub use model::nom::account_block_template::AccountBlockTemplate",
        "pub use wallet::keypair::KeyPair",
        "pub use client::factory::{ClientTransport, new_client}",
        "pub use client::http::HttpClient",
    ] {
        assert!(
            LIB_RS.contains(needle),
            "src/lib.rs is missing re-export: `{needle}`"
        );
    }
}

/// The crate-level docs should point users at the recommended imports and make
/// it clear that lower-level modules are still available.
#[test]
fn crate_docs_describe_recommended_imports() {
    for marker in ["Recommended imports", "Lower-level modules"] {
        assert!(
            LIB_RS.contains(marker),
            "src/lib.rs crate docs should mention `{marker}`"
        );
    }
}

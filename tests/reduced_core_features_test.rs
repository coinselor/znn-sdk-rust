//! Reduced-core feature layout checks.
//!
//! Layout checks assert on `Cargo.toml` text. Build checks compile the library
//! under reduced-core feature sets, so cfg errors such as unconditional imports
//! from gated modules fail here.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

use std::process::Command;

const CARGO_TOML: &str = include_str!("../Cargo.toml");

/// Runs `cargo check --lib` with the given features and asserts it succeeds.
/// Layout text checks can pass while the reduced-core build is broken, so every
/// feature set that excludes `client-ws` must compile.
fn assert_lib_compiles(features: &[&str]) {
    let mut cmd = Command::new(env!("CARGO"));
    cmd.args(["check", "--no-default-features", "--lib"]);
    for feature in features {
        cmd.args(["--features", feature]);
    }
    let output = cmd
        .output()
        .expect("cargo check must be invocable from the test process");
    let feature_label = if features.is_empty() {
        "(none)".to_string()
    } else {
        features.join(", ")
    };
    assert!(
        output.status.success(),
        "reduced-core build with features [{feature_label}] must compile.\n\
         stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The bare reduced-core build (no features) must compile. This catches
/// unconditional imports from modules gated behind `client-ws`, `wallet-fs`, or
/// `native-pow`.
#[test]
fn reduced_core_build_compiles_without_features() {
    assert_lib_compiles(&[]);
}

/// The `wallet-fs`-only build must compile without the websocket transport.
#[test]
fn reduced_core_build_compiles_with_wallet_fs_only() {
    assert_lib_compiles(&["wallet-fs"]);
}

/// The `native-pow`-only build must compile without the websocket transport.
#[test]
fn reduced_core_build_compiles_with_native_pow_only() {
    assert_lib_compiles(&["native-pow"]);
}

fn dependency_line(name: &str) -> Option<&'static str> {
    let prefix = format!("{name} = ");
    CARGO_TOML
        .lines()
        .find(|line| line.trim_start().starts_with(&prefix))
}

/// The reduced-core build mode must be expressible through a `[features]`
/// section with opt-in platform features and a working default set.
#[test]
fn reduced_build_feature_layout_is_defined() {
    assert!(
        CARGO_TOML.contains("[features]"),
        "Cargo.toml must declare a [features] section for the reduced-core feature layout"
    );
    for feature in ["client-ws", "wallet-fs", "native-pow"] {
        assert!(
            CARGO_TOML.contains(feature),
            "Cargo.toml must define the `{feature}` feature"
        );
    }
    assert!(
        CARGO_TOML.contains("default = ["),
        "Cargo.toml must define a default feature set preserving current behavior"
    );
}

/// Platform-specific dependencies must be optional so the reduced-core build
/// can exclude them. The checks are dependency-specific so unrelated optional
/// dependencies do not satisfy the feature-gating contract accidentally.
#[test]
fn io_bound_dependencies_are_gated_behind_optional_features() {
    for dependency in ["jsonrpsee", "getrandom"] {
        let line = dependency_line(dependency).expect("Cargo.toml must declare the dependency");
        assert!(
            line.contains("optional = true"),
            "`{dependency}` must be marked optional = true so the reduced build can drop it"
        );
    }
}

/// The `jsonrpsee` dependency must enable the HTTP client feature alongside the
/// WebSocket client so the HTTP transport compiles under `client-ws`. The
/// dependency declaration spans multiple lines, so this reads the whole file.
#[test]
fn jsonrpsee_dependency_enables_the_http_client_feature() {
    assert!(
        CARGO_TOML.contains("\"http-client\""),
        "Cargo.toml must enable the `http-client` feature on the jsonrpsee dependency"
    );
}

/// Feature definitions must wire optional dependencies to the features that
/// expose the corresponding platform behavior.
#[test]
fn feature_definitions_reference_their_optional_dependencies() {
    for needle in [
        "client-ws = [",
        "dep:jsonrpsee",
        "native-pow = [",
        "dep:getrandom",
        "wallet-fs = [",
    ] {
        assert!(
            CARGO_TOML.contains(needle),
            "Cargo.toml feature layout is missing `{needle}`"
        );
    }
}

/// With high-level SDK features enabled, the entry point, websocket client,
/// core keystore, and native `PoW` remain importable and usable. Reduced-core
/// builds compile this test module without these imports.
#[cfg(all(feature = "client-ws", feature = "native-pow"))]
#[test]
fn default_build_keeps_entry_point_client_wallet_and_pow_importable() {
    use znn_sdk_rust::client::websocket::WsClient;
    use znn_sdk_rust::pow;
    use znn_sdk_rust::primitives::hash::Hash;
    use znn_sdk_rust::wallet::keystore::KeyStore;
    use znn_sdk_rust::zenon::Zenon;

    let _ = std::any::type_name::<Zenon>();
    let _ = std::any::type_name::<WsClient>();
    let _ = std::any::type_name::<KeyStore>();
    let native_pow = pow::generate_pow as fn(&Hash, u64) -> [u8; 8];
    assert!(
        std::any::type_name_of_val(&native_pow).contains("fn"),
        "native PoW function must remain importable"
    );
}

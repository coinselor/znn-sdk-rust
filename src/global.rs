//! SDK-global state: version, on-disk directory layout, and the chain
//! identifier.

use crate::error::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// SDK version string, sourced from the crate's published version so it cannot
/// drift from `Cargo.toml`.
pub const ZNN_SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// On-disk root directory name for Zenon data.
pub const ZNN_ROOT_DIRECTORY: &str = "znn";

static CHAIN_ID: AtomicU32 = AtomicU32::new(1);

/// The three on-disk directories the SDK uses: main, wallet, and cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZnnPaths {
    main: PathBuf,
    wallet: PathBuf,
    cache: PathBuf,
}

impl ZnnPaths {
    /// Creates paths from their resolved directory locations.
    pub fn new(main: PathBuf, wallet: PathBuf, cache: PathBuf) -> Self {
        Self {
            main,
            wallet,
            cache,
        }
    }

    /// Returns the main directory.
    pub fn main(&self) -> &Path {
        &self.main
    }

    /// Returns the wallet directory.
    pub fn wallet(&self) -> &Path {
        &self.wallet
    }

    /// Returns the cache directory.
    pub fn cache(&self) -> &Path {
        &self.cache
    }
}

impl Default for ZnnPaths {
    /// Resolves the per-platform default layout (see [`default_paths`]).
    fn default() -> Self {
        default_paths()
    }
}

/// Returns the default on-disk directory layout for the current platform.
///
/// Linux resolves to `$HOME/.znn`, macOS to `$HOME/Library/znn`, Windows to
/// `%AppData%/znn`, and any other platform to `$HOME/znn`. The `wallet` and
/// `cache` directories are `main/wallet` and `main/syrius` respectively.
pub fn default_paths() -> ZnnPaths {
    let main = default_main_directory();
    let wallet = main.join("wallet");
    let cache = main.join("syrius");
    ZnnPaths::new(main, wallet, cache)
}

#[cfg(target_os = "linux")]
fn default_main_directory() -> PathBuf {
    home_directory().join(".znn")
}

#[cfg(all(unix, not(target_os = "linux")))]
fn default_main_directory() -> PathBuf {
    home_directory().join("Library").join(ZNN_ROOT_DIRECTORY)
}

#[cfg(target_os = "windows")]
fn default_main_directory() -> PathBuf {
    let app_data = std::env::var("AppData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_directory());
    app_data.join(ZNN_ROOT_DIRECTORY)
}

#[cfg(not(any(target_os = "linux", target_os = "windows", unix)))]
fn default_main_directory() -> PathBuf {
    home_directory().join(ZNN_ROOT_DIRECTORY)
}

fn home_directory() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

/// Creates the wallet and cache directories of [`default_paths`] when they do
/// not exist, returning an error only on an I/O failure.
pub fn ensure_directories_exist() -> Result<(), Error> {
    ensure_directories_exist_at(&default_paths())
}

/// Creates the wallet and cache directories of `paths` when they do not exist.
///
/// This is the path-taking variant of [`ensure_directories_exist`]; the zero-arg
/// form uses the per-platform [`default_paths`] layout.
pub fn ensure_directories_exist_at(paths: &ZnnPaths) -> Result<(), Error> {
    std::fs::create_dir_all(paths.wallet()).map_err(|e| io_error(&e))?;
    std::fs::create_dir_all(paths.cache()).map_err(|e| io_error(&e))?;
    Ok(())
}

fn io_error(error: &std::io::Error) -> Error {
    Error::Generic(format!("directory operation failed: {error}"))
}

/// Returns the process-wide chain identifier.
pub fn chain_identifier() -> u32 {
    CHAIN_ID.load(Ordering::Relaxed)
}

/// Sets the process-wide chain identifier.
pub fn set_chain_identifier(identifier: u32) {
    CHAIN_ID.store(identifier, Ordering::Relaxed);
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    /// Serializes tests that mutate the process-wide `HOME`.
    fn home_guard() -> &'static Mutex<()> {
        static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        GUARD.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn version_and_root_directory_constants_are_stable() {
        // The version must track the crate's published version rather than a
        // hard-coded literal that can drift.
        assert_eq!(
            ZNN_SDK_VERSION,
            env!("CARGO_PKG_VERSION"),
            "ZNN_SDK_VERSION must equal the crate's published version"
        );
        assert_eq!(ZNN_ROOT_DIRECTORY, "znn");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn default_paths_resolve_from_home_on_linux() {
        let _guard = home_guard().lock().expect("home guard locks");
        let home = std::env::var("HOME").expect("HOME is set");
        let paths = default_paths();
        let root = PathBuf::from(&home).join(".znn");
        assert_eq!(paths.main(), &root, "main must be $HOME/.znn");
        assert_eq!(
            paths.wallet(),
            &root.join("wallet"),
            "wallet must be main/wallet"
        );
        assert_eq!(
            paths.cache(),
            &root.join("syrius"),
            "cache must be main/syrius"
        );
    }

    #[test]
    fn default_paths_round_trips_through_znnpaths_default() {
        // `ZnnPaths::default()` reads `HOME` via `default_paths()`.
        let _guard = home_guard().lock().expect("home guard locks");
        assert_eq!(ZnnPaths::default(), default_paths());
    }

    #[test]
    fn chain_identifier_defaults_and_round_trips() {
        assert_eq!(chain_identifier(), 1, "chain identifier defaults to 1");
        set_chain_identifier(100);
        assert_eq!(
            chain_identifier(),
            100,
            "set_chain_identifier must take effect"
        );
        // Restore the default value for later tests.
        set_chain_identifier(1);
    }

    #[test]
    fn ensure_directories_exist_at_creates_wallet_and_cache() {
        let base = std::env::temp_dir().join(format!("znn-global-test-{}", std::process::id()));
        let paths = ZnnPaths::new(base.clone(), base.join("wallet"), base.join("syrius"));
        ensure_directories_exist_at(&paths).expect("directories are created");
        assert!(paths.wallet().exists(), "wallet directory must exist");
        assert!(paths.cache().exists(), "cache directory must exist");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[allow(unsafe_code)]
    fn ensure_directories_exist_creates_the_default_layout_from_home() {
        let _guard = home_guard().lock().expect("home guard locks");
        let base = std::env::temp_dir().join(format!("znn-global-zero-arg-{}", std::process::id()));
        let prior = std::env::var("HOME").ok();
        // This is the mutex-guarded test that mutates `HOME`.
        unsafe {
            std::env::set_var("HOME", &base);
        }
        ensure_directories_exist().expect("directories are created from default_paths");
        let paths = default_paths();
        assert!(paths.wallet().exists(), "wallet directory must exist");
        assert!(paths.cache().exists(), "cache directory must exist");
        unsafe {
            match &prior {
                Some(value) => std::env::set_var("HOME", value),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}

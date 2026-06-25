//! Embedded-contract protocol constants: limits, fees, regex patterns, symbol
//! exceptions, and status ordinals.
//!
//! Amounts are `u64` products of `ONE_ZNN` / `ONE_QSR`; the 256-bit token-supply
//! bounds are returned by functions because `BigUint` is not `const`-constructible.

use crate::utils::nom_constants::{ONE_QSR, ONE_ZNN};
use num_bigint::BigUint;

/// Genesis momentum timestamp (Unix seconds).
pub const GENESIS_TIMESTAMP: u64 = 1_637_755_200;

// === Plasma ===

/// Minimum QSR amount that may be fused.
pub const FUSE_MIN_QSR_AMOUNT: u64 = 10 * ONE_QSR;

/// Minimum plasma amount required for an account block.
pub const MIN_PLASMA_AMOUNT: u64 = 21_000;

// === Pillar ===

/// Minimum delegation amount.
pub const K_MIN_DELEGATION_AMOUNT: u64 = ONE_ZNN;

/// ZNN cost to register a pillar.
pub const PILLAR_REGISTER_ZNN_AMOUNT: u64 = 15_000 * ONE_ZNN;

/// QSR cost to register a pillar.
pub const PILLAR_REGISTER_QSR_AMOUNT: u64 = 150_000 * ONE_QSR;

/// Maximum pillar name length.
pub const PILLAR_NAME_MAX_LENGTH: usize = 40;

/// Pillar-name regex pattern (shared with token names).
pub const PILLAR_NAME_REGEXP: &str = r"^([a-zA-Z0-9]+[-._]?)*[a-zA-Z0-9]$";

// === Sentinel ===

/// ZNN cost to register a sentinel.
pub const SENTINEL_REGISTER_ZNN_AMOUNT: u64 = 5_000 * ONE_ZNN;

/// QSR cost to register a sentinel.
pub const SENTINEL_REGISTER_QSR_AMOUNT: u64 = 50_000 * ONE_QSR;

// === Staking ===

/// Minimum ZNN staking amount.
pub const STAKE_MIN_ZNN_AMOUNT: u64 = ONE_ZNN;

/// One staking time unit in seconds (30 days).
pub const STAKE_TIME_UNIT_SEC: u64 = 30 * 24 * 60 * 60;

/// Minimum stake duration in seconds.
pub const STAKE_TIME_MIN_SEC: u64 = STAKE_TIME_UNIT_SEC;

/// Maximum stake duration in seconds.
pub const STAKE_TIME_MAX_SEC: u64 = 12 * STAKE_TIME_UNIT_SEC;

/// Human-readable name of one staking time unit.
pub const STAKE_UNIT_DURATION_NAME: &str = "month";

// === Token ===

/// ZNN fee to issue a token standard.
pub const TOKEN_ZTS_ISSUE_FEE_IN_ZNN: u64 = ONE_ZNN;

/// Minimum non-zero token total/max supply.
pub const K_MIN_TOKEN_TOTAL_MAX_SUPPLY: u64 = 1;

/// Maximum token name length.
pub const TOKEN_NAME_MAX_LENGTH: usize = 40;

/// Maximum token symbol length.
pub const TOKEN_SYMBOL_MAX_LENGTH: usize = 10;

/// Token-name regex pattern (shared with pillar names).
pub const TOKEN_NAME_REGEXP: &str = r"^([a-zA-Z0-9]+[-._]?)*[a-zA-Z0-9]$";

/// Token-symbol regex pattern.
pub const TOKEN_SYMBOL_REGEXP: &str = r"^[A-Z0-9]+$";

/// Token-domain regex pattern.
pub const TOKEN_DOMAIN_REGEXP: &str =
    r"^([A-Za-z0-9][A-Za-z0-9-]{0,61}[A-Za-z0-9]\.)+[A-Za-z]{2,}$";

/// Token symbols reserved by the protocol.
pub const TOKEN_SYMBOL_EXCEPTIONS: &[&str] = &["ZNN", "QSR"];

/// Returns the upper token-supply bound `2^255`.
pub fn k_big_p255() -> BigUint {
    BigUint::from(1u8) << 255
}

/// Returns the lower token-supply bound `2^255 - 1`.
pub fn k_big_p255_m1() -> BigUint {
    k_big_p255() - 1u8
}

// === Accelerator ===

/// ZNN fee to create an accelerator project.
pub const PROJECT_CREATION_FEE_IN_ZNN: u64 = ONE_ZNN;

/// Maximum ZNN funds an accelerator project may request.
pub const K_ZNN_PROJECT_MAXIMUM_FUNDS: u64 = 5_000 * ONE_ZNN;

/// Maximum QSR funds an accelerator project may request.
pub const K_QSR_PROJECT_MAXIMUM_FUNDS: u64 = 50_000 * ONE_QSR;

/// Minimum ZNN funds an accelerator project may request.
pub const K_ZNN_PROJECT_MINIMUM_FUNDS: u64 = 10 * ONE_ZNN;

/// Minimum QSR funds an accelerator project may request.
pub const K_QSR_PROJECT_MINIMUM_FUNDS: u64 = 100 * ONE_QSR;

/// Maximum accelerator project description length.
pub const PROJECT_DESCRIPTION_MAX_LENGTH: usize = 240;

/// Maximum accelerator project name length.
pub const PROJECT_NAME_MAX_LENGTH: usize = 30;

/// Accelerator project status: voting.
pub const PROJECT_VOTING_STATUS: u32 = 0;

/// Accelerator project status: active.
pub const PROJECT_ACTIVE_STATUS: u32 = 1;

/// Accelerator project status: paid.
pub const PROJECT_PAID_STATUS: u32 = 2;

/// Accelerator project status: closed.
pub const PROJECT_CLOSED_STATUS: u32 = 3;

/// Accelerator project URL regex pattern.
pub const PROJECT_URL_REGEXP: &str =
    r"^[a-zA-Z0-9]{2,60}\.[a-zA-Z]{1,6}([a-zA-Z0-9()@:%_\+.~#?&/=-]{0,100})$";

// === Swap ===

/// Start timestamp for swap asset decay.
pub const SWAP_ASSET_DECAY_TIMESTAMP_START: u64 = 1_645_531_200;

/// Epoch offset before swap asset decay begins.
pub const SWAP_ASSET_DECAY_EPOCHS_OFFSET: u64 = 30 * 3;

/// Number of epochs per swap asset decay tick.
pub const SWAP_ASSET_DECAY_TICK_EPOCHS: u64 = 30;

/// Percentage value lost per swap asset decay tick.
pub const SWAP_ASSET_DECAY_TICK_VALUE_PERCENTAGE: u64 = 10;

// === Spork ===

/// Minimum spork name length.
pub const SPORK_NAME_MIN_LENGTH: usize = 5;

/// Maximum spork name length.
pub const SPORK_NAME_MAX_LENGTH: usize = 40;

/// Maximum spork description length.
pub const SPORK_DESCRIPTION_MAX_LENGTH: usize = 400;

// === Htlc ===

/// Minimum HTLC preimage length.
pub const HTLC_PREIMAGE_MIN_LENGTH: usize = 1;

/// Maximum HTLC preimage length.
pub const HTLC_PREIMAGE_MAX_LENGTH: usize = 255;

/// Default HTLC preimage length.
pub const HTLC_PREIMAGE_DEFAULT_LENGTH: usize = 32;

/// HTLC hash type: SHA3-256.
pub const HTLC_HASH_TYPE_SHA3: u32 = 0;

/// HTLC hash type: SHA-256.
pub const HTLC_HASH_TYPE_SHA256: u32 = 1;

// === Bridge ===

/// Minimum number of bridge guardians.
pub const BRIDGE_MIN_GUARDIANS: u32 = 5;

/// Maximum bridge fee (basis points).
pub const BRIDGE_MAXIMUM_FEE: u32 = 10_000;

/// Sentinel value confirming `ONE_ZNN` / `ONE_QSR` are in scope for the products.
const _SIZED: (u64, u64) = (ONE_ZNN, ONE_QSR);

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn amounts_and_timestamps_match_expected_values() {
        assert_eq!(GENESIS_TIMESTAMP, 1_637_755_200);
        assert_eq!(FUSE_MIN_QSR_AMOUNT, 10 * ONE_QSR);
        assert_eq!(MIN_PLASMA_AMOUNT, 21_000);
        assert_eq!(K_MIN_DELEGATION_AMOUNT, ONE_ZNN);
        assert_eq!(PILLAR_REGISTER_ZNN_AMOUNT, 15_000 * ONE_ZNN);
        assert_eq!(PILLAR_REGISTER_QSR_AMOUNT, 150_000 * ONE_QSR);
        assert_eq!(SENTINEL_REGISTER_ZNN_AMOUNT, 5_000 * ONE_ZNN);
        assert_eq!(SENTINEL_REGISTER_QSR_AMOUNT, 50_000 * ONE_QSR);
        assert_eq!(STAKE_MIN_ZNN_AMOUNT, ONE_ZNN);
        assert_eq!(STAKE_TIME_UNIT_SEC, 2_592_000);
        assert_eq!(STAKE_TIME_MIN_SEC, 2_592_000);
        assert_eq!(STAKE_TIME_MAX_SEC, 31_104_000);
        assert_eq!(STAKE_UNIT_DURATION_NAME, "month");
        assert_eq!(TOKEN_ZTS_ISSUE_FEE_IN_ZNN, ONE_ZNN);
        assert_eq!(K_MIN_TOKEN_TOTAL_MAX_SUPPLY, 1);
        assert_eq!(PROJECT_CREATION_FEE_IN_ZNN, ONE_ZNN);
        assert_eq!(K_ZNN_PROJECT_MAXIMUM_FUNDS, 5_000 * ONE_ZNN);
        assert_eq!(K_QSR_PROJECT_MAXIMUM_FUNDS, 50_000 * ONE_QSR);
        assert_eq!(K_ZNN_PROJECT_MINIMUM_FUNDS, 10 * ONE_ZNN);
        assert_eq!(K_QSR_PROJECT_MINIMUM_FUNDS, 100 * ONE_QSR);
        assert_eq!(SWAP_ASSET_DECAY_TIMESTAMP_START, 1_645_531_200);
        assert_eq!(SWAP_ASSET_DECAY_EPOCHS_OFFSET, 90);
        assert_eq!(SWAP_ASSET_DECAY_TICK_EPOCHS, 30);
        assert_eq!(SWAP_ASSET_DECAY_TICK_VALUE_PERCENTAGE, 10);
        assert_eq!(BRIDGE_MIN_GUARDIANS, 5);
        assert_eq!(BRIDGE_MAXIMUM_FEE, 10_000);
    }

    #[test]
    fn length_limits_match_expected_values() {
        assert_eq!(PILLAR_NAME_MAX_LENGTH, 40);
        assert_eq!(TOKEN_NAME_MAX_LENGTH, 40);
        assert_eq!(TOKEN_SYMBOL_MAX_LENGTH, 10);
        assert_eq!(PROJECT_DESCRIPTION_MAX_LENGTH, 240);
        assert_eq!(PROJECT_NAME_MAX_LENGTH, 30);
        assert_eq!(SPORK_NAME_MIN_LENGTH, 5);
        assert_eq!(SPORK_NAME_MAX_LENGTH, 40);
        assert_eq!(SPORK_DESCRIPTION_MAX_LENGTH, 400);
        assert_eq!(HTLC_PREIMAGE_MIN_LENGTH, 1);
        assert_eq!(HTLC_PREIMAGE_MAX_LENGTH, 255);
        assert_eq!(HTLC_PREIMAGE_DEFAULT_LENGTH, 32);
    }

    #[test]
    fn status_ordinals_match_expected_values() {
        assert_eq!(PROJECT_VOTING_STATUS, 0);
        assert_eq!(PROJECT_ACTIVE_STATUS, 1);
        assert_eq!(PROJECT_PAID_STATUS, 2);
        assert_eq!(PROJECT_CLOSED_STATUS, 3);
        assert_eq!(HTLC_HASH_TYPE_SHA3, 0);
        assert_eq!(HTLC_HASH_TYPE_SHA256, 1);
    }

    #[test]
    fn regex_patterns_match_expected_values_verbatim() {
        assert_eq!(PILLAR_NAME_REGEXP, r"^([a-zA-Z0-9]+[-._]?)*[a-zA-Z0-9]$");
        assert_eq!(TOKEN_NAME_REGEXP, r"^([a-zA-Z0-9]+[-._]?)*[a-zA-Z0-9]$");
        assert_eq!(TOKEN_SYMBOL_REGEXP, r"^[A-Z0-9]+$");
        assert_eq!(
            TOKEN_DOMAIN_REGEXP,
            r"^([A-Za-z0-9][A-Za-z0-9-]{0,61}[A-Za-z0-9]\.)+[A-Za-z]{2,}$"
        );
        assert_eq!(
            PROJECT_URL_REGEXP,
            r"^[a-zA-Z0-9]{2,60}\.[a-zA-Z]{1,6}([a-zA-Z0-9()@:%_\+.~#?&/=-]{0,100})$"
        );
    }

    #[test]
    fn symbol_exceptions_match_expected_values() {
        assert_eq!(TOKEN_SYMBOL_EXCEPTIONS, ["ZNN", "QSR"]);
    }

    #[test]
    fn big_256_bounds_match_powers_of_two() {
        let p255 = BigUint::from(1u8) << 255;
        assert_eq!(k_big_p255(), p255);
        assert_eq!(k_big_p255_m1() + 1u8, p255);
        assert_eq!(k_big_p255_m1(), p255 - 1u8);
    }
}

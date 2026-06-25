//! Network of Momentum protocol constants.

use core::time::Duration;

/// Number of decimal places for ZNN and QSR.
pub const COIN_DECIMALS: u8 = 8;

/// One ZNN expressed in its smallest unit (`10^COIN_DECIMALS`).
pub const ONE_ZNN: u64 = 100_000_000;

/// One QSR expressed in its smallest unit (`10^COIN_DECIMALS`).
pub const ONE_QSR: u64 = 100_000_000;

/// Target interval between momentums.
pub const INTERVAL_BETWEEN_MOMENTUMS: Duration = Duration::from_secs(10);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_decimals_is_8() {
        assert_eq!(COIN_DECIMALS, 8);
    }

    #[test]
    fn one_znn_is_one_hundred_million() {
        assert_eq!(ONE_ZNN, 100_000_000);
    }

    #[test]
    fn one_qsr_is_one_hundred_million() {
        assert_eq!(ONE_QSR, 100_000_000);
    }

    #[test]
    fn smallest_unit_values_match_coin_decimals() {
        let expected = 10u64.pow(u32::from(COIN_DECIMALS));
        assert_eq!(ONE_ZNN, expected, "ONE_ZNN must equal 10^COIN_DECIMALS");
        assert_eq!(ONE_QSR, expected, "ONE_QSR must equal 10^COIN_DECIMALS");
    }

    #[test]
    fn interval_between_momentums_is_ten_seconds() {
        assert_eq!(INTERVAL_BETWEEN_MOMENTUMS, Duration::from_secs(10));
    }
}

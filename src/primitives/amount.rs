//! Validated on-chain [`Amount`] type.
//!
//! A protocol amount is a non-negative integer base-unit value whose bit length
//! is at most [`MAX_BITS`]. The type enforces both rules at construction so an
//! invalid amount cannot reach transaction assembly.

use crate::error::Error;
use crate::utils::amount::{add_decimals, extract_decimals};
use core::fmt;
use num_bigint::{BigInt, BigUint};

/// Maximum bit length of a protocol amount.
pub const MAX_BITS: u64 = 255;

/// A non-negative on-chain amount of at most [`MAX_BITS`] bits, stored as an
/// unscaled base-unit value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(BigUint);

impl Amount {
    /// Returns the zero amount.
    pub fn zero() -> Self {
        Self(BigUint::from(0u8))
    }

    /// Parses a human-readable decimal `amount` at the given `decimals` count
    /// into a validated `Amount`.
    ///
    /// Returns [`Error::InvalidInput`] for a malformed, negative, or over-wide
    /// amount.
    pub fn from_str_units(amount: &str, decimals: u8) -> Result<Self, Error> {
        Self::try_from(extract_decimals(amount, decimals)?)
    }

    /// Renders the amount as a plain decimal string at the given `decimals`
    /// count.
    pub fn to_str_units(&self, decimals: u8) -> String {
        add_decimals(&self.to_bigint(), decimals)
    }

    /// Returns the unscaled base-unit value.
    pub fn as_biguint(&self) -> &BigUint {
        &self.0
    }

    /// Returns the unscaled base-unit value as a signed integer.
    pub fn to_bigint(&self) -> BigInt {
        BigInt::from(self.0.clone())
    }

    /// Returns the bit length of the amount.
    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    /// Validates the [`MAX_BITS`] bound and wraps the value.
    fn checked(value: BigUint) -> Result<Self, Error> {
        if value.bits() > MAX_BITS {
            return Err(Error::InvalidInput(format!(
                "amount exceeds {MAX_BITS} bits: {} bits",
                value.bits()
            )));
        }
        Ok(Self(value))
    }
}

impl TryFrom<BigUint> for Amount {
    type Error = Error;

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        Self::checked(value)
    }
}

impl TryFrom<BigInt> for Amount {
    type Error = Error;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        match value.to_biguint() {
            Some(value) => Self::checked(value),
            None => Err(Error::InvalidInput("amount is negative".to_string())),
        }
    }
}

impl fmt::Display for Amount {
    /// Renders the unscaled base-unit integer.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    /// `2^255`, the smallest value with a bit length above [`MAX_BITS`].
    fn two_pow_255() -> BigUint {
        BigUint::from(1u8) << 255usize
    }

    #[test]
    fn max_bits_is_255() {
        assert_eq!(MAX_BITS, 255);
    }

    #[test]
    fn accepts_a_small_value() {
        let amount = Amount::try_from(BigUint::from(1000u32)).expect("1000 is valid");
        assert_eq!(amount.as_biguint(), &BigUint::from(1000u32));
    }

    #[test]
    fn accepts_zero() {
        let amount = Amount::try_from(BigUint::from(0u32)).expect("zero is valid");
        assert_eq!(amount, Amount::zero());
    }

    #[test]
    fn accepts_the_255_bit_maximum() {
        let max = two_pow_255() - 1u8;
        let amount = Amount::try_from(max).expect("2^255 - 1 is valid");
        assert_eq!(amount.bits(), 255, "2^255 - 1 has a 255-bit length");
    }

    #[test]
    fn rejects_a_value_wider_than_255_bits() {
        let result = Amount::try_from(two_pow_255());
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "2^255 (256 bits) must be rejected, got {result:?}"
        );
    }

    #[test]
    fn try_from_bigint_rejects_a_negative_value() {
        let result = Amount::try_from(BigInt::from(-5));
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a negative integer must be rejected, got {result:?}"
        );
    }

    #[test]
    fn try_from_bigint_accepts_a_non_negative_value() {
        let amount = Amount::try_from(BigInt::from(5)).expect("5 is valid");
        assert_eq!(amount.as_biguint(), &BigUint::from(5u32));
    }

    #[test]
    fn from_str_units_parses_a_decimal_amount() {
        let amount = Amount::from_str_units("1.5", 8).expect("1.5 at 8 decimals is valid");
        assert_eq!(amount.as_biguint(), &BigUint::from(150_000_000u32));
    }

    #[test]
    fn from_str_units_rejects_a_negative_decimal_amount() {
        let result = Amount::from_str_units("-1", 8);
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a negative decimal amount must be rejected, got {result:?}"
        );
    }

    #[test]
    fn from_str_units_rejects_an_over_wide_decimal_amount() {
        let result = Amount::from_str_units(&two_pow_255().to_string(), 0);
        assert!(
            matches!(&result, Err(Error::InvalidInput(_))),
            "a 256-bit decimal amount must be rejected, got {result:?}"
        );
    }

    #[test]
    fn from_str_units_round_trips_through_to_str_units() {
        let amount = Amount::from_str_units("1.5", 8).expect("valid");
        assert_eq!(amount.to_str_units(8), "1.5");
    }

    #[test]
    fn zero_renders_as_zero() {
        assert_eq!(Amount::zero().to_str_units(8), "0");
    }

    #[test]
    fn equal_amounts_compare_equal() {
        let a = Amount::try_from(BigUint::from(1000u32)).expect("valid");
        let b = Amount::try_from(BigUint::from(1000u32)).expect("valid");
        assert_eq!(a, b);
    }

    #[test]
    fn ordering_follows_numeric_value() {
        let one = Amount::try_from(BigUint::from(1u32)).expect("valid");
        let two = Amount::try_from(BigUint::from(2u32)).expect("valid");
        assert!(one < two, "1 < 2");
        assert!(two > one, "2 > 1");
    }
}

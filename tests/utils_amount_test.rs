//! Integration tests for `crate::utils::amount`.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use znn_sdk_rust::Error;
use znn_sdk_rust::utils::amount::{BigInt, add_decimals, extract_decimals};

fn bi(n: i64) -> BigInt {
    BigInt::from(n)
}

// add_decimals.

#[test]
fn add_decimals_documented_examples() {
    assert_eq!(add_decimals(&bi(1_234_500), 3), "1234.5");
    assert_eq!(add_decimals(&bi(100_000), 2), "1000");
}

#[test]
fn add_decimals_scale_floors_at_zero() {
    assert_eq!(add_decimals(&bi(1000), 2), "10");
    assert_eq!(add_decimals(&bi(1000), 0), "1000");
}

#[test]
fn add_decimals_strips_trailing_fractional_zeros() {
    assert_eq!(add_decimals(&bi(1200), 3), "1.2");
    assert_eq!(add_decimals(&bi(123_400), 4), "12.34");
}

#[test]
fn add_decimals_handles_negative_numbers() {
    assert_eq!(add_decimals(&bi(-1000), 2), "-10");
    assert_eq!(add_decimals(&bi(-1200), 3), "-1.2");
}

#[test]
fn add_decimals_zero_stays_zero_regardless_of_decimals() {
    assert_eq!(add_decimals(&bi(0), 0), "0");
    assert_eq!(add_decimals(&bi(0), 1), "0");
    assert_eq!(add_decimals(&bi(0), 18), "0");
}

// extract_decimals.

#[test]
fn extract_decimals_appends_zeros_to_integer_amount() {
    assert_eq!(extract_decimals("1000", 2).expect("parses"), bi(100_000));
}

#[test]
fn extract_decimals_zero_decimals_leaves_integer_unchanged() {
    assert_eq!(extract_decimals("1000", 0).expect("parses"), bi(1000));
}

#[test]
fn extract_decimals_empty_at_zero_decimals_is_zero() {
    assert_eq!(extract_decimals("", 0).expect("parses"), bi(0));
}

#[test]
fn extract_decimals_pads_short_fraction() {
    assert_eq!(
        extract_decimals("1234.5", 3).expect("parses"),
        bi(1_234_500)
    );
}

#[test]
fn extract_decimals_pads_fraction_to_exact_decimals() {
    assert_eq!(extract_decimals("12.34", 4).expect("parses"), bi(123_400));
}

#[test]
fn extract_decimals_truncates_long_fraction() {
    assert_eq!(extract_decimals("1.23456", 2).expect("parses"), bi(123));
}

#[test]
fn extract_decimals_parses_negative_amounts() {
    assert_eq!(extract_decimals("-10", 2).expect("parses"), bi(-1000));
    assert_eq!(extract_decimals("-1.2", 3).expect("parses"), bi(-1200));
}

#[test]
fn extract_decimals_rejects_non_numeric_input() {
    let err = extract_decimals("abc", 2).expect_err("non-numeric must be rejected");
    assert!(
        matches!(err, Error::InvalidInput(_)),
        "expected Error::InvalidInput, got {err:?}"
    );
}

#[test]
fn extract_decimals_rejects_more_than_one_decimal_point() {
    let err = extract_decimals("1.2.3", 2).expect_err("multiple decimal points must be rejected");
    assert!(
        matches!(err, Error::InvalidInput(_)),
        "expected Error::InvalidInput, got {err:?}"
    );
}

// Round-trips.

#[test]
fn extract_then_add_reproduces_canonical_amount() {
    let unscaled = extract_decimals("1234.5", 3).expect("parses");
    assert_eq!(add_decimals(&unscaled, 3), "1234.5");
}

#[test]
fn add_then_extract_reproduces_unscaled_value() {
    let rendered = add_decimals(&bi(1_234_500), 3);
    assert_eq!(
        extract_decimals(&rendered, 3).expect("parses"),
        bi(1_234_500)
    );
}

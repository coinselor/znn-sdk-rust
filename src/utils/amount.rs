//! Token amount conversions between a human-readable decimal string and the
//! unscaled base-unit integer value used throughout the protocol.
//!
//! The arbitrary-precision signed integer type [`BigInt`] is re-exported here
//! so consumers use the SDK's type directly.

use crate::error::Error;
pub use num_bigint::BigInt;
use num_bigint::Sign;

/// Parses a human-readable decimal `amount` into its unscaled [`BigInt`] value
/// for the given `decimals` count.
///
/// With no fractional part the integer digits are followed by `decimals` zeros;
/// the empty string at `decimals == 0` is zero. With a fractional part the
/// fractional digits are right-padded with zeros when shorter than `decimals`
/// and truncated (not rounded) when longer. Returns [`Error::InvalidInput`] for
/// a malformed amount (non-numeric characters or more than one decimal point).
pub fn extract_decimals(amount: &str, decimals: u8) -> Result<BigInt, Error> {
    let decimals = usize::from(decimals);

    let (sign, digits) = match amount.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", amount),
    };

    // Reject more than one decimal point.
    if digits.matches('.').count() > 1 {
        return Err(Error::InvalidInput(format!(
            "amount has more than one decimal point: {amount}"
        )));
    }

    let mut parts = digits.splitn(2, '.');
    let integer = parts.next().unwrap_or("");
    let fraction = parts.next();

    // Reject any non-digit character in the integer or fractional part.
    if !integer.bytes().all(|b| b.is_ascii_digit())
        || fraction.is_some_and(|f| !f.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(Error::InvalidInput(format!("malformed amount: {amount}")));
    }

    let fraction = match fraction {
        Some(f) if f.len() > decimals => f.get(..decimals).unwrap_or(""),
        Some(f) => f,
        None => "",
    };
    let padding = decimals.saturating_sub(fraction.len());
    let combined = format!("{sign}{integer}{fraction}{}", "0".repeat(padding));

    // Treat the empty amount as zero when `decimals == 0`.
    // A non-empty input with no digits, such as `"-"`, falls through to parsing
    // and is rejected.
    if combined.is_empty() {
        return Ok(BigInt::from(0));
    }

    combined
        .parse::<BigInt>()
        .map_err(|e| Error::InvalidInput(format!("malformed amount {amount}: {e}")))
}

/// Renders an unscaled [`BigInt`] `number` as a plain (non-exponential) decimal
/// string for the given `decimals` count, stripping trailing fractional zeros
/// while the scale remains above `0`, preserving the sign, and rendering zero
/// as `"0"`.
pub fn add_decimals(number: &BigInt, decimals: u8) -> String {
    let ten = BigInt::from(10);
    let zero = BigInt::from(0);
    let mut int_val = number.clone();
    let mut scale = usize::from(decimals);

    // Strip trailing zero digits while the scale is still above 0, so trailing
    // fractional zeros are removed but whole-number zeros are preserved.
    while scale > 0 && &int_val % &ten == zero {
        int_val = &int_val / &ten;
        scale -= 1;
    }

    to_plain_string(&int_val, scale)
}

/// Renders an unscaled `int_val` at the given `scale` as a plain decimal string,
/// inserting the decimal point so the fractional part has exactly `scale` digits.
fn to_plain_string(int_val: &BigInt, scale: usize) -> String {
    if scale == 0 {
        return int_val.to_string();
    }
    let sign = if int_val.sign() == Sign::Minus {
        "-"
    } else {
        ""
    };
    let digits = int_val.magnitude().to_string();
    let padded = if digits.len() <= scale {
        format!("{}{digits}", "0".repeat(scale + 1 - digits.len()))
    } else {
        digits
    };
    let (int_part, frac_part) = padded.split_at(padded.len() - scale);
    format!("{sign}{int_part}.{frac_part}")
}

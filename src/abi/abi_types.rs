//! ABI type taxonomy plus single-value encode/decode entry points.
//!
//! Each [`AbiType`] knows how to [`AbiType::parse`] its canonical name, report
//! its head/fixed size and dynamic-ness, and encode/decode a single [`AbiValue`].
//! Word types always occupy 32 bytes; dynamic types (`string`, `bytes`, dynamic
//! arrays) are length-prefixed. The tuple codec in [`crate::abi::abi`] builds on
//! this metadata.

#![allow(clippy::indexing_slicing)]

use crate::error::Error;
use crate::primitives::address::{self, Address};
use crate::primitives::hash::{self, Hash};
use crate::primitives::token_standard::{self, TokenStandard};
use crate::utils::bytes::left_pad_bytes;
use core::cmp::Ordering;
use num_bigint::{BigInt, BigUint, Sign};
use std::num::NonZeroU32;

/// Head/fixed size of a single ABI word in bytes.
pub const WORD_SIZE: usize = 32;
/// Head/fixed size of a single ABI word in bytes, as `u32`.
const WORD_SIZE_U32: u32 = 32;

/// A supported ABI type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiType {
    /// `bool`.
    Bool,
    /// Signed integer with bit width.
    Int(u32),
    /// Unsigned integer with bit width.
    UInt(u32),
    /// Zenon address.
    Address,
    /// Zenon token standard.
    TokenStandard,
    /// UTF-8 string.
    String,
    /// Dynamic bytes.
    Bytes,
    /// Fixed-size bytes.
    BytesN(u32),
    /// Zenon hash.
    Hash,
    /// Function pointer payload.
    Function,
    /// Static or dynamic array of an element type.
    Array(Box<AbiType>, Option<NonZeroU32>),
}

/// A single ABI value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiValue {
    /// Boolean value.
    Bool(bool),
    /// Signed integer value.
    Int(BigInt),
    /// Unsigned integer value.
    UInt(BigUint),
    /// Zenon address value.
    Address(Address),
    /// Zenon token standard value.
    TokenStandard(TokenStandard),
    /// UTF-8 string value.
    String(String),
    /// Dynamic bytes value.
    Bytes(Vec<u8>),
    /// Zenon hash value.
    Hash(Hash),
    /// Function pointer payload.
    Function([u8; 24]),
    /// Array value.
    Array(Vec<AbiValue>),
}

impl AbiType {
    /// Parses an ABI type name into its [`AbiType`].
    ///
    /// Supports `bool`, `int`/`intN`, `uint`/`uintN`, `address`,
    /// `tokenStandard`, `string`, `bytes`, `bytesN`, `function`, `hash`, and
    /// the array suffixes `T[]` (dynamic) and `T[N]` (static). Bare `int`/`uint`
    /// resolve to the 256-bit width. Returns [`Error::InvalidInput`] for an
    /// unsupported name.
    pub fn parse(name: &str) -> Result<Self, Error> {
        if let Some(bracket) = name.find('[') {
            return Self::parse_array(name, bracket);
        }
        match name {
            "bool" => return Ok(Self::Bool),
            "address" => return Ok(Self::Address),
            "tokenStandard" => return Ok(Self::TokenStandard),
            "string" => return Ok(Self::String),
            "bytes" => return Ok(Self::Bytes),
            "function" => return Ok(Self::Function),
            "hash" => return Ok(Self::Hash),
            _ => {}
        }
        if let Some(bits) = name.strip_prefix("int") {
            return Ok(Self::Int(parse_bit_width(bits)?));
        }
        if let Some(bits) = name.strip_prefix("uint") {
            return Ok(Self::UInt(parse_bit_width(bits)?));
        }
        if let Some(size) = name.strip_prefix("bytes") {
            let size = size
                .parse::<u32>()
                .map_err(|_| Error::InvalidInput(format!("unsupported ABI type: {name}")))?;
            if size == 0 || size > WORD_SIZE_U32 {
                return Err(Error::InvalidInput(format!(
                    "bytesN width must be 1..=32, got {size}"
                )));
            }
            return Ok(Self::BytesN(size));
        }
        Err(Error::InvalidInput(format!("unsupported ABI type: {name}")))
    }

    /// Parses an array type name, peeling off the first `[...]` suffix and
    /// recursing on the remaining element type.
    fn parse_array(name: &str, bracket: usize) -> Result<Self, Error> {
        let close = name[bracket..]
            .find(']')
            .ok_or_else(|| Error::InvalidInput(format!("unsupported ABI type: {name}")))?;
        let close_abs = bracket + close;
        let dim = &name[bracket + 1..close_abs];

        // The element type is the prefix before the bracket plus anything after
        // the matching close bracket (e.g. `uint8[2][3]` -> `uint8[3]`).
        let prefix = &name[..bracket];
        let suffix = &name[close_abs + 1..];
        let element = Box::new(Self::parse(&format!("{prefix}{suffix}"))?);

        let length = if dim.is_empty() {
            None
        } else {
            let n = dim
                .parse::<u32>()
                .map_err(|_| Error::InvalidInput(format!("unsupported ABI type: {name}")))?;
            Some(NonZeroU32::new(n).ok_or_else(|| {
                Error::InvalidInput(format!("array length must be non-zero: {name}"))
            })?)
        };
        Ok(Self::Array(element, length))
    }

    /// Returns this type's canonical ABI name.
    ///
    /// Bare `int`/`uint` canonicalize as `int256`/`uint256`; arrays append `[]`
    /// or `[N]`.
    pub fn canonical_name(&self) -> String {
        match self {
            Self::Bool => "bool".to_string(),
            Self::Int(bits) => format!("int{bits}"),
            Self::UInt(bits) => format!("uint{bits}"),
            Self::Address => "address".to_string(),
            Self::TokenStandard => "tokenStandard".to_string(),
            Self::String => "string".to_string(),
            Self::Bytes => "bytes".to_string(),
            Self::BytesN(size) => format!("bytes{size}"),
            Self::Hash => "hash".to_string(),
            Self::Function => "function".to_string(),
            Self::Array(element, Some(len)) => {
                format!("{}[{len}]", element.canonical_name())
            }
            Self::Array(element, None) => format!("{}[]", element.canonical_name()),
        }
    }

    /// Encodes a single value of this type into 32-byte-aligned bytes.
    ///
    /// A type/value mismatch (for example a [`AbiValue::String`] given a
    /// [`AbiType::UInt`]) returns [`Error::InvalidInput`].
    pub fn encode(&self, value: &AbiValue) -> Result<Vec<u8>, Error> {
        match (self, value) {
            (Self::Bool, AbiValue::Bool(b)) => {
                Ok(encode_unsigned(&BigUint::from(u32::from(*b)), WORD_SIZE))
            }
            (Self::Int(bits), AbiValue::Int(n)) => {
                ensure_signed_fits(n, *bits)?;
                Ok(encode_signed(n, WORD_SIZE))
            }
            (Self::UInt(bits), AbiValue::UInt(n)) => {
                ensure_unsigned_fits(n, *bits)?;
                Ok(encode_unsigned(n, WORD_SIZE))
            }
            (Self::Address, AbiValue::Address(a)) => Ok(left_pad_bytes(a.core(), WORD_SIZE)),
            (Self::TokenStandard, AbiValue::TokenStandard(ts)) => {
                Ok(left_pad_bytes(ts.core(), WORD_SIZE))
            }
            (Self::String, AbiValue::String(s)) => Ok(encode_length_prefixed(s.as_bytes())),
            (Self::Bytes, AbiValue::Bytes(b)) => Ok(encode_length_prefixed(b)),
            (Self::Hash, AbiValue::Hash(h)) => Ok(h.bytes().to_vec()),
            (Self::Function, AbiValue::Function(payload)) => {
                let mut out = [0u8; WORD_SIZE];
                out[..24].copy_from_slice(payload);
                Ok(out.to_vec())
            }
            (Self::BytesN(size), AbiValue::Bytes(b)) => {
                let width = validate_bytes_n_width(*size)?;
                if b.len() != width {
                    return Err(Error::InvalidInput(format!(
                        "bytes{size} value must be exactly {size} bytes, got {}",
                        b.len()
                    )));
                }
                let mut out = vec![0u8; WORD_SIZE];
                out[..width].copy_from_slice(b);
                Ok(out)
            }
            (Self::Array(element, length), AbiValue::Array(items)) => {
                encode_array(element, *length, items)
            }
            _ => Err(Error::InvalidInput(format!(
                "type/value mismatch: {} cannot encode {}",
                self.canonical_name(),
                value_name(value)
            ))),
        }
    }

    /// Decodes a single value of this type from `encoded` starting at `offset`.
    pub fn decode(&self, encoded: &[u8], offset: usize) -> Result<AbiValue, Error> {
        let end = offset.checked_add(WORD_SIZE).ok_or_else(|| {
            Error::InvalidInput("encoded buffer too short for ABI word".to_string())
        })?;
        if end > encoded.len() {
            return Err(Error::InvalidInput(
                "encoded buffer too short for ABI word".to_string(),
            ));
        }
        match self {
            Self::Bool => Ok(AbiValue::Bool(
                read_word(encoded, offset)? != BigUint::from(0u32),
            )),
            Self::Int(bits) => {
                let value = BigInt::from_signed_bytes_be(&encoded[offset..end]);
                ensure_signed_fits(&value, *bits)?;
                Ok(AbiValue::Int(value))
            }
            Self::UInt(bits) => {
                let value = BigUint::from_bytes_be(&encoded[offset..end]);
                ensure_unsigned_fits(&value, *bits)?;
                Ok(AbiValue::UInt(value))
            }
            Self::Address => {
                let mut core = [0u8; address::CORE_SIZE];
                let core_start = offset + (WORD_SIZE - address::CORE_SIZE);
                core.copy_from_slice(&encoded[core_start..core_start + address::CORE_SIZE]);
                let address = Address::new(address::PREFIX, &core)?;
                Ok(AbiValue::Address(address))
            }
            Self::TokenStandard => {
                let mut core = [0u8; token_standard::CORE_SIZE];
                let core_start = offset + (WORD_SIZE - token_standard::CORE_SIZE);
                core.copy_from_slice(&encoded[core_start..core_start + token_standard::CORE_SIZE]);
                let ts = TokenStandard::from_bytes(&core)?;
                Ok(AbiValue::TokenStandard(ts))
            }
            Self::String => {
                let bytes = decode_length_prefixed(encoded, offset)?;
                let s = String::from_utf8(bytes).map_err(|e| {
                    Error::InvalidInput(format!("ABI string is not valid UTF-8: {e}"))
                })?;
                Ok(AbiValue::String(s))
            }
            Self::Bytes => Ok(AbiValue::Bytes(decode_length_prefixed(encoded, offset)?)),
            Self::Hash => {
                let mut bytes = [0u8; hash::LENGTH];
                bytes.copy_from_slice(&encoded[offset..end]);
                Ok(AbiValue::Hash(Hash::from_bytes(&bytes)?))
            }
            Self::Function => {
                let mut payload = [0u8; 24];
                payload.copy_from_slice(&encoded[offset..offset + 24]);
                Ok(AbiValue::Function(payload))
            }
            Self::BytesN(size) => {
                let width = validate_bytes_n_width(*size)?;
                Ok(AbiValue::Bytes(encoded[offset..offset + width].to_vec()))
            }
            Self::Array(element, length) => decode_array(element, *length, encoded, offset),
        }
    }

    /// Returns the ABI head/fixed size in bytes.
    ///
    /// Word types and dynamic head slots occupy 32 bytes; a static array of
    /// static elements is its element size times its length.
    pub fn fixed_size(&self) -> u32 {
        match self {
            Self::Array(element, Some(_)) if element.is_dynamic() => WORD_SIZE_U32,
            Self::Array(element, Some(len)) => element.fixed_size() * len.get(),
            _ => WORD_SIZE_U32,
        }
    }

    /// Returns whether this type is dynamic.
    ///
    /// `string`, `bytes`, dynamic arrays (`T[]`), and fixed arrays whose
    /// element type is dynamic are dynamic.
    pub fn is_dynamic(&self) -> bool {
        match self {
            Self::String | Self::Bytes | Self::Array(_, None) => true,
            Self::Array(element, Some(_)) => element.is_dynamic(),
            _ => false,
        }
    }
}

/// Parses a bare `int`/`uint` bit width, defaulting to 256 for an empty suffix.
fn parse_bit_width(bits: &str) -> Result<u32, Error> {
    if bits.is_empty() {
        return Ok(256);
    }
    let width = bits
        .parse::<u32>()
        .map_err(|_| Error::InvalidInput(format!("unsupported integer width: {bits}")))?;
    validate_integer_width(width)?;
    Ok(width)
}

/// Returns a human-readable name for `value`, used in mismatch errors.
fn value_name(value: &AbiValue) -> &'static str {
    match value {
        AbiValue::Bool(_) => "bool",
        AbiValue::Int(_) => "int",
        AbiValue::UInt(_) => "uint",
        AbiValue::Address(_) => "address",
        AbiValue::TokenStandard(_) => "tokenStandard",
        AbiValue::String(_) => "string",
        AbiValue::Bytes(_) => "bytes",
        AbiValue::Hash(_) => "hash",
        AbiValue::Function(_) => "function",
        AbiValue::Array(_) => "array",
    }
}

/// Validates a Solidity-style integer bit width.
fn validate_integer_width(bits: u32) -> Result<(), Error> {
    if (8..=256).contains(&bits) && bits.is_multiple_of(8) {
        Ok(())
    } else {
        Err(Error::InvalidInput(format!(
            "integer width must be a multiple of 8 in 8..=256, got {bits}"
        )))
    }
}

/// Ensures an unsigned value fits the declared integer width.
fn ensure_unsigned_fits(value: &BigUint, bits: u32) -> Result<(), Error> {
    validate_integer_width(bits)?;
    if value.bits() <= u64::from(bits) {
        Ok(())
    } else {
        Err(Error::InvalidInput(format!(
            "uint{bits} value does not fit in {bits} bits"
        )))
    }
}

/// Ensures a signed value fits the declared integer width.
fn ensure_signed_fits(value: &BigInt, bits: u32) -> Result<(), Error> {
    validate_integer_width(bits)?;
    let shift = usize::try_from(bits - 1)
        .map_err(|_| Error::InvalidInput(format!("integer width overflows usize: {bits}")))?;
    let magnitude = BigInt::from(1u8) << shift;
    let min = -magnitude.clone();
    let max = magnitude - BigInt::from(1u8);
    if value >= &min && value <= &max {
        Ok(())
    } else {
        Err(Error::InvalidInput(format!(
            "int{bits} value does not fit in {bits} bits"
        )))
    }
}

/// Validates a `bytesN` width and returns it as `usize`.
fn validate_bytes_n_width(size: u32) -> Result<usize, Error> {
    if (1..=WORD_SIZE_U32).contains(&size) {
        usize::try_from(size)
            .map_err(|_| Error::InvalidInput(format!("bytesN width overflows usize: {size}")))
    } else {
        Err(Error::InvalidInput(format!(
            "bytesN width must be 1..=32, got {size}"
        )))
    }
}

/// Encodes `value` as an unsigned big-endian integer in exactly `size` bytes,
/// left-padded with zeros (or truncated to the low `size` bytes if too large).
fn encode_unsigned(value: &BigUint, size: usize) -> Vec<u8> {
    let bytes = value.to_bytes_be();
    pad_or_truncate(&bytes, size, 0x00)
}

/// Encodes `value` as a signed two's-complement big-endian integer in exactly
/// `size` bytes, sign-extended with `0xff` when negative.
fn encode_signed(value: &BigInt, size: usize) -> Vec<u8> {
    let bytes = value.to_signed_bytes_be();
    let pad = if value.sign() == Sign::Minus {
        0xff
    } else {
        0x00
    };
    pad_or_truncate(&bytes, size, pad)
}

/// Left-pad or truncate `bytes` to exactly `size` bytes using `pad` for padding.
fn pad_or_truncate(bytes: &[u8], size: usize, pad: u8) -> Vec<u8> {
    match bytes.len().cmp(&size) {
        Ordering::Less => {
            let mut out = vec![pad; size - bytes.len()];
            out.extend_from_slice(bytes);
            out
        }
        Ordering::Greater => bytes[bytes.len() - size..].to_vec(),
        Ordering::Equal => bytes.to_vec(),
    }
}

/// Encodes a dynamic byte region as a 32-byte big-endian length prefix followed
/// by the data zero-padded to a multiple of 32 bytes.
fn encode_length_prefixed(data: &[u8]) -> Vec<u8> {
    let mut out = encode_unsigned(&BigUint::from(data.len()), WORD_SIZE);
    out.extend_from_slice(data);
    let rem = out.len() % WORD_SIZE;
    if rem != 0 {
        out.extend(std::iter::repeat_n(0u8, WORD_SIZE - rem));
    }
    out
}

/// Decodes a length-prefixed dynamic region beginning at `offset`, returning the
/// unpadded payload.
fn decode_length_prefixed(encoded: &[u8], offset: usize) -> Result<Vec<u8>, Error> {
    let length: usize = read_word(encoded, offset)?
        .try_into()
        .map_err(|_| Error::InvalidInput("ABI length overflows usize".to_string()))?;
    let start = offset + WORD_SIZE;
    let end = start
        .checked_add(length)
        .ok_or_else(|| Error::InvalidInput("ABI length overflows buffer".to_string()))?;
    if end > encoded.len() {
        return Err(Error::InvalidInput(
            "encoded buffer too short for ABI length".to_string(),
        ));
    }
    Ok(encoded[start..end].to_vec())
}

/// Reads a 32-byte big-endian word at `offset` as an unsigned [`BigUint`].
fn read_word(encoded: &[u8], offset: usize) -> Result<BigUint, Error> {
    let end = offset
        .checked_add(WORD_SIZE)
        .ok_or_else(|| Error::InvalidInput("encoded buffer too short for ABI word".to_string()))?;
    if end > encoded.len() {
        return Err(Error::InvalidInput(
            "encoded buffer too short for ABI word".to_string(),
        ));
    }
    Ok(BigUint::from_bytes_be(&encoded[offset..end]))
}

/// Encodes an array value, prefixing dynamic arrays with their element count.
fn encode_array(
    element: &AbiType,
    length: Option<NonZeroU32>,
    items: &[AbiValue],
) -> Result<Vec<u8>, Error> {
    if let Some(expected) = length
        && expected.get() as usize != items.len()
    {
        return Err(Error::InvalidInput(format!(
            "static array of length {} got {} values",
            expected,
            items.len()
        )));
    }
    let tuple = encode_tuple(element, items)?;
    if length.is_some() {
        return Ok(tuple);
    }
    let mut out = encode_unsigned(&BigUint::from(items.len()), WORD_SIZE);
    out.extend(tuple);
    Ok(out)
}

/// Encodes array elements using a head+tail layout when the element type is
/// dynamic, otherwise inline.
fn encode_tuple(element: &AbiType, items: &[AbiValue]) -> Result<Vec<u8>, Error> {
    if element.is_dynamic() {
        let mut head: Vec<Vec<u8>> = (0..items.len()).map(|_| Vec::new()).collect();
        let mut tail: Vec<Vec<u8>> = (0..items.len()).map(|_| Vec::new()).collect();
        let mut offset = items.len() * WORD_SIZE;
        for (i, value) in items.iter().enumerate() {
            let encoded = element.encode(value)?;
            let encoded_len = encoded.len();
            head[i] = encode_unsigned(&BigUint::from(offset), WORD_SIZE);
            tail[i] = encoded;
            offset += encoded_len;
        }
        let mut out = Vec::with_capacity(offset);
        for part in &head {
            out.extend_from_slice(part);
        }
        for part in &tail {
            out.extend_from_slice(part);
        }
        Ok(out)
    } else {
        let mut out = Vec::with_capacity(items.len() * element.fixed_size() as usize);
        for value in items {
            out.extend_from_slice(&element.encode(value)?);
        }
        Ok(out)
    }
}

/// Decodes an array value at `offset`, reading the element count for dynamic
/// arrays.
fn decode_array(
    element: &AbiType,
    length: Option<NonZeroU32>,
    encoded: &[u8],
    offset: usize,
) -> Result<AbiValue, Error> {
    let (count, body_offset) = if let Some(length) = length {
        (length.get() as usize, offset)
    } else {
        let count: usize = read_word(encoded, offset)?
            .try_into()
            .map_err(|_| Error::InvalidInput("ABI array length overflows usize".to_string()))?;
        (count, offset + WORD_SIZE)
    };

    let mut values = Vec::with_capacity(count);
    let stride = element.fixed_size() as usize;
    if element.is_dynamic() {
        for i in 0..count {
            let head = body_offset + i * WORD_SIZE;
            let relative: usize = read_word(encoded, head)?
                .try_into()
                .map_err(|_| Error::InvalidInput("ABI array offset overflows usize".to_string()))?;
            values.push(element.decode(encoded, body_offset + relative)?);
        }
    } else {
        for i in 0..count {
            values.push(element.decode(encoded, body_offset + i * stride)?);
        }
    }
    Ok(AbiValue::Array(values))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn bare_int_and_uint_canonicalize_to_256() {
        assert_eq!(AbiType::parse("int").unwrap().canonical_name(), "int256");
        assert_eq!(AbiType::parse("uint").unwrap().canonical_name(), "uint256");
    }

    #[test]
    fn unsupported_type_is_rejected() {
        let err = AbiType::parse("widget").unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn dynamic_and_static_arrays_report_metadata() {
        assert!(AbiType::parse("uint8[]").unwrap().is_dynamic());
        assert!(!AbiType::parse("uint8[2]").unwrap().is_dynamic());
    }

    #[test]
    fn negative_int_round_trips_twos_complement() {
        let ty = AbiType::Int(64);
        let value = AbiValue::Int(BigInt::from(-1));
        let encoded = ty.encode(&value).unwrap();
        assert_eq!(encoded, vec![0xff; WORD_SIZE]);
        assert_eq!(ty.decode(&encoded, 0).unwrap(), value);
    }

    #[test]
    fn uint_rejects_a_signed_value_variant() {
        let result = AbiType::UInt(8).encode(&AbiValue::Int(BigInt::from(1)));
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn bytes_are_length_prefixed_and_padded() {
        let ty = AbiType::Bytes;
        let encoded = ty.encode(&AbiValue::Bytes(b"hello".to_vec())).unwrap();
        assert_eq!(encoded.len(), 64);
        assert_eq!(read_word(&encoded, 0).unwrap(), BigUint::from(5u32));
        assert_eq!(&encoded[32..37], b"hello");
        assert!(ty.is_dynamic());
    }
}

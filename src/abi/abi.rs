//! ABI signature, selector, and tuple/function encoding and decoding.
//!
//! Function selectors are the first 4 bytes of the SHA3-256 digest of the
//! canonical signature. Arguments use a head+tail layout: static arguments sit
//! inline in the head while dynamic arguments are represented by an offset
//! pointer in the head and their encoding in the tail.

#![allow(clippy::indexing_slicing)]

use crate::abi::abi_types::{AbiType, AbiValue, WORD_SIZE};
use crate::crypto::crypto;
use crate::error::Error;
use serde_json::Value;

/// Formats a function signature from its name and canonical input types as
/// `name(type1,type2,...)`.
pub fn format_signature(name: &str, types: &[AbiType]) -> String {
    let body = types
        .iter()
        .map(AbiType::canonical_name)
        .collect::<Vec<_>>()
        .join(",");
    format!("{name}({body})")
}

/// Returns the 4-byte selector: the first 4 bytes of the SHA3-256 digest of the
/// canonical signature.
pub fn selector(name: &str, types: &[AbiType]) -> [u8; 4] {
    let signature = format_signature(name, types);
    let digest = crypto::digest(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

/// ABI-encodes a tuple of arguments using a head+tail layout.
///
/// Returns [`Error::InvalidInput`] when the value count does not equal the type
/// count, or when any value's type does not match its declared [`AbiType`].
pub fn encode_arguments(types: &[AbiType], values: &[AbiValue]) -> Result<Vec<u8>, Error> {
    if types.len() != values.len() {
        return Err(Error::InvalidInput(format!(
            "expected {} ABI arguments, got {}",
            types.len(),
            values.len()
        )));
    }

    let static_size: usize = types.iter().map(|t| t.fixed_size() as usize).sum();
    let dynamic_count = types.iter().filter(|t| t.is_dynamic()).count();

    let mut parts: Vec<Vec<u8>> = vec![Vec::new(); types.len() + dynamic_count];
    let mut dynamic_ptr = static_size;
    let mut dynamic_idx = 0usize;
    for (i, (ty, value)) in types.iter().zip(values).enumerate() {
        if ty.is_dynamic() {
            let encoded = ty.encode(value)?;
            parts[i] = encode_offset(dynamic_ptr);
            let tail_slot = types.len() + dynamic_idx;
            dynamic_ptr += encoded.len();
            parts[tail_slot] = encoded;
            dynamic_idx += 1;
        } else {
            parts[i] = ty.encode(value)?;
        }
    }

    let capacity = parts.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(capacity);
    for part in &parts {
        out.extend_from_slice(part);
    }
    Ok(out)
}

/// ABI-decodes a tuple of arguments, reversing the head+tail layout produced by
/// [`encode_arguments`].
pub fn decode_arguments(types: &[AbiType], bytes: &[u8]) -> Result<Vec<AbiValue>, Error> {
    let mut offset = 0usize;
    let mut result = Vec::with_capacity(types.len());
    for ty in types {
        let value = if ty.is_dynamic() {
            let pointer = read_offset(bytes, offset)?;
            ty.decode(bytes, pointer)?
        } else {
            ty.decode(bytes, offset)?
        };
        result.push(value);
        offset += ty.fixed_size() as usize;
    }
    Ok(result)
}

/// Encodes `value` as a 32-byte big-endian unsigned word (an offset pointer).
fn encode_offset(value: usize) -> Vec<u8> {
    let mut out = vec![0u8; WORD_SIZE];
    let bytes = (value as u128).to_be_bytes();
    out[WORD_SIZE - bytes.len()..].copy_from_slice(&bytes);
    out
}

/// Reads a 32-byte big-endian unsigned offset pointer at `offset`.
fn read_offset(bytes: &[u8], offset: usize) -> Result<usize, Error> {
    let end = offset.checked_add(WORD_SIZE).ok_or_else(|| {
        Error::InvalidInput("encoded buffer too short for ABI offset".to_string())
    })?;
    if end > bytes.len() {
        return Err(Error::InvalidInput(
            "encoded buffer too short for ABI offset".to_string(),
        ));
    }
    let mut value = 0usize;
    for &b in &bytes[offset..end] {
        value = value
            .checked_mul(256)
            .and_then(|v| v.checked_add(b as usize))
            .ok_or_else(|| Error::InvalidInput("ABI offset overflows usize".to_string()))?;
    }
    Ok(value)
}

/// A parsed ABI with named function entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Abi {
    functions: Vec<(String, Vec<AbiType>)>,
}

impl Abi {
    /// Creates an ABI from `(function_name, input_types)` pairs.
    pub fn new(functions: Vec<(String, Vec<AbiType>)>) -> Self {
        Self { functions }
    }

    /// Parses a JSON ABI array of `{type,name,inputs:[{name,type}]}` entries.
    ///
    /// Only `function` entries are accepted; any other entry type returns
    /// [`Error::InvalidInput`].
    pub fn from_json(json: &Value) -> Result<Self, Error> {
        let array = json.as_array().ok_or_else(|| {
            Error::InvalidInput("ABI JSON must be an array of entries".to_string())
        })?;
        let mut functions = Vec::with_capacity(array.len());
        for entry in array {
            let obj = entry.as_object().ok_or_else(|| {
                Error::InvalidInput("ABI entry must be a JSON object".to_string())
            })?;
            let entry_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::InvalidInput("ABI entry missing \"type\"".to_string()))?;
            if entry_type != "function" {
                return Err(Error::InvalidInput(format!(
                    "only ABI functions are supported, got \"{entry_type}\""
                )));
            }
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::InvalidInput("ABI function missing \"name\"".to_string()))?;
            let mut input_types = Vec::new();
            if let Some(inputs) = obj.get("inputs").and_then(|v| v.as_array()) {
                for input in inputs {
                    let input_obj = input.as_object().ok_or_else(|| {
                        Error::InvalidInput("ABI input must be a JSON object".to_string())
                    })?;
                    let type_str =
                        input_obj
                            .get("type")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                Error::InvalidInput("ABI input missing \"type\"".to_string())
                            })?;
                    input_types.push(AbiType::parse(type_str)?);
                }
            }
            functions.push((name.to_string(), input_types));
        }
        Ok(Self::new(functions))
    }

    /// Encodes `name(values...)` as the 4-byte selector followed by the encoded
    /// arguments. Returns [`Error::InvalidInput`] when `name` is unknown or the
    /// argument count does not match the entry's input count.
    pub fn encode_function(&self, name: &str, values: &[AbiValue]) -> Result<Vec<u8>, Error> {
        let types = self
            .functions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t)
            .ok_or_else(|| Error::InvalidInput(format!("unknown ABI function: {name}")))?;
        let mut out = selector(name, types).to_vec();
        out.extend(encode_arguments(types, values)?);
        Ok(out)
    }

    /// Matches an encoded selector to an entry and decodes the trailing
    /// arguments. Returns [`Error::InvalidInput`] when no entry matches.
    pub fn decode_function(&self, encoded: &[u8]) -> Result<Vec<AbiValue>, Error> {
        let prefix = encoded.get(..4).ok_or_else(|| {
            Error::InvalidInput("encoded function is shorter than the selector".to_string())
        })?;
        for (name, types) in &self.functions {
            if selector(name, types).as_slice() == prefix {
                return decode_arguments(types, &encoded[4..]);
            }
        }
        Err(Error::InvalidInput(
            "no ABI function matches the encoded selector".to_string(),
        ))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn signature_joins_canonical_names() {
        let types = [
            AbiType::parse("string").unwrap(),
            AbiType::parse("address").unwrap(),
        ];
        assert_eq!(
            format_signature("Register", &types),
            "Register(string,address)"
        );
    }

    #[test]
    fn selector_is_first_four_bytes_of_sha3() {
        let types = [AbiType::parse("string").unwrap()];
        let digest = crypto::digest(b"Register(string)");
        let expected = [digest[0], digest[1], digest[2], digest[3]];
        assert_eq!(selector("Register", &types), expected);
    }

    #[test]
    fn mismatched_value_count_is_rejected() {
        let result = encode_arguments(
            &[AbiType::UInt(256), AbiType::String],
            &[AbiValue::UInt(BigUint::from(1u32))],
        );
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn empty_signature_has_no_body() {
        assert_eq!(format_signature("Foo", &[]), "Foo()");
    }
}

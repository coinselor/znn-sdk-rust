//! Canonical JSON-access helpers for model `from_json` constructors.
//!
//! These helpers are pure functions over [`serde_json::Value`] /
//! [`serde_json::Map`] returning `Result<_, Error>` with `Error::InvalidInput`
//! on the sad path. They hold no per-model state, so every model file shares
//! this single definition instead of re-declaring its own private copy. The
//! module is unconditional so it compiles in the reduced-core build
//! (`--no-default-features`).

use crate::error::Error;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use num_bigint::BigUint;
use serde_json::{Map, Value};

/// Returns the object inside `value`, or an error when `value` is not an object.
pub(crate) fn json_object<'a>(
    value: &'a Value,
    name: &str,
) -> Result<&'a Map<String, Value>, Error> {
    value
        .as_object()
        .ok_or_else(|| Error::InvalidInput(format!("{name} must be a JSON object")))
}

/// Returns the value of a required field, or an error when it is absent.
pub(crate) fn required_value<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Value, Error> {
    object
        .get(field)
        .ok_or_else(|| Error::InvalidInput(format!("missing {field} field")))
}

/// Returns a required field as `&str`, or an error when absent or mistyped.
pub(crate) fn required_str<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, Error> {
    required_value(object, field)?
        .as_str()
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be a string")))
}

/// Returns a required field as `u64`, or an error when absent or mistyped.
pub(crate) fn required_u64(object: &Map<String, Value>, field: &str) -> Result<u64, Error> {
    required_value(object, field)?
        .as_u64()
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be an unsigned integer")))
}

/// Returns a required field as `u32`, or an error when absent, mistyped, or
/// out of the `u32` range.
pub(crate) fn required_u32(object: &Map<String, Value>, field: &str) -> Result<u32, Error> {
    let value = required_u64(object, field)?;
    u32::try_from(value).map_err(|_| Error::InvalidInput(format!("{field} is out of range")))
}

/// Returns a required field as `u8`, or an error when absent, mistyped, or out
/// of the `u8` range.
pub(crate) fn required_u8(object: &Map<String, Value>, field: &str) -> Result<u8, Error> {
    let value = required_u64(object, field)?;
    u8::try_from(value).map_err(|_| Error::InvalidInput(format!("{field} is out of range")))
}

/// Returns a required field as a boolean, or an error when absent or mistyped.
pub(crate) fn required_bool(object: &Map<String, Value>, field: &str) -> Result<bool, Error> {
    required_value(object, field)?
        .as_bool()
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be a boolean")))
}

/// Returns a required field parsed as a decimal string into a [`BigUint`], or
/// an error when absent, mistyped, or non-decimal.
pub(crate) fn required_big_uint(
    object: &Map<String, Value>,
    field: &str,
) -> Result<BigUint, Error> {
    let value = required_str(object, field)?;
    BigUint::parse_bytes(value.as_bytes(), 10)
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be a decimal string")))
}

/// Decodes each element of a required array field via `map`, returning the
/// collected `Vec<T>`. Errors when the field is absent, not an array, or `map`
/// fails for an element.
pub(crate) fn required_array<T, F>(
    object: &Map<String, Value>,
    field: &str,
    map: F,
) -> Result<Vec<T>, Error>
where
    F: Fn(&Value) -> Result<T, Error>,
{
    match object.get(field) {
        Some(Value::Array(values)) => values.iter().map(map).collect(),
        Some(_) => Err(Error::InvalidInput(format!("{field} must be an array"))),
        None => Err(Error::InvalidInput(format!("missing {field} field"))),
    }
}

/// Returns a required array field as a borrowed slice, or an error when the
/// field is absent or not an array.
pub(crate) fn required_array_ref<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a [Value], Error> {
    required_value(object, field)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| Error::InvalidInput(format!("{field} must be an array")))
}

/// Returns an optional unsigned integer field. `None` when the field is absent.
pub(crate) fn optional_u64(object: &Map<String, Value>, field: &str) -> Result<Option<u64>, Error> {
    object
        .get(field)
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| Error::InvalidInput(format!("{field} must be an unsigned integer")))
        })
        .transpose()
}

/// Returns an optional boolean field. `None` when the field is absent.
pub(crate) fn optional_bool(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<bool>, Error> {
    object
        .get(field)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| Error::InvalidInput(format!("{field} must be a boolean")))
        })
        .transpose()
}

/// Returns an optional hash field. `None` when the field is absent or null.
pub(crate) fn optional_hash(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<Hash>, Error> {
    match object.get(field) {
        Some(Value::Null) | None => Ok(None),
        Some(Value::String(value)) => Hash::parse(value).map(Some),
        Some(_) => Err(Error::InvalidInput(format!("{field} must be a string"))),
    }
}

/// Returns an optional address field. `None` when the field is absent or null.
pub(crate) fn optional_address(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<Address>, Error> {
    match object.get(field) {
        Some(Value::String(s)) => Address::parse(s).map(Some),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(Error::InvalidInput(format!(
            "{field} must be a string or null"
        ))),
    }
}

/// Decodes an optional standard-base64 field. An empty vector when the field is
/// absent or null.
pub(crate) fn optional_base64(object: &Map<String, Value>, field: &str) -> Result<Vec<u8>, Error> {
    match object.get(field) {
        Some(Value::String(s)) => STANDARD
            .decode(s)
            .map_err(|_| Error::InvalidInput(format!("{field} must be valid base64"))),
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(_) => Err(Error::InvalidInput(format!("{field} must be a string"))),
    }
}

/// Decodes a required standard-base64 field, or an error when absent, mistyped,
/// or not valid base64.
pub(crate) fn required_base64(object: &Map<String, Value>, field: &str) -> Result<Vec<u8>, Error> {
    let value = required_str(object, field)?;
    STANDARD
        .decode(value)
        .map_err(|e| Error::InvalidInput(format!("{field} must be standard base64: {e}")))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn object_with(field: &str, value: Value) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(field.to_string(), value);
        map
    }

    #[test]
    fn json_object_returns_the_inner_object() {
        let value = json!({"a": 1});
        let object = json_object(&value, "token").expect("object is a JSON object");
        assert_eq!(object.get("a"), Some(&json!(1)));
    }

    #[test]
    fn json_object_rejects_a_non_object_with_pinned_wording() {
        let value = json!("not-an-object");
        let result = json_object(&value, "token");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "token must be a JSON object"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_value_returns_the_field() {
        let object = object_with("a", json!(1));
        assert_eq!(required_value(&object, "a"), Ok(&json!(1)));
    }

    #[test]
    fn required_value_rejects_a_missing_field_with_pinned_wording() {
        let object = Map::new();
        let result = required_value(&object, "a");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "missing a field"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_str_returns_the_string() {
        let object = object_with("name", json!("zenon"));
        assert_eq!(required_str(&object, "name"), Ok("zenon"));
    }

    #[test]
    fn required_str_rejects_a_missing_field_with_pinned_wording() {
        let object = Map::new();
        let result = required_str(&object, "name");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "missing name field"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_str_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("name", json!(7));
        let result = required_str(&object, "name");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "name must be a string"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_u64_returns_the_integer() {
        let object = object_with("n", json!(42u64));
        assert_eq!(required_u64(&object, "n"), Ok(42));
    }

    #[test]
    fn required_u64_rejects_a_missing_field_with_pinned_wording() {
        let object = Map::new();
        let result = required_u64(&object, "n");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "missing n field"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_u64_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("n", json!("seven"));
        let result = required_u64(&object, "n");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "n must be an unsigned integer"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_u32_returns_the_integer() {
        let object = object_with("n", json!(7u64));
        assert_eq!(required_u32(&object, "n"), Ok(7));
    }

    #[test]
    fn required_u32_rejects_an_out_of_range_value_with_pinned_wording() {
        let object = object_with("n", json!(u64::from(u32::MAX) + 1));
        let result = required_u32(&object, "n");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "n is out of range"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_array_maps_each_element() {
        let object = object_with("list", json!([1, 2, 3]));
        let result = required_array(&object, "list", |v| {
            v.as_u64().ok_or_else(|| Error::InvalidInput("bad".into()))
        });
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[test]
    fn required_array_propagates_the_first_element_error() {
        let object = object_with("list", json!([1, "x", 3]));
        let result = required_array(&object, "list", |v| {
            v.as_u64()
                .ok_or_else(|| Error::InvalidInput("not a u64".into()))
        });
        assert!(matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "not a u64"));
    }

    #[test]
    fn required_array_rejects_a_missing_field_with_pinned_wording() {
        let object = Map::new();
        let result = required_array(&object, "list", |_| Ok(0u64));
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "missing list field"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_array_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("list", json!(7));
        let result = required_array(&object, "list", |_| Ok(0u64));
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "list must be an array"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_array_ref_returns_the_borrowed_slice() {
        let object = object_with("list", json!([1, 2, 3]));
        let slice = required_array_ref(&object, "list").expect("array present");
        assert_eq!(slice, &[json!(1), json!(2), json!(3)]);
    }

    #[test]
    fn required_array_ref_rejects_a_missing_field_with_pinned_wording() {
        let object = Map::new();
        let result = required_array_ref(&object, "list");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "missing list field"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_array_ref_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("list", json!(7));
        let result = required_array_ref(&object, "list");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "list must be an array"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn optional_u64_returns_none_when_absent() {
        let object = Map::new();
        assert_eq!(optional_u64(&object, "n").unwrap(), None);
    }

    #[test]
    fn optional_u64_returns_the_integer_when_present() {
        let object = object_with("n", json!(9u64));
        assert_eq!(optional_u64(&object, "n").unwrap(), Some(9));
    }

    #[test]
    fn optional_u64_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("n", json!("nine"));
        let result = optional_u64(&object, "n");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "n must be an unsigned integer"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn optional_bool_returns_none_when_absent() {
        let object = Map::new();
        assert_eq!(optional_bool(&object, "flag").unwrap(), None);
    }

    #[test]
    fn optional_bool_returns_the_bool_when_present() {
        let object = object_with("flag", json!(true));
        assert_eq!(optional_bool(&object, "flag").unwrap(), Some(true));
    }

    #[test]
    fn optional_bool_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("flag", json!("yes"));
        let result = optional_bool(&object, "flag");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "flag must be a boolean"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn optional_hash_returns_none_when_absent_or_null() {
        let object = Map::new();
        assert_eq!(optional_hash(&object, "h").unwrap(), None);

        let object = object_with("h", Value::Null);
        assert_eq!(optional_hash(&object, "h").unwrap(), None);
    }

    #[test]
    fn optional_hash_parses_a_present_hash_string() {
        let hex = "5454a3b64225d7b7e5b25b8b7670b8a4f5f9c7f0a9b8c7d6e5f4a3b2c1d0e0f0";
        let object = object_with("h", json!(hex));
        let parsed = optional_hash(&object, "h").expect("hash parses");
        assert_eq!(parsed, Some(Hash::parse(hex).expect("hex is a valid hash")));
    }

    #[test]
    fn optional_hash_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("h", json!(7));
        let result = optional_hash(&object, "h");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "h must be a string"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn optional_address_returns_none_when_absent_or_null() {
        let object = Map::new();
        assert_eq!(optional_address(&object, "a").unwrap(), None);

        let object = object_with("a", Value::Null);
        assert_eq!(optional_address(&object, "a").unwrap(), None);
    }

    #[test]
    fn optional_address_parses_a_present_address_string() {
        let s = "z1qxemdeddedxpyllarxxxxxxxxxxxxxxxsy3fmg";
        let object = object_with("a", json!(s));
        let parsed = optional_address(&object, "a").expect("address parses");
        assert_eq!(
            parsed,
            Some(Address::parse(s).expect("string is a valid address"))
        );
    }

    #[test]
    fn optional_address_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("a", json!(7));
        let result = optional_address(&object, "a");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "a must be a string or null"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn optional_base64_returns_empty_when_absent_or_null() {
        let object = Map::new();
        assert_eq!(optional_base64(&object, "data").unwrap(), Vec::<u8>::new());

        let object = object_with("data", Value::Null);
        assert_eq!(optional_base64(&object, "data").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn optional_base64_decodes_a_present_string() {
        let object = object_with("data", json!("aGVsbG8=")); // "hello"
        assert_eq!(optional_base64(&object, "data").unwrap(), b"hello");
    }

    #[test]
    fn optional_base64_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("data", json!(7));
        let result = optional_base64(&object, "data");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "data must be a string"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_u8_returns_the_integer() {
        let object = object_with("n", json!(7u64));
        assert_eq!(required_u8(&object, "n"), Ok(7));
    }

    #[test]
    fn required_u8_rejects_an_out_of_range_value_with_pinned_wording() {
        let object = object_with("n", json!(u64::from(u8::MAX) + 1));
        let result = required_u8(&object, "n");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "n is out of range"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_u8_rejects_a_missing_field_with_pinned_wording() {
        let object = Map::new();
        let result = required_u8(&object, "n");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "missing n field"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_bool_returns_the_bool() {
        let object = object_with("flag", json!(true));
        assert_eq!(required_bool(&object, "flag"), Ok(true));
    }

    #[test]
    fn required_bool_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("flag", json!("yes"));
        let result = required_bool(&object, "flag");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "flag must be a boolean"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_big_uint_returns_the_decimal_value() {
        let object = object_with("n", json!("123456789"));
        assert_eq!(
            required_big_uint(&object, "n"),
            Ok(BigUint::from(123_456_789u64))
        );
    }

    #[test]
    fn required_big_uint_rejects_a_non_decimal_string_with_pinned_wording() {
        let object = object_with("n", json!("not-a-number"));
        let result = required_big_uint(&object, "n");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "n must be a decimal string"),
            "wrong wording: {result:?}"
        );
    }

    #[test]
    fn required_base64_decodes_a_present_string() {
        let object = object_with("data", json!("aGVsbG8=")); // "hello"
        assert_eq!(required_base64(&object, "data"), Ok(b"hello".to_vec()));
    }

    #[test]
    fn required_base64_rejects_a_mistyped_field_with_pinned_wording() {
        let object = object_with("data", json!(7));
        let result = required_base64(&object, "data");
        assert!(
            matches!(result, Err(Error::InvalidInput(ref msg)) if msg == "data must be a string"),
            "wrong wording: {result:?}"
        );
    }
}

//! Tests for ABI type parsing, metadata, and single-value coding.
#![allow(clippy::expect_used)]

use num_bigint::{BigInt, BigUint};
use serde::Deserialize;
use znn_sdk_rust::abi::{AbiType, AbiValue};
use znn_sdk_rust::error::Error;

const TYPE_VECTORS: &str = include_str!("vectors/abi/types.json");

#[derive(Deserialize)]
struct TypeVectors {
    canonical: Vec<CanonicalCase>,
    arrays: Vec<ArrayCase>,
    #[serde(rename = "negativeInt64")]
    negative_int64: NegativeIntCase,
    bytes: BytesCase,
    #[serde(rename = "fixedArray")]
    fixed_array: FixedArrayCase,
}

#[derive(Deserialize)]
struct CanonicalCase {
    name: String,
    canonical: String,
}

#[derive(Deserialize)]
struct ArrayCase {
    name: String,
    dynamic: bool,
    length: Option<u32>,
}

#[derive(Deserialize)]
struct NegativeIntCase {
    value: i64,
    #[serde(rename = "encodedHex")]
    encoded_hex: String,
}

#[derive(Deserialize)]
struct BytesCase {
    #[serde(rename = "valueHex")]
    value_hex: String,
    #[serde(rename = "encodedHex")]
    encoded_hex: String,
}

#[derive(Deserialize)]
struct FixedArrayCase {
    name: String,
    #[serde(rename = "fixedSize")]
    fixed_size: u32,
}

fn vectors() -> TypeVectors {
    serde_json::from_str(TYPE_VECTORS).expect("ABI type vectors parse")
}

#[test]
fn canonical_names_resolve() {
    for case in vectors().canonical {
        let ty = AbiType::parse(&case.name).expect("supported ABI type parses");
        assert_eq!(
            ty.canonical_name(),
            case.canonical,
            "canonical name for {}",
            case.name
        );
    }
}

#[test]
fn arrays_distinguish_static_and_dynamic() {
    for case in vectors().arrays {
        let ty = AbiType::parse(&case.name).expect("array type parses");
        let length_matches = match (&ty, case.length) {
            (AbiType::Array(_, None), None) => true,
            (AbiType::Array(_, Some(len)), Some(expected)) if len.get() == expected => true,
            _ => false,
        };
        assert!(
            length_matches,
            "{} must parse with length {:?}, got {ty:?}",
            case.name, case.length
        );
        assert_eq!(
            ty.is_dynamic(),
            case.dynamic,
            "dynamic metadata for {}",
            case.name
        );
    }
}

#[test]
fn negative_int_encodes_twos_complement_and_round_trips() {
    let case = vectors().negative_int64;
    let ty = AbiType::Int(64);
    let value = AbiValue::Int(BigInt::from(case.value));

    let encoded = ty.encode(&value).expect("int encodes");

    let expected = const_hex::decode(case.encoded_hex).expect("hex decodes");
    assert_eq!(
        encoded, expected,
        "negative int must encode in two's complement"
    );
    assert_eq!(ty.decode(&encoded, 0), Ok(value));
}

#[test]
fn bytes_are_length_prefixed_and_dynamic() {
    let case = vectors().bytes;
    let ty = AbiType::Bytes;
    let value = const_hex::decode(case.value_hex).expect("bytes value hex decodes");
    let encoded = ty.encode(&AbiValue::Bytes(value)).expect("bytes encode");

    let expected = const_hex::decode(case.encoded_hex).expect("encoded bytes hex decodes");
    assert_eq!(encoded, expected);
    assert!(ty.is_dynamic(), "bytes is a dynamic ABI type");
}

#[test]
fn type_value_mismatch_is_rejected() {
    let result = AbiType::UInt(256).encode(&AbiValue::String("x".into()));

    assert!(
        matches!(result, Err(Error::InvalidInput(_))),
        "type/value mismatch must be Error::InvalidInput, got {result:?}"
    );
}

#[test]
fn bytes_n_requires_declared_width_and_round_trips() {
    let ty = AbiType::parse("bytes4").expect("bytes4 parses");
    let value = AbiValue::Bytes(vec![1, 2, 3, 4]);
    let encoded = ty.encode(&value).expect("bytes4 encodes");

    assert_eq!(encoded.len(), 32);
    assert_eq!(encoded.get(..4), Some(&[1, 2, 3, 4][..]));
    assert_eq!(encoded.get(4..), Some(&[0u8; 28][..]));
    assert_eq!(ty.decode(&encoded, 0), Ok(value));

    let too_short = ty.encode(&AbiValue::Bytes(vec![1, 2, 3]));
    assert!(matches!(too_short, Err(Error::InvalidInput(_))));
}

#[test]
fn integer_widths_and_values_are_validated() {
    assert!(matches!(
        AbiType::parse("uint7"),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        AbiType::parse("int264"),
        Err(Error::InvalidInput(_))
    ));

    let overflow = AbiType::UInt(8).encode(&AbiValue::UInt(BigUint::from(256u32)));
    assert!(matches!(overflow, Err(Error::InvalidInput(_))));

    let signed_overflow = AbiType::Int(8).encode(&AbiValue::Int(BigInt::from(128)));
    assert!(matches!(signed_overflow, Err(Error::InvalidInput(_))));
}

#[test]
fn fixed_array_size_is_element_times_length() {
    let case = vectors().fixed_array;
    let ty = AbiType::parse(&case.name).expect("fixed array parses");

    assert_eq!(ty.fixed_size(), case.fixed_size);
}

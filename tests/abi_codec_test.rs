//! Tests for ABI signature, tuple, and function coding.
#![allow(clippy::expect_used, clippy::indexing_slicing)]

use num_bigint::BigUint;
use serde::Deserialize;
use serde_json::{Value, json};
use sha3::{Digest, Sha3_256};
use znn_sdk_rust::abi::{
    Abi, AbiType, AbiValue, decode_arguments, encode_arguments, format_signature, selector,
};
use znn_sdk_rust::error::Error;
use znn_sdk_rust::primitives::address::Address;

const CODEC_VECTORS: &str = include_str!("vectors/abi/codec.json");

#[derive(Deserialize)]
struct CodecVectors {
    signature: SignatureCase,
    selector: SelectorCase,
    tuple: TupleCase,
    function: FunctionCase,
    #[serde(rename = "roundTripFunction")]
    round_trip_function: RoundTripFunctionCase,
}

#[derive(Deserialize)]
struct SignatureCase {
    name: String,
    types: Vec<String>,
    formatted: String,
}

#[derive(Deserialize)]
struct SelectorCase {
    name: String,
    types: Vec<String>,
    signature: String,
}

#[derive(Deserialize)]
struct TupleCase {
    address: String,
    uint: u64,
    string: String,
    #[serde(rename = "encodedHex")]
    encoded_hex: String,
}

#[derive(Deserialize)]
struct FunctionCase {
    json: Value,
    name: String,
    signature: String,
}

#[derive(Deserialize)]
struct RoundTripFunctionCase {
    name: String,
    types: Vec<String>,
    string: String,
    uint: u64,
    signature: String,
}

fn vectors() -> CodecVectors {
    serde_json::from_str(CODEC_VECTORS).expect("ABI codec vectors parse")
}

fn vector_types(names: &[String]) -> Vec<AbiType> {
    names
        .iter()
        .map(|name| AbiType::parse(name).expect("vector type parses"))
        .collect()
}

fn selector_bytes(signature: &str) -> [u8; 4] {
    let digest = Sha3_256::digest(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

#[test]
fn signature_formats_with_canonical_names() {
    let case = vectors().signature;
    let types = vector_types(&case.types);
    let formatted = format_signature(&case.name, &types);

    assert_eq!(formatted, case.formatted);
}

#[test]
fn selector_is_the_sha3_256_prefix() {
    let case = vectors().selector;
    let expected = selector_bytes(&case.signature);
    let types = vector_types(&case.types);

    assert_eq!(selector(&case.name, &types), expected);
}

#[test]
fn mixed_static_dynamic_tuple_round_trips() {
    let case = vectors().tuple;
    let address = Address::parse(&case.address).expect("address parses");
    let types = [AbiType::UInt(256), AbiType::String, AbiType::Address];
    let values = [
        AbiValue::UInt(BigUint::from(case.uint)),
        AbiValue::String(case.string),
        AbiValue::Address(address),
    ];

    let encoded = encode_arguments(&types, &values).expect("arguments encode");
    let expected = const_hex::decode(case.encoded_hex).expect("tuple encoded hex decodes");

    assert_eq!(encoded, expected, "string content must sit in the tail");
    assert_eq!(decode_arguments(&types, &encoded), Ok(values.to_vec()));
}

#[test]
fn fixed_array_of_dynamic_values_round_trips_as_dynamic_argument() {
    let types = [
        AbiType::parse("string[2]").expect("string[2] parses"),
        AbiType::UInt(256),
    ];
    let values = [
        AbiValue::Array(vec![
            AbiValue::String("alpha".to_string()),
            AbiValue::String("beta".to_string()),
        ]),
        AbiValue::UInt(BigUint::from(7u32)),
    ];

    let encoded = encode_arguments(&types, &values).expect("arguments encode");

    assert_eq!(
        encoded.get(31),
        Some(&64),
        "the dynamic fixed-array argument must be represented by a head offset"
    );
    assert_eq!(decode_arguments(&types, &encoded), Ok(values.to_vec()));
}

#[test]
fn value_count_mismatch_is_rejected() {
    let result = encode_arguments(
        &[AbiType::UInt(256), AbiType::String],
        &[AbiValue::UInt(1u32.into())],
    );

    assert!(
        matches!(result, Err(Error::InvalidInput(_))),
        "mismatched value count must be Error::InvalidInput, got {result:?}"
    );
}

#[test]
fn function_entry_parses_from_json() {
    let case = vectors().function;
    let abi = Abi::from_json(&case.json).expect("function entry parses");
    let encoded = abi
        .encode_function(&case.name, &[AbiValue::Int((-1).into())])
        .expect("function encodes");

    assert_eq!(
        encoded.get(0..4),
        Some(&selector_bytes(&case.signature)[..])
    );
}

#[test]
fn non_function_entry_is_rejected() {
    let result = Abi::from_json(&json!([
        {"type":"event", "name":"X", "inputs":[]}
    ]));

    assert!(
        matches!(result, Err(Error::InvalidInput(_))),
        "non-function ABI entry must be Error::InvalidInput, got {result:?}"
    );
}

#[test]
fn encode_then_decode_function_round_trips() {
    let case = vectors().round_trip_function;
    let abi = Abi::new(vec![(case.name.clone(), vector_types(&case.types))]);
    let values = vec![
        AbiValue::String(case.string),
        AbiValue::UInt(BigUint::from(case.uint)),
    ];

    let encoded = abi
        .encode_function(&case.name, &values)
        .expect("function encodes");

    assert_eq!(
        encoded.get(0..4),
        Some(&selector_bytes(&case.signature)[..]),
        "encoded function must begin with the entry selector"
    );
    assert_eq!(abi.decode_function(&encoded), Ok(values));
}

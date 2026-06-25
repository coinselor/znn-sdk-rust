//! Bech32 bit conversion and Zenon address codec.
//!
//! Provides the bit-group conversion used by Zenon's Bech32 encoding, plus
//! encode and decode helpers for Zenon addresses. Zenon addresses use the
//! human-readable part `z`, a 20-byte core, and the Bech32 checksum.

use crate::error::Error;
use bech32::primitives::decode::CheckedHrpstring;
use bech32::{Bech32, Hrp};

/// Human-readable part for Zenon addresses.
const ZNN_HRP: &str = "z";

/// Byte length of a Zenon address core.
const CORE_SIZE: usize = 20;

/// Converts `data` from groups of `from_bits` bits to groups of `to_bits` bits.
///
/// Each input value must fit within `from_bits` bits. When `pad` is `true`,
/// trailing bits are zero-padded into a final group. When `pad` is `false`,
/// leftover bits must be fewer than `from_bits` and all zero; otherwise
/// [`Error::InvalidInput`] is returned.
pub fn convert_bech32_bits(
    data: &[u8],
    from_bits: u32,
    to_bits: u32,
    pad: bool,
) -> Result<Vec<u8>, Error> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let max_value: u32 = (1u32 << to_bits) - 1;
    let mut result = Vec::new();

    for &byte in data {
        let value = u32::from(byte);
        if (value >> from_bits) != 0 {
            return Err(Error::InvalidInput(format!(
                "value {value} does not fit in {from_bits} bits"
            )));
        }
        acc = (acc << from_bits) | value;
        bits += from_bits;
        while bits >= to_bits {
            bits -= to_bits;
            let group = (acc >> bits) & max_value;
            result.push(group_to_byte(group)?);
        }
    }

    if pad {
        if bits > 0 {
            let group = (acc << (to_bits - bits)) & max_value;
            result.push(group_to_byte(group)?);
        }
    } else if bits >= from_bits {
        return Err(Error::InvalidInput("illegal zero padding".to_string()));
    } else if ((acc << (to_bits - bits)) & max_value) != 0 {
        return Err(Error::InvalidInput("non-zero padding bits".to_string()));
    }

    Ok(result)
}

/// Converts a single output group to a byte, rejecting groups wider than 8 bits.
fn group_to_byte(group: u32) -> Result<u8, Error> {
    u8::try_from(group).map_err(|_| Error::InvalidInput("output group exceeds 8 bits".to_string()))
}

/// Decodes a Zenon Bech32 address into its human-readable part and 20-byte
/// address core. Strings carrying a Bech32m checksum are rejected.
pub fn decode_bech32_address(s: &str) -> Result<(String, Vec<u8>), Error> {
    let checked = CheckedHrpstring::new::<Bech32>(s)
        .map_err(|e| Error::InvalidInput(format!("invalid bech32 address: {e}")))?;

    let hrp = checked.hrp().to_string();
    if hrp != ZNN_HRP {
        return Err(Error::InvalidInput(format!(
            "unexpected human-readable part: {hrp}"
        )));
    }

    let core: Vec<u8> = checked.byte_iter().collect();
    if core.len() != CORE_SIZE {
        return Err(Error::InvalidInput(format!(
            "address core must be {CORE_SIZE} bytes, got {}",
            core.len()
        )));
    }

    Ok((hrp, core))
}

/// Encodes a 20-byte address core with the given human-readable part using the
/// Bech32 checksum, producing a canonical lowercase address string.
pub fn encode_bech32_address(hrp: &str, core: &[u8]) -> Result<String, Error> {
    if core.len() != CORE_SIZE {
        return Err(Error::InvalidInput(format!(
            "address core must be {CORE_SIZE} bytes, got {}",
            core.len()
        )));
    }

    let hrp = Hrp::parse(hrp)
        .map_err(|e| Error::InvalidInput(format!("invalid human-readable part: {e}")))?;

    bech32::encode::<Bech32>(hrp, core)
        .map_err(|e| Error::InvalidInput(format!("bech32 encode failed: {e}")))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ConvertVector {
        from_bits: u32,
        to_bits: u32,
        pad: bool,
        input: Vec<u8>,
        output: Vec<u8>,
    }

    #[derive(Deserialize)]
    struct AddressVectors {
        valid: Vec<ValidAddress>,
        invalid: Vec<InvalidAddress>,
    }

    #[derive(Deserialize)]
    struct ValidAddress {
        address: String,
        hrp: String,
        core_hex: String,
    }

    #[derive(Deserialize)]
    struct InvalidAddress {
        address: String,
    }

    const CONVERT_8_TO_5_PAD: &str =
        include_str!("../../tests/vectors/primitives/bech32/convert_8_to_5_pad.json");
    const CONVERT_5_TO_8_NO_PAD: &str =
        include_str!("../../tests/vectors/primitives/bech32/convert_5_to_8_no_pad.json");
    const ADDRESSES: &str = include_str!("../../tests/vectors/primitives/bech32/addresses.json");

    fn convert_vector(json: &str) -> ConvertVector {
        serde_json::from_str(json).expect("valid convert vector")
    }

    fn address_vectors() -> AddressVectors {
        serde_json::from_str(ADDRESSES).expect("valid address vectors")
    }

    #[test]
    fn convert_8_to_5_with_padding_matches_vector() {
        let v = convert_vector(CONVERT_8_TO_5_PAD);
        let got = convert_bech32_bits(&v.input, v.from_bits, v.to_bits, v.pad)
            .expect("conversion succeeds");
        assert_eq!(got, v.output);
    }

    #[test]
    fn convert_5_to_8_without_padding_matches_vector() {
        let v = convert_vector(CONVERT_5_TO_8_NO_PAD);
        let got = convert_bech32_bits(&v.input, v.from_bits, v.to_bits, v.pad)
            .expect("conversion succeeds");
        assert_eq!(got, v.output);
    }

    #[test]
    fn convert_rejects_invalid_digit() {
        // 8 has bit set at/above from_bits = 3 (8 >> 3 != 0).
        let err = convert_bech32_bits(&[8], 3, 5, true);
        assert!(matches!(err, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn convert_rejects_illegal_zero_padding() {
        // from_bits = 8, to_bits = 5, pad = false: one full 8-bit group of
        // input leaves bits = 8 >= from_bits after emitting one 5-bit group,
        // i.e. leftover bits fill at least one full input group.
        let err = convert_bech32_bits(&[0xff, 0xff], 8, 5, false);
        assert!(matches!(err, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn convert_rejects_non_zero_trailing_bits() {
        // 5-bit value 0b00001 -> trailing bits non-zero when no padding.
        let err = convert_bech32_bits(&[1], 5, 8, false);
        assert!(matches!(err, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn empty_address_round_trip() {
        let vectors = address_vectors();
        let empty = vectors
            .valid
            .iter()
            .find(|a| a.address == "z1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsggv2f")
            .expect("empty vector present");
        let core = const_hex::decode(&empty.core_hex).expect("valid hex");

        let encoded = encode_bech32_address(&empty.hrp, &core).expect("encode succeeds");
        assert_eq!(encoded, empty.address);

        let (hrp, decoded_core) = decode_bech32_address(&empty.address).expect("decode succeeds");
        assert_eq!(hrp, empty.hrp);
        assert_eq!(decoded_core, core);
        assert_eq!(decoded_core.len(), 20);
    }

    #[test]
    fn non_trivial_addresses_round_trip() {
        let vectors = address_vectors();
        for a in &vectors.valid {
            let core = const_hex::decode(&a.core_hex).expect("valid hex");
            assert_eq!(core.len(), 20, "core must be 20 bytes: {}", a.address);

            let (hrp, decoded_core) = decode_bech32_address(&a.address).expect("decode succeeds");
            assert_eq!(hrp, a.hrp);
            assert_eq!(decoded_core, core, "core mismatch for {}", a.address);

            let encoded = encode_bech32_address(&a.hrp, &core).expect("encode succeeds");
            assert_eq!(encoded, a.address, "canonical string mismatch");
        }
    }

    #[test]
    fn decode_rejects_invalid_addresses() {
        let vectors = address_vectors();
        for a in &vectors.invalid {
            assert!(
                decode_bech32_address(&a.address).is_err(),
                "expected error decoding {}",
                a.address
            );
        }
    }

    #[test]
    fn decode_rejects_bech32m_checksum() {
        // A string carrying a Bech32m checksum (HRP z) must be rejected.
        // See the "bech32m_checksum" invalid vector.
        let bech32m_z = "z1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq95cq0t";
        assert!(decode_bech32_address(bech32m_z).is_err());
    }
}

//! Byte helpers: concatenation, fixed-width big-endian integer encoding,
//! left-padding, and lowercase hex rendering.
//!
//! These dependency-free helpers are used across the SDK's binary
//! serialization.

/// Concatenates a list of byte slices in order into a single vector.
pub fn merge(arrays: &[&[u8]]) -> Vec<u8> {
    let capacity = arrays.iter().map(|a| a.len()).sum();
    let mut merged = Vec::with_capacity(capacity);
    for array in arrays {
        merged.extend_from_slice(array);
    }
    merged
}

/// Returns the 4-byte big-endian encoding of a 32-bit unsigned value.
pub fn int_to_bytes(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

/// Returns the 8-byte big-endian encoding of a 64-bit unsigned value.
pub fn long_to_bytes(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

/// Left-pads `bytes` with leading zeros to `size` bytes, returning the input
/// unchanged when it is already at least `size` bytes long.
pub fn left_pad_bytes(bytes: &[u8], size: usize) -> Vec<u8> {
    if bytes.len() >= size {
        return bytes.to_vec();
    }
    let mut padded = vec![0u8; size - bytes.len()];
    padded.extend_from_slice(bytes);
    padded
}

/// Returns the canonical lowercase hexadecimal string of a byte slice.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const_hex::encode(bytes)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct MergeVectors {
        cases: Vec<MergeCase>,
    }

    #[derive(Deserialize)]
    struct MergeCase {
        name: String,
        arrays: Vec<Vec<u8>>,
        merged: Vec<u8>,
    }

    #[derive(Deserialize)]
    struct IntVectors {
        cases: Vec<IntCase>,
    }

    #[derive(Deserialize)]
    struct IntCase {
        name: String,
        value: u32,
        bytes: Vec<u8>,
    }

    #[derive(Deserialize)]
    struct LongVectors {
        cases: Vec<LongCase>,
    }

    #[derive(Deserialize)]
    struct LongCase {
        name: String,
        value: u64,
        bytes: Vec<u8>,
    }

    #[derive(Deserialize)]
    struct LeftPadVectors {
        cases: Vec<LeftPadCase>,
    }

    #[derive(Deserialize)]
    struct LeftPadCase {
        name: String,
        bytes: Vec<u8>,
        size: usize,
        padded: Vec<u8>,
    }

    #[derive(Deserialize)]
    struct HexVectors {
        cases: Vec<HexCase>,
    }

    #[derive(Deserialize)]
    struct HexCase {
        name: String,
        bytes: Vec<u8>,
        hex: String,
    }

    const MERGE: &str = include_str!("../../tests/vectors/utils/bytes/merge.json");
    const INT_TO_BYTES: &str = include_str!("../../tests/vectors/utils/bytes/int_to_bytes.json");
    const LONG_TO_BYTES: &str = include_str!("../../tests/vectors/utils/bytes/long_to_bytes.json");
    const LEFT_PAD: &str = include_str!("../../tests/vectors/utils/bytes/left_pad.json");
    const BYTES_TO_HEX: &str = include_str!("../../tests/vectors/utils/bytes/bytes_to_hex.json");

    #[test]
    fn merge_concatenates_known_vectors() {
        let cases = serde_json::from_str::<MergeVectors>(MERGE)
            .expect("valid merge vectors")
            .cases;
        for case in cases {
            let slices: Vec<&[u8]> = case.arrays.iter().map(Vec::as_slice).collect();
            assert_eq!(merge(&slices), case.merged, "merge for {}", case.name);
        }
    }

    #[test]
    fn int_to_bytes_matches_known_vectors() {
        let cases = serde_json::from_str::<IntVectors>(INT_TO_BYTES)
            .expect("valid int_to_bytes vectors")
            .cases;
        for case in cases {
            assert_eq!(
                int_to_bytes(case.value).as_slice(),
                case.bytes.as_slice(),
                "int_to_bytes for {}",
                case.name
            );
        }
    }

    #[test]
    fn int_to_bytes_zero_boundary() {
        assert_eq!(int_to_bytes(0), [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn int_to_bytes_max_boundary() {
        assert_eq!(int_to_bytes(u32::MAX), [0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn long_to_bytes_matches_known_vectors() {
        let cases = serde_json::from_str::<LongVectors>(LONG_TO_BYTES)
            .expect("valid long_to_bytes vectors")
            .cases;
        for case in cases {
            assert_eq!(
                long_to_bytes(case.value).as_slice(),
                case.bytes.as_slice(),
                "long_to_bytes for {}",
                case.name
            );
        }
    }

    #[test]
    fn long_to_bytes_zero_boundary() {
        assert_eq!(long_to_bytes(0), [0x00; 8]);
    }

    #[test]
    fn long_to_bytes_max_boundary() {
        assert_eq!(long_to_bytes(u64::MAX), [0xff; 8]);
    }

    #[test]
    fn left_pad_bytes_matches_known_vectors() {
        let cases = serde_json::from_str::<LeftPadVectors>(LEFT_PAD)
            .expect("valid left_pad vectors")
            .cases;
        for case in cases {
            assert_eq!(
                left_pad_bytes(&case.bytes, case.size),
                case.padded,
                "left_pad_bytes for {}",
                case.name
            );
        }
    }

    #[test]
    fn bytes_to_hex_matches_known_vectors() {
        let cases = serde_json::from_str::<HexVectors>(BYTES_TO_HEX)
            .expect("valid bytes_to_hex vectors")
            .cases;
        for case in cases {
            assert_eq!(
                bytes_to_hex(&case.bytes),
                case.hex,
                "bytes_to_hex for {}",
                case.name
            );
        }
    }
}

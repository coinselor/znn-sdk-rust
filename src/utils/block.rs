//! Account-block utilities: block-type classification and the canonical
//! transaction serialization/hash.
//!
//! This module holds the pure, deterministic serialization contract. The
//! serialized layout is 306 bytes for a fully-populated template.

use crate::crypto::crypto;
use crate::error::Error;
use crate::model::nom::account_block_template::{AccountBlockTemplate, BlockType};
use crate::primitives::hash::Hash;
use crate::utils::bytes::{left_pad_bytes, long_to_bytes, merge};
use num_bigint::BigUint;

/// Returns `true` for `UserSend` and `ContractSend` block types.
pub fn is_send_block(block_type: u32) -> bool {
    matches!(
        BlockType::from_u32(block_type),
        Some(BlockType::UserSend | BlockType::ContractSend)
    )
}

/// Returns `true` for `UserReceive` and `GenesisReceive` block types.
///
/// `ContractReceive` is intentionally not classified as a receive block.
pub fn is_receive_block(block_type: u32) -> bool {
    matches!(
        BlockType::from_u32(block_type),
        Some(BlockType::UserReceive | BlockType::GenesisReceive)
    )
}

/// Returns the canonical transaction serialization of `template`.
///
/// The 306 bytes concatenate, in order: version, chainIdentifier, blockType
/// (8-byte big-endian each), previousHash (32), height (8), momentumAcknowledged
/// (40), address core (20), toAddress core (20), amount (32-byte big-endian
/// left-padded), tokenStandard (10), fromBlockHash (32), the digest of the empty
/// descendent list (32), the digest of `data` (32), fusedPlasma, difficulty
/// (8-byte big-endian each), and the nonce (hex-decoded and left-padded to 8).
///
/// Returns [`Error::InvalidInput`] when the template `nonce` is not valid
/// hexadecimal or decodes to more than 8 bytes.
pub fn get_transaction_bytes(template: &AccountBlockTemplate) -> Result<Vec<u8>, Error> {
    let version = long_to_bytes(u64::from(template.version()));
    let chain_identifier = long_to_bytes(u64::from(template.chain_identifier()));
    let block_type = long_to_bytes(u64::from(template.block_type()));
    let previous_hash = template.previous_hash().bytes();
    let height = long_to_bytes(template.height());
    let momentum_acknowledged = template.momentum_acknowledged().get_bytes();
    let address = template.address().core();
    let to_address = template.to_address().core();
    let amount = big_uint_to_fixed_bytes(template.amount(), 32);
    let token_standard = template.token_standard().get_bytes();
    let from_block_hash = template.from_block_hash().bytes();
    let descendent_blocks = crypto::digest(&[]);
    let data = crypto::digest(template.data());
    let fused_plasma = long_to_bytes(u64::from(template.fused_plasma()));
    let difficulty = long_to_bytes(u64::from(template.difficulty()));
    let nonce_bytes = const_hex::decode(template.nonce())
        .map_err(|e| Error::InvalidInput(format!("nonce must be valid hex: {e}")))?;
    if nonce_bytes.len() > 8 {
        return Err(Error::InvalidInput(format!(
            "nonce must decode to at most 8 bytes, got {}",
            nonce_bytes.len()
        )));
    }
    let nonce = left_pad_bytes(&nonce_bytes, 8);
    Ok(merge(&[
        &version,
        &chain_identifier,
        &block_type,
        previous_hash,
        &height,
        &momentum_acknowledged,
        address,
        to_address,
        &amount,
        token_standard,
        from_block_hash,
        &descendent_blocks,
        &data,
        &fused_plasma,
        &difficulty,
        &nonce,
    ]))
}

/// Returns the transaction hash: `sha3-256` of the transaction bytes.
///
/// Propagates [`Error::InvalidInput`] when the template `nonce` is malformed.
pub fn get_transaction_hash(template: &AccountBlockTemplate) -> Result<Hash, Error> {
    Ok(digest_to_hash(&get_transaction_bytes(template)?))
}

#[allow(clippy::expect_used)]
fn digest_to_hash(data: &[u8]) -> Hash {
    Hash::from_bytes(&crypto::digest(data)).expect("sha3-256 yields 32 bytes")
}

fn big_uint_to_fixed_bytes(value: &BigUint, size: usize) -> Vec<u8> {
    left_pad_bytes(&value.to_bytes_be(), size)
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]
mod tests {
    use super::*;
    use crate::primitives::address::Address;
    use crate::primitives::token_standard::TokenStandard;
    use crate::utils::bytes::bytes_to_hex;
    use num_bigint::BigUint;

    fn sample_template() -> AccountBlockTemplate {
        let to = Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap();
        let ts = TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap();
        AccountBlockTemplate::send(
            to,
            ts,
            BigUint::from(123_456_789u64),
            Some(vec![0xde, 0xad, 0xbe, 0xef]),
        )
        .with_nonce("0123456789abcdef")
    }

    fn expected_bytes(template: &AccountBlockTemplate) -> Vec<u8> {
        let version = long_to_bytes(u64::from(template.version()));
        let chain_identifier = long_to_bytes(u64::from(template.chain_identifier()));
        let block_type = long_to_bytes(u64::from(template.block_type()));
        let previous_hash = template.previous_hash().bytes();
        let height = long_to_bytes(template.height());
        let momentum_acknowledged = template.momentum_acknowledged().get_bytes();
        let address = template.address().core();
        let to_address = template.to_address().core();
        let amount = big_uint_to_fixed_bytes(template.amount(), 32);
        let token_standard = template.token_standard().get_bytes();
        let from_block_hash = template.from_block_hash().bytes();
        let descendent_blocks = crypto::digest(&[]);
        let data = crypto::digest(template.data());
        let fused_plasma = long_to_bytes(u64::from(template.fused_plasma()));
        let difficulty = long_to_bytes(u64::from(template.difficulty()));
        let nonce = left_pad_bytes(&const_hex::decode(template.nonce()).unwrap_or_default(), 8);
        merge(&[
            &version,
            &chain_identifier,
            &block_type,
            previous_hash,
            &height,
            &momentum_acknowledged,
            address,
            to_address,
            &amount,
            token_standard,
            from_block_hash,
            &descendent_blocks,
            &data,
            &fused_plasma,
            &difficulty,
            &nonce,
        ])
    }

    #[test]
    fn is_send_block_classifies_send_and_non_send_ordinals() {
        assert!(is_send_block(BlockType::UserSend.as_u32()));
        assert!(is_send_block(BlockType::ContractSend.as_u32()));
        assert!(!is_send_block(BlockType::UserReceive.as_u32()));
        assert!(!is_send_block(BlockType::Unknown.as_u32()));
    }

    #[test]
    fn is_receive_block_classifies_receive_ordinals() {
        assert!(is_receive_block(BlockType::UserReceive.as_u32()));
        assert!(is_receive_block(BlockType::GenesisReceive.as_u32()));
        assert!(
            !is_receive_block(BlockType::ContractReceive.as_u32()),
            "contractReceive is not classified as a receive block"
        );
        assert!(!is_receive_block(BlockType::UserSend.as_u32()));
    }

    #[test]
    fn transaction_bytes_are_306_bytes_and_match_composition() {
        let template = sample_template();
        let bytes = get_transaction_bytes(&template).expect("template serializes");
        assert_eq!(bytes.len(), 306, "transaction bytes must be 306 bytes");
        assert_eq!(
            bytes,
            expected_bytes(&template),
            "bytes must match the composed layout"
        );
    }

    #[test]
    fn transaction_bytes_place_block_type_at_offset_16() {
        let template = sample_template();
        let bytes = get_transaction_bytes(&template).expect("template serializes");
        assert_eq!(bytes.len(), 306, "transaction bytes must be 306 bytes");
        assert_eq!(&bytes[16..23], &[0u8; 7], "block-type high bytes are zero");
        assert_eq!(bytes[23], BlockType::UserSend.as_u32() as u8);
    }

    #[test]
    fn transaction_bytes_encode_a_non_empty_nonce() {
        let template = sample_template();
        let bytes = get_transaction_bytes(&template).expect("template serializes");
        assert_eq!(bytes.len(), 306, "transaction bytes must be 306 bytes");
        assert_eq!(
            &bytes[298..306],
            &[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
            "the final 8 bytes must be the decoded, left-padded nonce"
        );
    }

    #[test]
    fn get_transaction_bytes_rejects_a_malformed_nonce() {
        let to = Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap();
        let ts = TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap();
        let template =
            AccountBlockTemplate::send(to, ts, BigUint::from(1u64), None).with_nonce("not-hex!");
        let err = get_transaction_bytes(&template).expect_err("malformed nonce is rejected");
        assert!(matches!(err, Error::InvalidInput(_)), "got {err:?}");
        let hash_err =
            get_transaction_hash(&template).expect_err("hash propagates the nonce error");
        assert!(
            matches!(hash_err, Error::InvalidInput(_)),
            "got {hash_err:?}"
        );
    }

    #[test]
    fn get_transaction_bytes_rejects_an_overlong_nonce() {
        let to = Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap();
        let ts = TokenStandard::parse("zts1znnxxxxxxxxxxxxx9z4ulx").unwrap();
        let template = AccountBlockTemplate::send(to, ts, BigUint::from(1u64), None)
            .with_nonce("000102030405060708");

        let err = get_transaction_bytes(&template).expect_err("overlong nonce is rejected");

        assert!(matches!(err, Error::InvalidInput(_)), "got {err:?}");
    }

    #[test]
    fn transaction_hash_is_the_digest_of_the_bytes() {
        let template = sample_template();
        let bytes = get_transaction_bytes(&template).expect("template serializes");
        let expected = digest_to_hash(&bytes);
        let hash = get_transaction_hash(&template).expect("hash computes");
        assert_eq!(hash, expected);
        // sanity: the hash renders as 64 hex characters
        assert_eq!(bytes_to_hex(hash.bytes()).len(), 64);
    }
}

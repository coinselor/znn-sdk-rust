//! Plasma contract models.

use crate::error::Error;
use crate::model::json::*;
use crate::model::nom::account_block_template::BlockType;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use num_bigint::BigUint;
use serde_json::{Map, Value, json};

/// A plasma fusion entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FusionEntry {
    qsr_amount: BigUint,
    beneficiary: Address,
    expiration_height: u64,
    id: Hash,
    is_revocable: Option<bool>,
}

impl FusionEntry {
    /// Creates a fusion entry.
    pub fn new(
        qsr_amount: BigUint,
        beneficiary: Address,
        expiration_height: u64,
        id: Hash,
        is_revocable: Option<bool>,
    ) -> Self {
        Self {
            qsr_amount,
            beneficiary,
            expiration_height,
            id,
            is_revocable,
        }
    }

    /// Returns the qsr amount.
    pub fn qsr_amount(&self) -> &BigUint {
        &self.qsr_amount
    }
    /// Returns the beneficiary.
    pub fn beneficiary(&self) -> &Address {
        &self.beneficiary
    }
    /// Returns the expiration height.
    pub fn expiration_height(&self) -> u64 {
        self.expiration_height
    }
    /// Returns the id.
    pub fn id(&self) -> &Hash {
        &self.id
    }
    /// Returns whether revocable.
    pub fn is_revocable(&self) -> Option<bool> {
        self.is_revocable
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "qsrAmount": self.qsr_amount.to_string(),
            "beneficiary": self.beneficiary.to_string(),
            "expirationHeight": self.expiration_height,
            "id": self.id.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "fusion entry")?;
        Ok(Self {
            qsr_amount: required_big_uint(object, "qsrAmount")?,
            beneficiary: Address::parse(required_str(object, "beneficiary")?)?,
            expiration_height: required_u64(object, "expirationHeight")?,
            id: Hash::parse(required_str(object, "id")?)?,
            is_revocable: None,
        })
    }
}

/// A paged list of fusion entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FusionEntryList {
    qsr_amount: BigUint,
    count: u64,
    list: Vec<FusionEntry>,
}

impl FusionEntryList {
    /// Creates a fusion entry list.
    pub fn new(qsr_amount: BigUint, count: u64, list: Vec<FusionEntry>) -> Self {
        Self {
            qsr_amount,
            count,
            list,
        }
    }

    /// Returns the qsr amount.
    pub fn qsr_amount(&self) -> &BigUint {
        &self.qsr_amount
    }
    /// Returns the count.
    pub fn count(&self) -> u64 {
        self.count
    }
    /// Returns the list.
    pub fn list(&self) -> &[FusionEntry] {
        &self.list
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "count": self.count,
            "list": self.list.iter().map(FusionEntry::to_json).collect::<Vec<_>>(),
            "qsrAmount": self.qsr_amount.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "fusion entry list")?;
        Ok(Self::new(
            required_big_uint(object, "qsrAmount")?,
            required_u64(object, "count")?,
            required_array(object, "list", FusionEntry::from_json)?,
        ))
    }
}

/// Plasma balance info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlasmaInfo {
    current_plasma: u64,
    max_plasma: u64,
    qsr_amount: BigUint,
}

impl PlasmaInfo {
    /// Creates plasma info.
    pub fn new(current_plasma: u64, max_plasma: u64, qsr_amount: BigUint) -> Self {
        Self {
            current_plasma,
            max_plasma,
            qsr_amount,
        }
    }

    /// Returns the current plasma.
    pub fn current_plasma(&self) -> u64 {
        self.current_plasma
    }
    /// Returns the max plasma.
    pub fn max_plasma(&self) -> u64 {
        self.max_plasma
    }
    /// Returns the qsr amount.
    pub fn qsr_amount(&self) -> &BigUint {
        &self.qsr_amount
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "currentPlasma": self.current_plasma,
            "maxPlasma": self.max_plasma,
            "qsrAmount": self.qsr_amount.to_string(),
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "plasma info")?;
        Ok(Self::new(
            required_u64(object, "currentPlasma")?,
            required_u64(object, "maxPlasma")?,
            required_big_uint(object, "qsrAmount")?,
        ))
    }
}

/// Input for a required-plasma query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetRequiredParam {
    address: Address,
    block_type: BlockType,
    to_address: Option<Address>,
    data: Vec<u8>,
}

impl GetRequiredParam {
    /// Creates a required-plasma param. For `UserReceive`, `to_address` is set
    /// to `address`.
    pub fn new(
        address: Address,
        block_type: BlockType,
        to_address: Option<Address>,
        data: Vec<u8>,
    ) -> Self {
        let to_address = if block_type == BlockType::UserReceive {
            Some(address.clone())
        } else {
            to_address
        };
        Self {
            address,
            block_type,
            to_address,
            data,
        }
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }
    /// Returns the block type.
    pub fn block_type(&self) -> BlockType {
        self.block_type
    }
    /// Returns the to address.
    pub fn to_address(&self) -> Option<&Address> {
        self.to_address.as_ref()
    }
    /// Returns the data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "address": self.address.to_string(),
            "blockType": self.block_type.as_u32().to_string(),
            "toAddress": self.to_address.as_ref().map(Address::to_string),
            "data": STANDARD.encode(&self.data),
        })
    }

    /// Deserializes from a JSON object, reading `toAddress` verbatim.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "get required param")?;
        Ok(Self {
            address: Address::parse(required_str(object, "address")?)?,
            block_type: required_block_type(object, "blockType")?,
            to_address: optional_address(object, "toAddress")?,
            data: optional_base64(object, "data")?,
        })
    }
}

/// Response from a required-plasma query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetRequiredResponse {
    available_plasma: u64,
    base_plasma: u64,
    required_difficulty: u64,
}

impl GetRequiredResponse {
    /// Creates a required-plasma response.
    pub fn new(available_plasma: u64, base_plasma: u64, required_difficulty: u64) -> Self {
        Self {
            available_plasma,
            base_plasma,
            required_difficulty,
        }
    }

    /// Returns the available plasma.
    pub fn available_plasma(&self) -> u64 {
        self.available_plasma
    }
    /// Returns the base plasma.
    pub fn base_plasma(&self) -> u64 {
        self.base_plasma
    }
    /// Returns the required difficulty.
    pub fn required_difficulty(&self) -> u64 {
        self.required_difficulty
    }

    /// Serializes to a JSON object.
    pub fn to_json(&self) -> Value {
        json!({
            "availablePlasma": self.available_plasma,
            "basePlasma": self.base_plasma,
            "requiredDifficulty": self.required_difficulty,
        })
    }

    /// Deserializes from a JSON object.
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let object = json_object(value, "get required response")?;
        Ok(Self::new(
            required_u64(object, "availablePlasma")?,
            required_u64(object, "basePlasma")?,
            required_u64(object, "requiredDifficulty")?,
        ))
    }
}

fn required_block_type(object: &Map<String, Value>, field: &str) -> Result<BlockType, Error> {
    let s = required_str(object, field)?;
    let n: u32 = s
        .parse()
        .map_err(|_| Error::InvalidInput(format!("{field} must be a decimal string")))?;
    BlockType::from_u32(n).ok_or_else(|| Error::InvalidInput(format!("{field} is out of range")))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Conformance {
        #[allow(dead_code)]
        description: String,
        fusion_entry: Value,
        fusion_entry_list: Value,
        plasma_info: Value,
        get_required_param: Value,
        get_required_response: Value,
    }

    const CONFORMANCE: &str = include_str!("../../../tests/conformance/embedded/plasma.json");

    fn conf() -> Conformance {
        serde_json::from_str(CONFORMANCE).expect("valid plasma conformance")
    }

    #[test]
    fn fusion_entry_round_trip() {
        let original = FusionEntry::new(
            BigUint::from(5_000_000_000u64),
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            100,
            Hash::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
            None,
        );
        let round_trip = FusionEntry::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
        assert_eq!(round_trip.is_revocable(), None);
    }

    #[test]
    fn fusion_entry_list_round_trip() {
        let value = conf().fusion_entry_list;
        let list = FusionEntryList::from_json(&value).expect("conformance parses");
        assert_eq!(list.to_json(), value);
        assert_eq!(list.count(), 1);
    }

    #[test]
    fn plasma_info_round_trip() {
        let value = conf().plasma_info;
        let info = PlasmaInfo::from_json(&value).expect("conformance parses");
        assert_eq!(info.to_json(), value);
    }

    #[test]
    fn get_required_param_user_receive_rule() {
        let address = Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap();
        let param =
            GetRequiredParam::new(address.clone(), BlockType::UserReceive, None, Vec::new());
        assert_eq!(param.to_address(), Some(&address));
    }

    #[test]
    fn get_required_param_block_type_parsed_from_decimal_string() {
        let param =
            GetRequiredParam::from_json(&conf().get_required_param).expect("conformance parses");
        assert_eq!(param.block_type(), BlockType::UserSend);
    }

    #[test]
    fn get_required_param_round_trip() {
        let original = GetRequiredParam::new(
            Address::parse("z1qzal6c5s9rjnnxd2z7dvdhjxpmmj4fmw56a0mz").unwrap(),
            BlockType::UserSend,
            Some(Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx").unwrap()),
            vec![0x00, 0x01, 0x02],
        );
        let round_trip =
            GetRequiredParam::from_json(&original.to_json()).expect("round-trip parses");
        assert_eq!(round_trip, original);
        assert_eq!(round_trip.data(), [0x00, 0x01, 0x02]);
    }

    #[test]
    fn get_required_param_missing_data_defaults_to_empty() {
        let mut missing = conf().get_required_param;
        missing.as_object_mut().unwrap().remove("data");
        let param = GetRequiredParam::from_json(&missing).expect("missing data parses");
        assert!(param.data().is_empty());

        // A populated data field must decode.
        assert_eq!(
            GetRequiredParam::from_json(&conf().get_required_param)
                .expect("conformance parses")
                .data(),
            [0x00, 0x01, 0x02]
        );
    }

    #[test]
    fn get_required_param_rejects_malformed_data() {
        let mut bad_base64 = conf().get_required_param;
        bad_base64["data"] = json!("@@@@");
        let result = GetRequiredParam::from_json(&bad_base64);
        assert!(result.is_err(), "invalid base64 data must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));

        let mut wrong_kind = conf().get_required_param;
        wrong_kind["data"] = json!(42);
        let result = GetRequiredParam::from_json(&wrong_kind);
        assert!(result.is_err(), "non-string data must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn get_required_param_reads_to_address_verbatim() {
        let param =
            GetRequiredParam::from_json(&conf().get_required_param).expect("conformance parses");
        assert_eq!(
            param.to_address().map(Address::to_string),
            Some("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx".to_string())
        );
    }

    #[test]
    fn get_required_param_rejects_malformed() {
        let mut bad = conf().get_required_param;
        bad.as_object_mut().unwrap().remove("address");
        let result = GetRequiredParam::from_json(&bad);
        assert!(result.is_err(), "missing address must be rejected");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn get_required_response_round_trip() {
        let value = conf().get_required_response;
        let response = GetRequiredResponse::from_json(&value).expect("conformance parses");
        assert_eq!(response.to_json(), value);
    }
}

//! Password-encrypted file format for keystores.
//!
//! An [`EncryptedFile`] is a JSON document with optional metadata, an AES-256-GCM
//! `crypto` section, a timestamp, and a version. Binary fields (`salt`, `nonce`,
//! `cipherData`) are `0x`-prefixed hex. Decryption derives the AES key with
//! Argon2id and authenticates with the associated data `"zenon"`.

use crate::crypto::argon2;
use crate::error::Error;
use crate::wallet::exceptions::WalletError;
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use serde_json::{Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

/// Associated data authenticated alongside the ciphertext.
const AAD: &[u8] = b"zenon";
/// Cipher name recorded in the file.
const CIPHER_NAME: &str = "aes-256-gcm";
/// Key derivation function identifier recorded in the file.
const KDF: &str = "argon2.IDKey";

/// Argon2 parameters stored in an encrypted file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Argon2Params {
    /// Argon2 salt.
    pub salt: Vec<u8>,
}

/// The `crypto` section of an encrypted file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CryptoData {
    /// Argon2 KDF parameters.
    pub argon2_params: Option<Argon2Params>,
    /// Ciphertext followed by the 16-byte GCM authentication tag.
    pub cipher_data: Vec<u8>,
    /// Cipher name, for example `aes-256-gcm`.
    pub cipher_name: String,
    /// Key derivation function identifier.
    pub kdf: String,
    /// AES-GCM nonce.
    pub nonce: Vec<u8>,
}

/// A password-encrypted file.
#[derive(Debug, Clone, Default)]
pub struct EncryptedFile {
    /// Optional plaintext metadata stored alongside the ciphertext.
    pub metadata: Option<Map<String, Value>>,
    /// The encryption parameters and ciphertext.
    pub crypto: Option<CryptoData>,
    /// Unix timestamp (seconds) when the file was written.
    pub timestamp: Option<i64>,
    /// File format version.
    pub version: Option<u32>,
}

/// Decodes a `0x`-prefixed hex string.
fn from_hex_0x(value: &Value) -> Result<Vec<u8>, Error> {
    let s = value
        .as_str()
        .ok_or_else(|| Error::InvalidInput("expected a hex string".to_string()))?;
    let hex = s.strip_prefix("0x").unwrap_or(s);
    const_hex::decode(hex).map_err(|e| Error::InvalidInput(format!("invalid hex: {e}")))
}

/// Encodes bytes as a `0x`-prefixed hex string.
fn to_hex_0x(bytes: &[u8]) -> String {
    format!("0x{}", const_hex::encode(bytes))
}

impl CryptoData {
    fn from_value(value: &Value) -> Result<Self, Error> {
        let obj = value
            .as_object()
            .ok_or_else(|| Error::InvalidInput("crypto must be an object".to_string()))?;
        let argon2_params = match obj.get("argon2Params") {
            Some(p) => {
                let salt = from_hex_0x(
                    p.get("salt")
                        .ok_or_else(|| Error::InvalidInput("missing argon2 salt".to_string()))?,
                )?;
                Some(Argon2Params { salt })
            }
            None => None,
        };
        let cipher_data = from_hex_0x(
            obj.get("cipherData")
                .ok_or_else(|| Error::InvalidInput("missing cipherData".to_string()))?,
        )?;
        let nonce = from_hex_0x(
            obj.get("nonce")
                .ok_or_else(|| Error::InvalidInput("missing nonce".to_string()))?,
        )?;
        let cipher_name = obj
            .get("cipherName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let kdf = obj
            .get("kdf")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        Ok(Self {
            argon2_params,
            cipher_data,
            cipher_name,
            kdf,
            nonce,
        })
    }

    fn to_value(&self) -> Value {
        let mut obj = Map::new();
        if let Some(params) = &self.argon2_params {
            let mut p = Map::new();
            p.insert("salt".to_string(), Value::String(to_hex_0x(&params.salt)));
            obj.insert("argon2Params".to_string(), Value::Object(p));
        }
        obj.insert(
            "cipherData".to_string(),
            Value::String(to_hex_0x(&self.cipher_data)),
        );
        obj.insert(
            "cipherName".to_string(),
            Value::String(self.cipher_name.clone()),
        );
        obj.insert("kdf".to_string(), Value::String(self.kdf.clone()));
        obj.insert("nonce".to_string(), Value::String(to_hex_0x(&self.nonce)));
        Value::Object(obj)
    }
}

impl EncryptedFile {
    /// Parses an encrypted file from its JSON representation.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let value: Value =
            serde_json::from_str(json).map_err(|e| Error::Serialization(e.to_string()))?;
        let mut obj = value
            .as_object()
            .ok_or_else(|| Error::InvalidInput("encrypted file must be an object".to_string()))?
            .clone();

        let crypto = match obj.remove("crypto") {
            Some(c) => Some(CryptoData::from_value(&c)?),
            None => None,
        };
        let timestamp = obj.remove("timestamp").and_then(|v| v.as_i64());
        let version = obj
            .remove("version")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok());
        let metadata = if obj.is_empty() { None } else { Some(obj) };

        Ok(Self {
            metadata,
            crypto,
            timestamp,
            version,
        })
    }

    /// Returns the JSON representation as a [`Value`].
    pub fn to_json_value(&self) -> Result<Value, Error> {
        let mut obj = self.metadata.clone().unwrap_or_default();
        if let Some(crypto) = &self.crypto {
            obj.insert("crypto".to_string(), crypto.to_value());
        }
        if let Some(timestamp) = self.timestamp {
            obj.insert("timestamp".to_string(), Value::from(timestamp));
        }
        if let Some(version) = self.version {
            obj.insert("version".to_string(), Value::from(version));
        }
        Ok(Value::Object(obj))
    }

    /// Returns the JSON representation as a string.
    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(&self.to_json_value()?)
            .map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Returns the file's metadata, if any.
    pub fn metadata(&self) -> Option<&Map<String, Value>> {
        self.metadata.as_ref()
    }

    /// Encrypts `data` with `password`, attaching optional `metadata`.
    pub fn encrypt(
        data: &[u8],
        password: &str,
        metadata: Option<Map<String, Value>>,
    ) -> Result<Self, Error> {
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        getrandom::getrandom(&mut salt)
            .map_err(|e| Error::generic(format!("randomness unavailable: {e}")))?;
        getrandom::getrandom(&mut nonce_bytes)
            .map_err(|e| Error::generic(format!("randomness unavailable: {e}")))?;

        let key = Zeroizing::new(argon2::derive_key(password.as_bytes(), &salt)?);
        let cipher = Aes256Gcm::new_from_slice(key.as_slice())
            .map_err(|e| Error::generic(format!("aes key error: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher_data = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: data,
                    aad: AAD,
                },
            )
            .map_err(|e| Error::generic(format!("encryption failed: {e}")))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));

        Ok(Self {
            metadata,
            crypto: Some(CryptoData {
                argon2_params: Some(Argon2Params {
                    salt: salt.to_vec(),
                }),
                cipher_data,
                cipher_name: CIPHER_NAME.to_string(),
                kdf: KDF.to_string(),
                nonce: nonce_bytes.to_vec(),
            }),
            timestamp: Some(timestamp),
            version: Some(1),
        })
    }

    /// Decrypts the file with `password`, returning the plaintext.
    ///
    /// Returns [`WalletError::IncorrectPassword`] if authentication fails.
    pub fn decrypt(&self, password: &str) -> Result<Vec<u8>, WalletError> {
        if self.version != Some(1) {
            return Err(WalletError::wallet(format!(
                "unsupported encrypted-file version: {:?}",
                self.version
            )));
        }
        let crypto = self
            .crypto
            .as_ref()
            .ok_or_else(|| WalletError::wallet("encrypted file has no crypto section"))?;
        if crypto.cipher_name != CIPHER_NAME {
            return Err(WalletError::wallet(format!(
                "unsupported cipher: {}",
                crypto.cipher_name
            )));
        }
        if crypto.kdf != KDF {
            return Err(WalletError::wallet(format!(
                "unsupported kdf: {}",
                crypto.kdf
            )));
        }
        let params = crypto
            .argon2_params
            .as_ref()
            .ok_or_else(|| WalletError::wallet("encrypted file has no argon2 parameters"))?;
        if params.salt.len() != 16 {
            return Err(WalletError::wallet("salt must be 16 bytes"));
        }
        if crypto.nonce.len() != 12 {
            return Err(WalletError::wallet("nonce must be 12 bytes"));
        }
        if crypto.cipher_data.len() < 16 {
            return Err(WalletError::wallet(
                "cipherData must include a 16-byte authentication tag",
            ));
        }

        let key = Zeroizing::new(
            argon2::derive_key(password.as_bytes(), &params.salt)
                .map_err(|e| WalletError::wallet(e.to_string()))?,
        );
        let cipher = Aes256Gcm::new_from_slice(key.as_slice())
            .map_err(|e| WalletError::wallet(format!("aes key error: {e}")))?;
        let nonce = Nonce::from_slice(&crypto.nonce);
        cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &crypto.cipher_data,
                    aad: AAD,
                },
            )
            .map_err(|_| WalletError::IncorrectPassword)
    }
}

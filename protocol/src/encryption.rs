use aes_gcm_siv::{
  aead::{Aead, KeyInit},
  Aes256GcmSiv, Nonce,
};
use base64::prelude::*;
use rand::Rng;
use serde::Serialize;
use serde_json::json;

use crate::model::Envelope;

fn generate_nonce() -> [u8; 12] {
  let mut rng = rand::thread_rng();
  let mut nonce = [0u8; 12];
  rng.fill(&mut nonce);
  nonce
}

pub fn encrypt_envelope<T>(data: &T, key: &Vec<u8>) -> Envelope
where
  T: Serialize,
{
  let serde_json = json!(data);

  let canonicalized = canonize_json(&serde_json);
  let nonce_bytes = generate_nonce();
  let nonce = Nonce::from_slice(&nonce_bytes); // unique to each receiver and included in message
  let cipher = Aes256GcmSiv::new(key[..].into());
  let encrypted = match cipher.encrypt(nonce, canonicalized.as_bytes()) {
    Ok(ciphertext) => BASE64_STANDARD.encode(ciphertext),
    Err(e) => panic!("Error encrypting data: {}", e),
  };
  Envelope {
    encrypted,
    nonce: BASE64_STANDARD.encode(nonce_bytes),
  }
}

#[cfg(test)]
mod encrypt_tests {
  use std::hash::DefaultHasher;

  use aes_gcm_siv::{
    aead::{Aead, KeyInit, Payload},
    Aes256GcmSiv,
  };
  use base64::prelude::*;
  use pretty_assertions::assert_eq;
  use rand::rngs::OsRng;
  use serde::{Deserialize, Serialize};
  use serde_json::json;
  use std::hash::{Hash, Hasher};

  fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
  }

  pub fn generate_hash<T>(data: &T) -> u64
  where
    T: Serialize,
  {
    let serde_json_val = json!(data);

    let canonicalized = match json_canon::to_string(&serde_json_val) {
      Ok(canonicalized) => canonicalized,
      Err(e) => panic!("Error canonicalizing JSON: {}", e),
    };

    calculate_hash(&canonicalized)
  }
  #[derive(Serialize, Deserialize, Debug)]
  pub struct DummyLineItem;

  #[derive(Serialize, Deserialize, Debug)]
  pub struct DummyReceipt {
    /// The Unix Epoch timestamp of the transaction authorization
    pub authorized: i64,
    pub authorization_id: String,
    pub amount: i64,
    pub merchant_entity_id: String,
    pub details: Vec<DummyLineItem>,
  }

  #[test]
  fn test_encrypt_and_hash() {
    let data = DummyReceipt {
      merchant_entity_id: "Amazon".into(),
      authorized: 4545,
      amount: 3445,
      authorization_id: "foo".into(),
      details: vec![],
    };

    let bytekey = Aes256GcmSiv::generate_key(&mut OsRng);
    let registration_hash = generate_hash(&data);
    let envelope = super::encrypt_envelope(&data, &bytekey.to_vec());

    let cipher = Aes256GcmSiv::new(&bytekey);
    let decrypted = cipher
      .decrypt(
        BASE64_STANDARD.decode(envelope.nonce).unwrap()[..].into(),
        Payload::from(&BASE64_STANDARD.decode(envelope.encrypted).unwrap()[..]),
      )
      .expect("Decryption works");
    assert_eq!(decrypted, "{\"amount\":3445,\"authorization_id\":\"foo\",\"authorized\":4545,\"details\":[],\"merchant_entity_id\":\"Amazon\"}".as_bytes());
    let canonical_json = String::from_utf8(decrypted).expect("Works");
    let deserialized =
      serde_json::from_str::<DummyReceipt>(&canonical_json).expect("Deserialization should work");
    assert_eq!(deserialized.merchant_entity_id, "Amazon".to_string());

    let recalculated_hash = calculate_hash(&canonical_json);
    assert_eq!(recalculated_hash, registration_hash);
  }
}

fn canonize_json(serde_json_val: &serde_json::Value) -> String {
  match json_canon::to_string(serde_json_val) {
    Ok(canonicalized) => canonicalized,
    Err(e) => panic!("Error canonicalizing JSON: {}", e),
  }
}

#[cfg(test)]
mod encryption_tests {
  use pretty_assertions::assert_eq;
  use serde_json::json;

  #[test]
  fn test_canonize_json() {
    let json = json!({
        "foo": "bar",
        "baz": 123,
        "qux": null
    });

    let canonicalized = super::canonize_json(&json);
    assert_eq!(canonicalized, "{\"baz\":123,\"foo\":\"bar\",\"qux\":null}");
  }
}

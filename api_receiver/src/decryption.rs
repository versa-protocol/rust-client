// TODO: move into SDK

use aes_gcm_siv::{
  aead::{Aead, KeyInit, Payload},
  Aes256GcmSiv,
};
use base64::prelude::*;
use serde::Deserialize;

use tracing::info;
use versa::protocol::Envelope;

use crate::report_misuse::MisuseCode;

pub fn decrypt_envelope<T>(envelope: Envelope, key: &String) -> Result<T, MisuseCode>
where
  T: for<'a> Deserialize<'a>,
{
  let encrypted_data = BASE64_STANDARD.decode(envelope.encrypted).unwrap();
  let nonce = BASE64_STANDARD.decode(envelope.nonce).unwrap();
  let key = BASE64_STANDARD.decode(key).unwrap();
  let cipher = Aes256GcmSiv::new(key[..].into());
  let decrypted = match cipher.decrypt(nonce[..].into(), Payload::from(&encrypted_data[..])) {
    Ok(decrypted) => decrypted,
    Err(e) => {
      info!("Failed to decrypt envelope: {:?}", e);
      return Err(MisuseCode::ProtocolDecryptionFailed);
    }
  };
  let canonical_json = String::from_utf8(decrypted).expect("Works");
  match serde_json::from_str::<T>(&canonical_json) {
    Ok(data) => Ok(data),
    Err(_) => Err(MisuseCode::ProtocolDeserializationFailed),
  }
}

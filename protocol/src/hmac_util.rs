use base64::prelude::*;
use hmac::Mac;

pub async fn generate_token(body: bytes::Bytes, secret: String) -> String {
  let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(&secret.as_bytes()).unwrap();
  mac.update(body.as_ref());
  let code_bytes = mac.finalize().into_bytes();
  let encoded = BASE64_STANDARD.encode(&code_bytes.to_vec());
  encoded
}

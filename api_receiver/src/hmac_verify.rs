use base64::prelude::*;
use hmac::Mac;

pub async fn verify_with_secret(
  body: axum::body::Body,
  secret: String,
  token: &str,
) -> (bool, hyper::body::Bytes) {
  let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(&secret.as_bytes()).unwrap();
  let body_bytes = axum::body::to_bytes(body, 512_000_000).await.unwrap();
  mac.update(body_bytes.as_ref());
  let code_bytes = mac.finalize().into_bytes();
  let encoded = BASE64_STANDARD.encode(&code_bytes.to_vec());
  (encoded == token, body_bytes)
}

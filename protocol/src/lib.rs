// TODO: This should likely be replaced by a public protocol crate

use base64::prelude::*;
use hmac::Mac;
use serde::{Deserialize, Serialize};

pub mod encryption;
pub mod model;

use model::{Envelope, Receiver};

use versa::protocol::TransactionHandles;

use tracing::info;

#[derive(Serialize)]
pub struct ReceiptRegistrationRequest {
  pub schema_version: String,
  pub handles: TransactionHandles,
  pub transaction_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryEnv {
  Prod,
  Test,
}

#[derive(Deserialize)]
pub struct ReceiptRegistrationResponse {
  pub env: RegistryEnv,
  pub receipt_id: String,
  pub transaction_id: String,
  pub receivers: Vec<Receiver>,
  pub encryption_key: String,
}

#[derive(Deserialize)]
pub struct DryRunReceiver {
  pub client_id: String,
  pub org_id: String,
}

#[derive(Deserialize)]
pub struct DryRunReceiversResponse {
  pub env: RegistryEnv,
  pub receivers: Vec<DryRunReceiver>,
}

pub async fn dryrun(
  client_id: &str,
  client_secret: &str,
  handles: TransactionHandles,
) -> Result<DryRunReceiversResponse, ()> {
  let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
  let credential = format!("Basic {}:{}", client_id, client_secret);

  let payload_json = serde_json::to_string(&handles).unwrap();

  let url = format!("{}/regcheck", registry_url);
  info!("Sending dryrun registration request to: {}", url);
  let client = reqwest::Client::new();
  let response_result = client
    .post(url)
    .header("Accept", "application/json")
    .header("Authorization", credential)
    .header("Content-Type", "application/json")
    .body(payload_json)
    .send()
    .await;

  let res = match response_result {
    Ok(res) => res,
    Err(e) => {
      info!("Error placing request: {:?}", e);
      return Err(());
    }
  };
  info!("Registration response received");

  if res.status().is_success() {
    let data: DryRunReceiversResponse = match res.json().await {
      Ok(val) => val,
      Err(e) => {
        info!("Failed to deserialize due to error: {}", e);
        return Err(());
      }
    };
    return Ok(data);
  } else {
    info!("Received error status from registry: {}", res.status());
  }

  return Err(());
}

pub async fn register(
  client_id: &str,
  client_secret: &str,
  handles: TransactionHandles,
) -> Result<ReceiptRegistrationResponse, ()> {
  let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
  let credential = format!("Basic {}:{}", client_id, client_secret);

  let payload = ReceiptRegistrationRequest {
    schema_version: "1.2.0".into(),
    handles,
    transaction_id: None,
  };

  let payload_json = serde_json::to_string(&payload).unwrap();

  let url = format!("{}/register", registry_url);
  info!("Sending registration request to: {}", url);
  let client = reqwest::Client::new();
  let response_result = client
    .post(url)
    .header("Accept", "application/json")
    .header("Authorization", credential)
    .header("Content-Type", "application/json")
    .body(payload_json)
    .send()
    .await;

  let res = match response_result {
    Ok(res) => res,
    Err(e) => {
      info!("Error placing request: {:?}", e);
      return Err(());
    }
  };
  info!("Registration response received");

  if res.status().is_success() {
    let data: ReceiptRegistrationResponse = match res.json().await {
      Ok(val) => val,
      Err(e) => {
        info!("Failed to deserialize due to error: {}", e);
        return Err(());
      }
    };
    return Ok(data);
  } else {
    info!("Received error status from registry: {}", res.status());
  }

  return Err(());
}

#[derive(Deserialize, Serialize)]
pub struct ReceiverPayload {
  pub sender_client_id: String,
  pub receipt_id: String,
  pub envelope: Envelope,
}

pub async fn generate_token(body: bytes::Bytes, secret: String) -> String {
  let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(&secret.as_bytes()).unwrap();
  mac.update(body.as_ref());
  let code_bytes = mac.finalize().into_bytes();
  let encoded = BASE64_STANDARD.encode(&code_bytes.to_vec());
  encoded
}

pub async fn encrypt_and_send<T>(
  receiver: &Receiver,
  client_id: &str,
  receipt_id: String,
  encryption_key: String,
  data: T,
) -> Result<(), ()>
where
  T: Serialize,
{
  let envelope =
    encryption::encrypt_envelope(&data, &BASE64_STANDARD.decode(encryption_key).unwrap());

  let payload = ReceiverPayload {
    sender_client_id: client_id.to_string(),
    receipt_id,
    envelope,
  };

  let payload_json = serde_json::to_string(&payload).unwrap();
  let byte_body = bytes::Bytes::from(payload_json.clone());
  let token = generate_token(byte_body, receiver.secret.clone()).await;
  let client = reqwest::Client::new();
  let response_result = client
    .post(&receiver.address)
    .header("Content-Type", "application/json")
    .header("X-Request-Signature", token)
    .body(payload_json)
    .send()
    .await;

  let res = match response_result {
    Ok(res) => res,
    Err(e) => {
      info!("Error placing request: {:?}", e);
      return Err(());
    }
  };

  if res.status().is_success() {
    info!("Successfully sent data to receiver: {}", receiver.address);
    // TODO: process response from each receiver
    return Ok(());
  } else {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    info!("Received an error from the receiver: {} {}", status, text);
  }
  // info!("Received an error from the receiver: {:?}", res);

  return Err(());
}

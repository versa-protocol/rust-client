use serde::{Deserialize, Serialize};

pub mod customer_registration;
pub mod hmac_util;
pub mod model;

use versa::protocol::{Receiver, TransactionHandles};

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

pub async fn check_registry(
  client_id: &str,
  client_secret: &str,
  handles: TransactionHandles,
) -> Result<DryRunReceiversResponse, ()> {
  let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
  let credential = format!("Basic {}:{}", client_id, client_secret);

  let payload_json = serde_json::to_string(&handles).unwrap();

  let url = format!("{}/check_registry", registry_url);
  info!("Sending check_registry registration request to: {}", url);
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

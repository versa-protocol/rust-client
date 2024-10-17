use serde::Serialize;
use tracing::info;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MisuseCode {
  /// Decrypting the received receipt JSON failed
  ProtocolDecryptionFailed,
  /// Decryption succeeded, but the payload was not valid JSON
  ProtocolDeserializationFailed,
  /// The schema version provided is invalid, or does not exist
  SchemaVersionInvalid,
  /// Schema validation of the receipt based on its provided schema version failed
  SchemaValidationFailed,
}

#[derive(Debug, Serialize)]
pub struct ReportMisuseRequest {
  pub receipt_id: String,
  pub misuse_code: MisuseCode,
}

pub async fn send(
  client_id: &str,
  client_secret: &str,
  receipt_id: String,
  misuse_code: MisuseCode,
) -> Result<(), ()> {
  let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
  let credential = format!("Basic {}:{}", client_id, client_secret);

  let payload = ReportMisuseRequest {
    receipt_id,
    misuse_code,
  };

  let payload_json = serde_json::to_string(&payload).unwrap();

  let client = reqwest::Client::new();
  let endpoint_url = format!("{}/report_misuse", registry_url);
  info!("Sending report_misuse request to: {}", endpoint_url);
  let response_result = client
    .post(endpoint_url)
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

  match res.status().is_success() {
    true => Ok(()),
    false => Err(()),
  }
}
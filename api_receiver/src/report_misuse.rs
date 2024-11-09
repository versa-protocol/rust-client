use tracing::info;
use versa::protocol::misuse::{MisuseCode, ReportMisuseRequest};

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

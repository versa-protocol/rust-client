use axum::extract::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use versa::protocol::TransactionHandles;

use tracing::info;

#[derive(Deserialize)]
pub struct SendRequestPayload {
  pub receipt: Option<Value>,
  pub schema_version: String,
  pub handles: TransactionHandles,
}

pub async fn send(
  Json(payload): Json<SendRequestPayload>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();

  let Some(receipt) = payload.receipt else {
    return Err((
      http::StatusCode::BAD_REQUEST,
      "A receipt must be provided to the sending target".to_string(),
    ));
  };

  // 1. Register with Versa registry

  let registration_response = protocol::register(&client_id, &client_secret, payload.handles)
    .await
    .map_err(|e| {
      info!("Registration failed: {:?}", e);
      (
        http::StatusCode::SERVICE_UNAVAILABLE,
        format!("Registration failed: {:?}", e),
      )
    })?;

  info!(
    "Registration successful, received {} receivers",
    registration_response.receivers.len()
  );

  // 2 and 3. Encrypt and send to each receiver

  for receiver in registration_response.receivers {
    info!(
      "Encrypting and sending envelope to receiver {} at {}",
      receiver.org_id, receiver.address
    );
    match protocol::encrypt_and_send(
      &receiver,
      &client_id,
      registration_response.receipt_id.clone(),
      registration_response.encryption_key.clone(),
      &receipt,
    )
    .await
    {
      Ok(_) => info!("Successfully sent to receiver: {}", receiver.address),
      Err(e) => {
        info!("Failed to send to receiver: {:?}", e)
      }
    }
  }

  Ok(http::StatusCode::OK)
}

#[derive(Serialize)]
pub struct DryRunResponse {
  pub has_receivers: bool,
}

pub async fn regcheck(
  Json(payload): Json<SendRequestPayload>,
) -> Result<axum::Json<DryRunResponse>, (axum::http::StatusCode, String)> {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();

  let registration_response = protocol::dryrun(&client_id, &client_secret, payload.handles)
    .await
    .map_err(|e| {
      info!("Registration dryrun failed: {:?}", e);
      (
        http::StatusCode::SERVICE_UNAVAILABLE,
        format!("Registration dryrun failed: {:?}", e),
      )
    })?;

  info!(
    "Registration dryrun successful, received {} receivers",
    registration_response.receivers.len()
  );

  let has_receivers = registration_response.receivers.len() > 0;

  Ok(axum::Json(DryRunResponse { has_receivers }))
}

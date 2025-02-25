use axum::extract::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use versa::{
  client_sender::VersaSender,
  protocol::{customer_registration::HandleType, TransactionHandles},
};

use tracing::info;

#[derive(Deserialize)]
pub struct SendRequestPayload {
  pub receipt: Option<Value>,
  pub schema_version: String,
  pub handles: TransactionHandles,
  pub transaction_id: Option<String>,
}

#[derive(Serialize)]
pub struct SendReceiptResponse {
  pub receipt_id: String,
  pub transaction_id: String,
}

pub async fn send(
  Json(payload): Json<SendRequestPayload>,
) -> Result<axum::Json<SendReceiptResponse>, (axum::http::StatusCode, String)> {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();

  let Some(receipt) = payload.receipt else {
    return Err((
      http::StatusCode::BAD_REQUEST,
      "A receipt must be provided to the sending target".to_string(),
    ));
  };

  let versa_client = versa::client::VersaClient::new(client_id, client_secret)
    .with_client_string(&util::get_client_string())
    .sending_client(payload.schema_version);

  // 1. Register with Versa registry

  let registration_response = versa_client
    .register_receipt(payload.handles, payload.transaction_id)
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

  let (encryption_key, summary, receivers) = registration_response.ready_for_delivery();

  // 2 and 3. Encrypt and send to each receiver

  for receiver in receivers {
    info!(
      "Encrypting and sending envelope to receiver {} at {}",
      receiver.org_id, receiver.address
    );
    match versa_client
      .encrypt_and_send(&receiver, summary.clone(), encryption_key.clone(), &receipt)
      .await
    {
      Ok(_) => info!("Successfully sent to receiver: {}", receiver.address),
      Err(e) => {
        info!("Failed to send to receiver: {:?}", e)
      }
    }
  }

  let receipt_id = summary.receipt_id;
  let transaction_id = summary.transaction_id;

  Ok(axum::Json(SendReceiptResponse {
    receipt_id,
    transaction_id,
  }))
}

#[derive(Serialize)]
pub struct DryRunResponse {
  pub has_receivers: bool,
}

pub async fn check_registry(
  Json(payload): Json<SendRequestPayload>,
) -> Result<axum::Json<DryRunResponse>, (axum::http::StatusCode, String)> {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();

  let registration_response = protocol::check_registry(&client_id, &client_secret, payload.handles)
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

#[derive(Deserialize)]
pub struct SenderCustomerReference {
  pub handle: String,
  pub handle_type: HandleType,
  pub receiver_client_id: String,
}

pub async fn register_customer(Json(payload): Json<SenderCustomerReference>) -> http::StatusCode {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();

  let SenderCustomerReference {
    handle,
    handle_type,
    receiver_client_id,
  } = payload;

  let versa_client = versa::client::VersaClient::new(client_id, client_secret)
    .with_client_string(&util::get_client_string())
    .sending_client("1.8.0".into());

  match protocol::customer_registration::register_customer(
    versa_client,
    handle,
    handle_type,
    Some(receiver_client_id),
  )
  .await
  {
    Ok(_) => http::StatusCode::OK,
    Err(_) => http::StatusCode::SERVICE_UNAVAILABLE,
  }
}

pub async fn deregister_customer(Json(payload): Json<SenderCustomerReference>) -> http::StatusCode {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();

  let SenderCustomerReference {
    handle,
    handle_type,
    receiver_client_id,
  } = payload;

  let versa_client = versa::client::VersaClient::new(client_id, client_secret)
    .with_client_string(&util::get_client_string())
    .sending_client("1.8.0".into());

  match protocol::customer_registration::deregister_customer(
    versa_client,
    handle,
    handle_type,
    Some(receiver_client_id),
  )
  .await
  {
    Ok(_) => http::StatusCode::OK,
    Err(_) => http::StatusCode::SERVICE_UNAVAILABLE,
  }
}

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

  let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
  let Ok(versa_client) = versa::client::VersaClient::new(registry_url, client_id, client_secret)
    .sending_client("1.5.1".into())
  else {
    return Err((
      http::StatusCode::INTERNAL_SERVER_ERROR,
      "Failed to create Versa client".to_string(),
    ));
  };

  // 1. Register with Versa registry

  let registration_response = versa_client
    .register_receipt(payload.handles, None)
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
      &versa_client.client_id(),
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

pub async fn check_registry(
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

  let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
  let Ok(versa_client) = versa::client::VersaClient::new(registry_url, client_id, client_secret)
    .sending_client("1.5.0".into())
  else {
    return http::StatusCode::INTERNAL_SERVER_ERROR;
  };

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

  let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
  let Ok(versa_client) = versa::client::VersaClient::new(registry_url, client_id, client_secret)
    .sending_client("1.5.0".into())
  else {
    return http::StatusCode::INTERNAL_SERVER_ERROR;
  };

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

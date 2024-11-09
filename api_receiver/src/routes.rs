use http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use versa::{
  client_receiver::VersaReceiver,
  protocol::{ReceiverPayload, Sender, TransactionHandles},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct DecryptedPayload {
  pub handles: TransactionHandles,
  pub receipt_id: String,
  pub receipt: serde_json::Value,
  pub receiver_client_id: String,
  pub sender_client_id: String,
  pub sender: Option<Sender>,
  pub transaction_id: String,
}

pub async fn target(
  headers: HeaderMap,
  raw_body: axum::body::Body,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
  let (receiver_client_id, receiver_client_secret) = util::get_client_id_and_client_secret();
  let receiver_secret = crate::r_config::get_receiver_secret();

  let Some(request_signature) = headers.get("X-Request-Signature") else {
    return Err((
      http::StatusCode::BAD_REQUEST,
      "Missing X-Request-Signature header".to_string(),
    ));
  };
  let Ok(request_token) = request_signature.to_str() else {
    return Err((
      http::StatusCode::BAD_REQUEST,
      "Malformed X-Request-Signature header".to_string(),
    ));
  };
  let (verified, body_bytes) =
    crate::hmac_verify::verify_with_secret(raw_body, receiver_secret.clone(), request_token).await;
  if !verified {
    return Err((
      http::StatusCode::UNAUTHORIZED,
      "Failed to verify request signature".to_string(),
    ));
  }
  info!("Successfully verified hmac request signature");
  let body: ReceiverPayload = match serde_json::from_slice(&body_bytes) {
    Ok(val) => val,
    Err(e) => {
      return Err((
        http::StatusCode::BAD_REQUEST,
        format!("Failed to parse body: {}", e),
      ));
    }
  };

  let ReceiverPayload {
    sender_client_id,
    receipt_id,
    envelope,
  } = body;

  info!("Received envelope from sender={}", sender_client_id);
  info!("Checking out key for receipt_id={}", receipt_id);

  let versa_client =
    versa::client::VersaClient::new(receiver_client_id.clone(), receiver_client_secret.clone())
      .receiving_client(receiver_secret);
  let checkout = versa_client.checkout_key(receipt_id).await.map_err(|_| {
    (
      http::StatusCode::INTERNAL_SERVER_ERROR,
      "Failed to checkout key".to_string(),
    )
  })?;

  info!("Received keys for sender: {:?}", checkout.sender);
  let data = match versa_client.decrypt_envelope::<Value>(envelope, checkout.key) {
    Ok(val) => val,
    Err(misuse_code) => {
      crate::report_misuse::send(
        &receiver_client_id,
        &receiver_client_secret,
        checkout.receipt_id,
        misuse_code.clone(),
      )
      .await
      .expect("Reporting misuse failed");
      return Err((
        http::StatusCode::BAD_REQUEST,
        format!("Failed to decrypt envelope: {:?}", misuse_code),
      ));
    }
  };

  info!(
    "DATA RECEIVED FROM SENDER_CLIENT_ID={}: {:?}",
    sender_client_id,
    serde_json::to_string(&data).unwrap()
  );

  match crate::schema::validate(&data, &checkout.schema_version).await {
    Ok(val) => val,
    Err(misuse_code) => {
      crate::report_misuse::send(
        &receiver_client_id,
        &receiver_client_secret,
        checkout.receipt_id.clone(),
        misuse_code.clone(),
      )
      .await
      .expect("Reporting misuse failed");
      info!("WARN: Failed to validate receipt data: {:?}", misuse_code);
    }
  };

  let payload = DecryptedPayload {
    sender_client_id,
    receipt_id: checkout.receipt_id,
    transaction_id: checkout.transaction_id,
    receiver_client_id,
    handles: checkout.handles,
    sender: checkout.sender,
    receipt: data,
  };

  ///////////////////////////////////////////////////////////////////////////////////////////////////
  //
  // TODO: Logic to handle the decrypted payload, whether storing or forwarding it to another service
  //
  ///////////////////////////////////////////////////////////////////////////////////////////////////

  info!(
    "Successfully received data over the Versa network: {:?}",
    payload
  );

  Ok(http::StatusCode::OK)
}

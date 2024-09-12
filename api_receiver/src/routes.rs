use crate::r_protocol::DecryptedPayload;
use http::HeaderMap;
use protocol::ReceiverPayload;
use serde_json::Value;
use tracing::info;

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

  let checkout =
    crate::r_protocol::checkout_key(&receiver_client_id, &receiver_client_secret, receipt_id)
      .await
      .map_err(|_| {
        (
          http::StatusCode::INTERNAL_SERVER_ERROR,
          "Failed to checkout key".to_string(),
        )
      })?;

  info!("Received keys for sender: {:?}", checkout.sender);
  let data = crate::decryption::decrypt_envelope::<Value>(envelope, &checkout.key);

  info!(
    "DATA RECEIVED FROM SENDER_CLIENT_ID={}: {:?}",
    sender_client_id,
    serde_json::to_string(&data).unwrap()
  );

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

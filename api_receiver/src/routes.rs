use http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use versa::{
  client_receiver::VersaReceiver,
  protocol::{
    customer_registration::HandleType, ReceiverPayload, Sender, TransactionHandles, WebhookEvent,
    WebhookEventType,
  },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct DecryptedPayload {
  pub handles: TransactionHandles,
  pub receipt_id: String,
  pub receipt: serde_json::Value,
  pub receiver_client_id: String,
  pub schema_version: String,
  pub sender_client_id: String,
  pub sender: Option<Sender>,
  pub transaction_id: String,
}

pub async fn target(
  headers: HeaderMap,
  raw_body: axum::body::Body,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
  let (receiver_client_id, receiver_client_secret) = util::get_client_id_and_client_secret();
  let receiver_secret = crate::r_config::get_webhook_secret();

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
  let body: WebhookEvent<ReceiverPayload> = match serde_json::from_slice(&body_bytes) {
    Ok(val) => val,
    Err(e) => {
      return Err((
        http::StatusCode::BAD_REQUEST,
        format!("Failed to parse body: {}", e),
      ));
    }
  };

  let event = body.event;
  if event != WebhookEventType::Receipt && event != WebhookEventType::Itinerary {
    info!(
      "WARN: Received event other than 'receipt' or 'itinerary': {}",
      event
    );
  }

  let ReceiverPayload {
    sender_client_id,
    receipt_id,
    envelope,
  } = body.data;

  info!("Received envelope from sender={}", sender_client_id);
  info!("Checking out key for receipt_id={}", receipt_id);

  let versa_client =
    versa::client::VersaClient::new(receiver_client_id.clone(), receiver_client_secret.clone())
      .with_client_string(&util::get_client_string())
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

  match crate::schema::validate(&event, &data).await {
    Ok(val) => val,
    Err((misuse_code, msg)) => {
      info!("WARN: Schema validation failed: {}", msg);
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
    handles: checkout.handles,
    receipt_id: checkout.receipt_id,
    receipt: data,
    receiver_client_id,
    schema_version: checkout.schema_version,
    sender_client_id,
    sender: checkout.sender,
    transaction_id: checkout.transaction_id,
  };

  info!(
    "Successfully received data over the Versa network: {:?}",
    payload
  );

  match std::env::var("LOCAL_TARGET_URL") {
    Ok(url) => {
      // send to url
      let client = reqwest::Client::new();
      let res = client.post(url).json(&payload).send().await.map_err(|e| {
        (
          http::StatusCode::INTERNAL_SERVER_ERROR,
          format!("Failed to send data to local target: {:?}", e),
        )
      })?;

      if res.status().is_success() {
        info!("Successfully sent data to local target");
      } else {
        info!("Failed to send data to local target: {:?}", res);
      }
    }
    Err(_) => {
      info!("WARN: LOCAL_TARGET_URL not set, data not sent to local endpoint");
    }
  }

  Ok(http::StatusCode::OK)
}

#[derive(Deserialize)]
pub struct ReceiverCustomerReference {
  pub handle: String,
  pub handle_type: HandleType,
}

pub async fn register_customer(
  axum::extract::Json(payload): axum::extract::Json<ReceiverCustomerReference>,
) -> http::StatusCode {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();
  let receiver_secret = crate::r_config::get_webhook_secret();

  let ReceiverCustomerReference {
    handle,
    handle_type,
  } = payload;

  let versa_client = versa::client::VersaClient::new(client_id, client_secret)
    .with_client_string(&util::get_client_string())
    .receiving_client(receiver_secret);

  match protocol::customer_registration::register_customer(versa_client, handle, handle_type, None)
    .await
  {
    Ok(_) => http::StatusCode::OK,
    Err(_) => http::StatusCode::SERVICE_UNAVAILABLE,
  }
}

pub async fn deregister_customer(
  axum::extract::Json(payload): axum::extract::Json<ReceiverCustomerReference>,
) -> http::StatusCode {
  let (client_id, client_secret) = util::get_client_id_and_client_secret();
  let receiver_secret = crate::r_config::get_webhook_secret();

  let ReceiverCustomerReference {
    handle,
    handle_type,
  } = payload;

  let versa_client = versa::client::VersaClient::new(client_id, client_secret)
    .with_client_string(&util::get_client_string())
    .receiving_client(receiver_secret);

  match protocol::customer_registration::deregister_customer(
    versa_client,
    handle,
    handle_type,
    None,
  )
  .await
  {
    Ok(_) => http::StatusCode::OK,
    Err(_) => http::StatusCode::SERVICE_UNAVAILABLE,
  }
}

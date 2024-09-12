// use data_access::model::{receiver::Receiver, transaction::Transaction};
use serde::Serialize;
use std::{env, time::SystemTime};

#[derive(Serialize)]
pub struct ServiceInfo {
  pub service_name: String,
  pub service_version: String,
  pub service_env: String,
  pub system_time: SystemTime,
  pub timestamp_utc: i64,
}

pub async fn service_info() -> axum::Json<ServiceInfo> {
  let service_name = env!("CARGO_PKG_NAME").to_string();
  let service_version = env!("CARGO_PKG_VERSION").to_string();
  let service_env = std::env::var("VERSA_ENV").unwrap_or("development".to_string());
  let system_time = SystemTime::now();
  let timestamp_utc = system_time
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;
  let service_info = ServiceInfo {
    service_name,
    service_version,
    service_env,
    system_time,
    timestamp_utc,
  };
  axum::Json(service_info)
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Merchant {
  pub id: String,
  pub name: String,
  pub brand_color: String,
  pub logo: String,
  pub mcc: String,
  pub website: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Envelope {
  pub encrypted: String,
  pub nonce: String,
}

#[derive(Debug, Serialize)]
pub struct RegistrationData {
  pub hash: Option<u64>,
  pub key: String,
}

#[derive(Serialize, Debug, Default)]
pub struct RoutingInfo {
  pub customer_email: Option<String>,
  pub authorization_bin: Option<String>,
  pub authorization_par: Option<String>,
}

#[derive(Deserialize)]
pub struct Receiver {
  pub address: String,
  pub client_id: String,
  pub org_id: String,
  pub secret: String,
}

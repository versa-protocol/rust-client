// TODO: move into SDK

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ThirdParty {
  pub first_party_relation: String,
  pub make_primary: bool,
  pub merchant: Merchant,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SenderReceiptHeader {
  pub id: String,
  pub currency: String,
  pub amount: i64,
  pub subtotal: i64,
  pub date_time: i64,
  pub sender_client_id: String,
  pub third_party: Option<ThirdParty>,
}

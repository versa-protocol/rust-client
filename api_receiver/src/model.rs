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

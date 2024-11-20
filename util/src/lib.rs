pub fn get_client_id_and_client_secret() -> (String, String) {
  let client_id = std::env::var("VERSA_CLIENT_ID").expect("VERSA_CLIENT_ID must be set");
  let client_secret =
    std::env::var("VERSA_CLIENT_SECRET").expect("VERSA_CLIENT_SECRET must be set");
  (client_id, client_secret)
}

pub fn get_client_string() -> String {
  format!(
    "rust-client-official/{}/{}",
    env!("CARGO_PKG_VERSION").to_string(),
    std::env::var("IMAGE_VERSION").unwrap_or("".into())
  )
}

pub fn get_client_id_and_client_secret() -> (String, String) {
  let client_id = std::env::var("VERSA_CLIENT_ID").expect("VERSA_CLIENT_ID must be set");
  let client_secret =
    std::env::var("VERSA_CLIENT_SECRET").expect("VERSA_CLIENT_SECRET must be set");
  (client_id, client_secret)
}

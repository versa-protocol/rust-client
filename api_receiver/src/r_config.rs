pub fn get_receiver_secret() -> String {
  std::env::var("VERSA_RECEIVER_SECRET").expect("VERSA_RECEIVER_SECRET must be set")
}

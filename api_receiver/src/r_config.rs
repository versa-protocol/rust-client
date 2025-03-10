pub fn get_webhook_secret() -> String {
  std::env::var("VERSA_WEBHOOK_SECRET").expect("VERSA_WEBHOOK_SECRET must be set")
}

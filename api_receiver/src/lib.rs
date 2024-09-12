use axum::routing::post;
use axum::Router;

pub mod routes;

// TODO - move into SDK
mod decryption;
mod hmac_verify;
mod model;
mod r_config;
mod r_protocol;

pub fn configure() -> Router {
  Router::new().route("/target", post(routes::target))
}

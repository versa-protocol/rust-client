use axum::routing::post;
use axum::Router;

pub mod routes;

mod hmac_verify;
mod model;
mod r_config;
mod report_misuse; // move to SDK

mod schema; // move to SDK

pub fn configure() -> Router {
  Router::new().route("/target", post(routes::target))
}

use axum::routing::{delete, post};
use axum::Router;

pub mod routes;

mod hmac_verify;
mod model;
mod r_config;
mod report_misuse; // move to SDK

mod schema; // move to SDK

pub fn configure() -> Router {
  Router::new()
    .route("/customer", delete(routes::deregister_customer))
    .route("/customer", post(routes::register_customer))
    .route("/target", post(routes::target))
}

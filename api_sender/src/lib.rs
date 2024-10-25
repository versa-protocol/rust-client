use axum::routing::{delete, post};
use axum::Router;

pub mod routes;

pub fn configure() -> Router {
  Router::new()
    .route("/handle", delete(routes::deregister_customer))
    .route("/handle", post(routes::register_customer))
    .route("/check_registry", post(routes::check_registry))
    .route("/send", post(routes::send))
}

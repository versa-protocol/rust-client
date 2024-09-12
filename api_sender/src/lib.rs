use axum::routing::post;
use axum::Router;

pub mod routes;

pub fn configure() -> Router {
  Router::new()
    .route("/regcheck", post(routes::regcheck))
    .route("/send", post(routes::send))
}

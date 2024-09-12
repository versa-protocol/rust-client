use axum::body::Body;
use axum::routing::get;
use axum::Router;
use http::Request;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::{info, info_span, Level};

mod middleware;
mod service_info;

#[tokio::main]
async fn main() {
  dotenv::dotenv().ok();
  tracing_subscriber::fmt().with_max_level(Level::INFO).init();

  let mut app = Router::new().route("/", get(service_info::service_info));

  #[cfg(feature = "receiver")]
  {
    let receiver_routes = api_receiver::configure();
    app = app.nest("/receiver", receiver_routes);
  }
  #[cfg(feature = "sender")]
  {
    let sender_routes = api_sender::configure();
    app = app.nest("/sender", sender_routes);
  }

  app = app
    .layer(axum::middleware::from_fn(middleware::log_request))
    .layer(
      TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        // Get request id from the extensions...
        let request_id = request
          .extensions()
          .get::<RequestId>()
          .map(ToString::to_string)
          .unwrap_or_else(|| "unknown".into());
        // ...and add it into info span
        info_span!(
            "request",
            id = %request_id,
            method = %request.method(),
            uri = %request.uri(),
        )
      }),
    )
    // This layer creates a new id for each request and puts it into the request extensions.
    // Note that it should be added after the Trace layer.
    .layer(RequestIdLayer);

  info!("Listening on port 8080");

  let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
  axum::serve(listener, app.into_make_service())
    .await
    .unwrap();
}

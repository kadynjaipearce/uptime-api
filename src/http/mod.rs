use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    Router,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    routing::get,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{config::Config, core::AppState};

mod health;

/// Assembles the HTTP surface: routes, CORS, tracing. Handlers in this
/// module do request/response translation only — extract, call `core`, wrap
/// the result in `ApiResponse`/`AppError`. No business logic lives here.
pub fn router(config: &Config, state: Arc<AppState>) -> Result<Router> {
    let frontend_origin = config
        .frontend_url
        .parse::<HeaderValue>()
        .context("FRONTEND_URL must be a valid origin")?;
    let cors = CorsLayer::new()
        .allow_origin(frontend_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    Ok(Router::new()
        .route("/health", get(health::health))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}

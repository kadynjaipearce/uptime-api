use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    Router,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    routing::{delete, get, patch, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{config::Config, core::AppState};

pub mod check;
pub mod content_snapshot;
pub mod incident;
pub mod url;

mod health;

/// Assembles the HTTP surface: routes, CORS, tracing. Handlers in this
pub fn router(config: &Config, state: Arc<AppState>) -> Result<Router> {
    let frontend_origin = config
        .frontend_url
        .parse::<HeaderValue>()
        .context("FRONTEND_URL must be a valid origin")?;
    let cors = CorsLayer::new()
        .allow_origin(frontend_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    let merged_router = Router::new()
        .route("/health", get(health::health))
        .route("/url", post(url::create_url))
        .route(
            "/url/{id}",
            get(url::get_url)
                .patch(url::update_url)
                .delete(url::delete_url),
        )
        .route("/url/{id}/incidents", get(incident::get_incident))
        .route("/url/{id}/checks", get(check::get_check_history));

    Ok(Router::new()
        .nest(format!("/api/{}/", config.version).as_str(), merged_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}

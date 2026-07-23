use std::sync::Arc;

use axum::{extract::State, http::StatusCode};

use crate::{
    core::{self, AppState, health::HealthStatus},
    response::ApiResponse,
};

pub async fn health(State(state): State<Arc<AppState>>) -> ApiResponse<HealthStatus> {
    let status = core::health::status(&state).await;
    let code = if status.is_healthy() {
        tracing::info!("Health OK");
        StatusCode::OK
    } else {
        tracing::error!("Service Unavailable");
        StatusCode::SERVICE_UNAVAILABLE
    };

    ApiResponse::new(code, status)
}

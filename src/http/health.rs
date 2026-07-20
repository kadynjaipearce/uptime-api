use std::sync::Arc;

use axum::extract::State;

use crate::{core, core::AppState, response::ApiResponse};

pub async fn health(State(state): State<Arc<AppState>>) -> ApiResponse<&'static str> {
    ApiResponse::ok(core::health::status(&state).await)
}

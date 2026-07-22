use axum::extract::State;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::AppState,
    database::{Database, models::url::UrlRow},
    response::ApiResponse,
};

/// Payload for registering a new URL to monitor.
#[derive(Debug, Deserialize)]
pub struct CreateUrl {
    pub domain: String,
    pub name: String,
    pub check_interval_seconds: Option<i32>,
    pub expected_content: Option<String>,
}

/// Payload for partially updating an existing URL. Every field is optional
/// so callers only send what they want to change.
#[derive(Debug, Deserialize)]
pub struct UpdateUrl {
    pub domain: Option<String>,
    pub name: Option<String>,
    pub check_interval_seconds: Option<i32>,
    pub expected_content: Option<String>,
    pub is_active: Option<bool>,
}

/// `url` row as returned to API clients.
#[derive(Debug, Serialize)]
pub struct UrlResponse {
    pub id: Uuid,
    pub domain: String,
    pub name: String,
    pub check_interval_seconds: i32,
    pub expected_content: Option<String>,
    pub is_active: bool,
    pub next_check_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<UrlRow> for UrlResponse {
    fn from(row: UrlRow) -> Self {
        Self {
            id: row.id,
            domain: row.domain,
            name: row.name,
            check_interval_seconds: row.check_interval_seconds,
            expected_content: row.expected_content,
            is_active: row.is_active,
            next_check_at: row.next_check_at,
            created_at: row.created_at,
        }
    }
}

pub async fn create_url(State(state): State<Arc<AppState>>) -> ApiResponse<UrlResponse> {
    unimplemented!();
}

pub async fn get_url(State(state): State<Arc<AppState>>) -> ApiResponse<UrlResponse> {
    unimplemented!()
}

pub async fn update_url(State(state): State<Arc<AppState>>) -> ApiResponse<UrlResponse> {
    unimplemented!()
}

pub async fn delete_url(State(state): State<Arc<AppState>>) -> ApiResponse<UrlResponse> {
    unimplemented!()
}

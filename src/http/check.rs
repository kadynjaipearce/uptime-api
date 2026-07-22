use std::sync::Arc;

use axum::extract::State;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{core::AppState, database::models::check::CheckRow, response::ApiResponse};

/// Result of a single probe, reported by a region worker after it checks a
/// URL. `id` and `checked_at` are assigned server-side.
#[derive(Debug, Deserialize)]
pub struct RecordCheck {
    pub url_id: Uuid,
    pub check_round_id: Uuid,
    pub region: String,
    pub dns_ms: Option<i32>,
    pub connect_ms: Option<i32>,
    pub tls_ms: Option<i32>,
    pub ttfb_ms: Option<i32>,
    pub total_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub success: bool,
    pub error_stage: Option<String>,
    pub error_message: Option<String>,
    pub content_hash: Option<String>,
}

/// `checks` row as returned to API clients.
#[derive(Debug, Serialize)]
pub struct CheckResponse {
    pub id: Uuid,
    pub url_id: Uuid,
    pub check_round_id: Uuid,
    pub region: String,
    pub checked_at: DateTime<Utc>,
    pub dns_ms: Option<i32>,
    pub connect_ms: Option<i32>,
    pub tls_ms: Option<i32>,
    pub ttfb_ms: Option<i32>,
    pub total_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub success: bool,
    pub error_stage: Option<String>,
    pub error_message: Option<String>,
    pub content_hash: Option<String>,
}

impl From<CheckRow> for CheckResponse {
    fn from(row: CheckRow) -> Self {
        Self {
            id: row.id,
            url_id: row.url_id,
            check_round_id: row.check_round_id,
            region: row.region,
            checked_at: row.checked_at,
            dns_ms: row.dns_ms,
            connect_ms: row.connect_ms,
            tls_ms: row.tls_ms,
            ttfb_ms: row.ttfb_ms,
            total_ms: row.total_ms,
            status_code: row.status_code,
            success: row.success,
            error_stage: row.error_stage,
            error_message: row.error_message,
            content_hash: row.content_hash,
        }
    }
}

pub async fn get_check_history(State(state): State<Arc<AppState>>) -> ApiResponse<AppState> {
    unimplemented!()
}

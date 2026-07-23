use std::sync::Arc;

use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    core::AppState, database::models::incident::IncidentRow, error::AppError, response::ApiResponse,
};

/// Opens a new incident for a URL that started failing checks. `started_at`
/// is assigned server-side (now).
#[derive(Debug, Deserialize)]
pub struct OpenIncident {
    pub url_id: Uuid,
    pub cause: Option<String>,
}

/// Marks an open incident resolved. `resolved_at` is assigned server-side
/// (now).
#[derive(Debug, Deserialize)]
pub struct ResolveIncident {
    pub cause: Option<String>,
}

/// `incidents` row as returned to API clients.
#[derive(Debug, Serialize)]
pub struct IncidentResponse {
    pub id: Uuid,
    pub url_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub cause: Option<String>,
}

impl From<IncidentRow> for IncidentResponse {
    fn from(row: IncidentRow) -> Self {
        Self {
            id: row.id,
            url_id: row.url_id,
            started_at: row.started_at,
            resolved_at: row.resolved_at,
            resolved: row.resolved,
            cause: row.cause,
        }
    }
}

pub async fn get_incident(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<IncidentResponse>>, AppError> {
    let rows = state.db.list_open_incidents_for_url(url_id).await?;
    let incidents = rows.into_iter().map(IncidentResponse::from).collect();

    Ok(ApiResponse::ok(incidents))
}

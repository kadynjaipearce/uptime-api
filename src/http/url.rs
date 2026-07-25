use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

use crate::{
    core::AppState, database::models::url::UrlRow, error::AppError, extract::AppJson,
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

pub trait UrlPayload {
    fn domain(&self) -> Option<&str>;
    fn name(&self) -> Option<&str>;
    fn check_interval_seconds(&self) -> Option<i32>;

    fn validate(&self) -> Result<(), AppError> {
        static DOMAIN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,63}$")
                .expect("invalid domain regex")
        });

        if let Some(domain) = self.domain() {
            if domain.trim().is_empty() {
                return Err(AppError::BadRequest("domain must not be empty".into()));
            }

            if !DOMAIN.is_match(domain) {
                return Err(AppError::BadRequest("domain must be valid TLD".into()));
            }
        }
        if let Some(name) = self.name()
            && name.trim().is_empty()
        {
            return Err(AppError::BadRequest("name must not be empty".into()));
        }
        if let Some(secs) = self.check_interval_seconds()
            && secs <= 0
        {
            return Err(AppError::BadRequest(
                "check_interval_seconds must be positive".into(),
            ));
        }
        Ok(())
    }
}

impl UrlPayload for CreateUrl {
    fn domain(&self) -> Option<&str> {
        Some(&self.domain)
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn check_interval_seconds(&self) -> Option<i32> {
        self.check_interval_seconds
    }
}

impl UrlPayload for UpdateUrl {
    fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn check_interval_seconds(&self) -> Option<i32> {
        self.check_interval_seconds
    }
}

pub async fn create_url(
    State(state): State<Arc<AppState>>,
    AppJson(params): AppJson<CreateUrl>,
) -> Result<ApiResponse<UrlResponse>, AppError> {
    params.validate()?;

    let row = state.db.create_url(params).await?;
    Ok(ApiResponse::new(
        StatusCode::CREATED,
        UrlResponse::from(row),
    ))
}

pub async fn get_url(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<Uuid>,
) -> Result<ApiResponse<UrlResponse>, AppError> {
    let row = state.db.get_url(url_id).await?.ok_or(AppError::NotFound)?;
    Ok(ApiResponse::ok(UrlResponse::from(row)))
}

pub async fn update_url(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<Uuid>,
    AppJson(params): AppJson<UpdateUrl>,
) -> Result<ApiResponse<UrlResponse>, AppError> {
    params.validate()?;

    let row = state
        .db
        .update_url(url_id, params)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(ApiResponse::ok(UrlResponse::from(row)))
}

pub async fn delete_url(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<Uuid>,
) -> Result<ApiResponse<()>, AppError> {
    let deleted = state.db.delete_url(url_id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(ApiResponse::new(StatusCode::NO_CONTENT, ()))
}

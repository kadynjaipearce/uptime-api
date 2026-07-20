use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{database::Database, http::check::RecordCheck};

/// Row shape of the `checks` table.
#[derive(Debug, FromRow)]
pub struct CheckRow {
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

impl Database {
    pub async fn record_check(&self, input: RecordCheck) -> sqlx::Result<CheckRow> {
        sqlx::query_as::<_, CheckRow>(
            r#"
            INSERT INTO checks (
                url_id, check_round_id, region, dns_ms, connect_ms, tls_ms,
                ttfb_ms, total_ms, status_code, success, error_stage,
                error_message, content_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, url_id, check_round_id, region, checked_at, dns_ms,
                      connect_ms, tls_ms, ttfb_ms, total_ms, status_code,
                      success, error_stage, error_message, content_hash
            "#,
        )
        .bind(input.url_id)
        .bind(input.check_round_id)
        .bind(input.region)
        .bind(input.dns_ms)
        .bind(input.connect_ms)
        .bind(input.tls_ms)
        .bind(input.ttfb_ms)
        .bind(input.total_ms)
        .bind(input.status_code)
        .bind(input.success)
        .bind(input.error_stage)
        .bind(input.error_message)
        .bind(input.content_hash)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_checks_for_url(&self, url_id: Uuid, limit: i64) -> sqlx::Result<Vec<CheckRow>> {
        sqlx::query_as::<_, CheckRow>(
            "SELECT * FROM checks WHERE url_id = $1 ORDER BY checked_at DESC LIMIT $2",
        )
        .bind(url_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}

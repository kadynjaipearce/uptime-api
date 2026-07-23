use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::database::Database;

/// Row shape of the `jobs` table.
#[derive(Debug, FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub job_type: String,
    pub payload: Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub run_at: DateTime<Utc>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Database {
    /// Enqueues one `check` job for a single region. `check_round_id` ties
    /// every region's job for the same due-URL tick together, matching the
    /// `check_round_id` column region workers report back on `checks`.
    pub async fn enqueue_check_job(
        &self,
        url_id: Uuid,
        check_round_id: Uuid,
        region: &str,
    ) -> sqlx::Result<JobRow> {
        sqlx::query_as::<_, JobRow>(
            r#"
            INSERT INTO jobs (job_type, payload, status)
            VALUES (
                'check',
                jsonb_build_object('url_id', $1, 'check_round_id', $2, 'region', $3),
                'pending'
            )
            RETURNING id, job_type, payload, status, attempts, max_attempts,
                      run_at, claimed_by, claimed_at, error_message, created_at
            "#,
        )
        .bind(url_id)
        .bind(check_round_id)
        .bind(region)
        .fetch_one(&self.pool)
        .await
    }
}

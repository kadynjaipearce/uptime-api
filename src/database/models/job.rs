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

    /// Claims up to `limit` pending jobs for `region`, marking them
    /// `claimed` under `worker_id` so no other worker picks them up.
    pub async fn claim_jobs(
        &self,
        region: &str,
        worker_id: &str,
        limit: i64,
    ) -> sqlx::Result<Vec<JobRow>> {
        let mut tx = self.pool.begin().await?;

        let claimed = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT * FROM jobs
            WHERE status = 'pending'
              AND run_at <= now()
              AND payload->>'region' = $1
            ORDER BY run_at
            LIMIT $2
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(region)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;

        for job in &claimed {
            sqlx::query(
                r#"
                UPDATE jobs
                SET status = 'claimed', claimed_by = $2, claimed_at = now(),
                    attempts = attempts + 1
                WHERE id = $1
                "#,
            )
            .bind(job.id)
            .bind(worker_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(claimed)
    }

    /// Marks a claimed job as done.
    pub async fn complete_job(&self, job_id: Uuid) -> sqlx::Result<()> {
        sqlx::query("UPDATE jobs SET status = 'completed' WHERE id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Records a job failure. Jobs that have exhausted `max_attempts` are
    /// left `failed`; others go back to `pending` so a worker retries them.
    pub async fn fail_job(&self, job_id: Uuid, error_message: &str) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = CASE WHEN attempts >= max_attempts THEN 'failed' ELSE 'pending' END,
                error_message = $2,
                claimed_by = NULL,
                claimed_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .bind(error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

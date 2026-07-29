use std::{net::Ipv4Addr, time::Duration};

use anyhow::Result;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    core::handler::check,
    database::{Database, models::job::JobRow},
    http::check::RecordCheck,
};

/// How many jobs a single poll will claim for this worker's region.
const BATCH_SIZE: i64 = 10;

/// How often the worker polls the `jobs` table.
pub const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Deserialize)]
struct CheckJobPayload {
    url_id: Uuid,
    check_round_id: Uuid,
    region: String,
}

/// Polls the `jobs` table for pending jobs matching this worker's region.
/// dispatches into the DNS -> TCP -> TLS -> HTTP pipeline.
pub struct Worker {
    id: String,
    db: Database,
    region: String,
    dns_servers: Vec<Ipv4Addr>,
    poll_interval: Duration,
}

impl Worker {
    pub fn new(
        db: Database,
        region: String,
        dns_servers: Vec<Ipv4Addr>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            db,
            region,
            dns_servers,
            poll_interval,
        }
    }

    pub async fn run(&self) {
        let mut ticker = tokio::time::interval(self.poll_interval);

        loop {
            ticker.tick().await;

            if let Err(err) = self.poll().await {
                tracing::error!(region = %self.region, "worker poll failed: {err:?}");
            }
        }
    }

    async fn poll(&self) -> Result<()> {
        let jobs = self
            .db
            .claim_jobs(&self.region, &self.id, BATCH_SIZE)
            .await?;

        for job in jobs {
            self.process(job).await;
        }

        Ok(())
    }

    /// Dispatches a claimed job to its handler and records the outcome.
    async fn process(&self, job: JobRow) {
        let outcome = match job.job_type.as_str() {
            "check" => self.process_check(&job).await,
            other => Err(anyhow::anyhow!("unknown job type: {other}")),
        };

        let report = match outcome {
            Ok(()) => self.db.complete_job(job.id).await,
            Err(err) => {
                tracing::error!(job_id = %job.id, "job failed: {err:?}");
                self.db.fail_job(job.id, &err.to_string()).await
            }
        };

        if let Err(err) = report {
            tracing::error!(job_id = %job.id, "failed to record job outcome: {err:?}");
        }
    }

    /// Runs the check pipeline (DNS -> TCP -> TLS -> HTTP) for a `check` job
    /// and stores the result.
    async fn process_check(&self, job: &JobRow) -> Result<()> {
        let payload: CheckJobPayload = serde_json::from_value(job.payload.clone())?;

        let url = self
            .db
            .get_url(payload.url_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("url {} not found", payload.url_id))?;

        let result = check::run_check(
            &url.domain,
            url.expected_content.as_deref(),
            &self.dns_servers,
        )
        .await;

        self.db
            .record_check(RecordCheck {
                url_id: payload.url_id,
                check_round_id: payload.check_round_id,
                region: payload.region,
                dns_ms: result.dns_ms.map(|ms| ms as i32),
                connect_ms: result.connect_ms.map(|ms| ms as i32),
                tls_ms: result.tls_ms.map(|ms| ms as i32),
                ttfb_ms: result.ttfb_ms.map(|ms| ms as i32),
                total_ms: Some(result.total_ms as i32),
                status_code: result.status_code.map(i32::from),
                success: result.success,
                error_stage: result.error_stage.map(|stage| format!("{stage:?}")),
                error_message: result.error_message,
                content_hash: result.content_hash,
            })
            .await?;

        Ok(())
    }
}

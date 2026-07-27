use std::time::Duration;

use anyhow::Result;
use uuid::Uuid;

use crate::database::Database;

/// How many due URLs a single tick will claim.
const BATCH_SIZE: i64 = 100;

/// How often the scheduler checks for due URLs.
pub const TICK_INTERVAL: Duration = Duration::from_secs(5);

/// Polls the `url` table for due checks and fans each one out into one
/// `jobs` row per configured region. Region workers claim jobs matching
/// their own region and report results back via `POST /checks`.
pub struct Scheduler {
    db: Database,
    tick_interval: Duration,
    regions: Vec<String>,
}

impl Scheduler {
    pub fn new(db: Database, tick_interval: Duration, regions: Vec<String>) -> Self {
        Self {
            db,
            tick_interval,
            regions,
        }
    }

    pub async fn run(&self) {
        let mut ticker = tokio::time::interval(self.tick_interval);

        loop {
            ticker.tick().await;

            if let Err(err) = self.tick().await {
                tracing::error!("scheduler tick failed: {err:?}");
            }
        }
    }

    async fn tick(&self) -> Result<()> {
        let due = self.db.claim_due_urls(BATCH_SIZE).await?;

        for url in due {
            self.enqueue_check_round(url.id).await;
            tracing::info!(
                target = "queue event",
                url_id = ?url.id,
                domain = ?url.domain
            )
        }

        Ok(())
    }

    /// Enqueues one job per configured region for a single due URL
    async fn enqueue_check_round(&self, url_id: Uuid) {
        let check_round_id = Uuid::new_v4();

        for region in &self.regions {
            if let Err(err) = self
                .db
                .enqueue_check_job(url_id, check_round_id, region)
                .await
            {
                tracing::error!(%url_id, %region, "failed to enqueue check job: {err:?}");
            }
        }
    }
}

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{database::Database, http::content_snapshot::RecordContentSnapshot};

/// Row shape of the `content_snapshots` table.
#[derive(Debug, FromRow)]
pub struct ContentSnapshotRow {
    pub id: Uuid,
    pub url_id: Uuid,
    pub content_hash: String,
    pub captured_at: DateTime<Utc>,
}

impl Database {
    pub async fn record_content_snapshot(
        &self,
        input: RecordContentSnapshot,
    ) -> sqlx::Result<ContentSnapshotRow> {
        sqlx::query_as::<_, ContentSnapshotRow>(
            r#"
            INSERT INTO content_snapshots (url_id, content_hash)
            VALUES ($1, $2)
            RETURNING id, url_id, content_hash, captured_at
            "#,
        )
        .bind(input.url_id)
        .bind(input.content_hash)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn latest_content_snapshot(
        &self,
        url_id: Uuid,
    ) -> sqlx::Result<Option<ContentSnapshotRow>> {
        sqlx::query_as::<_, ContentSnapshotRow>(
            "SELECT * FROM content_snapshots WHERE url_id = $1 ORDER BY captured_at DESC LIMIT 1",
        )
        .bind(url_id)
        .fetch_optional(&self.pool)
        .await
    }
}

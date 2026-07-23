use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::content_snapshot::ContentSnapshotRow;

/// Records a new content hash observed for a URL. `id` and `captured_at`
/// are assigned server-side.
#[derive(Debug, Deserialize)]
pub struct RecordContentSnapshot {
    pub url_id: Uuid,
    pub content_hash: String,
}

/// `content_snapshots` row as returned to API clients.
#[derive(Debug, Serialize)]
pub struct ContentSnapshotResponse {
    pub id: Uuid,
    pub url_id: Uuid,
    pub content_hash: String,
    pub captured_at: DateTime<Utc>,
}

impl From<ContentSnapshotRow> for ContentSnapshotResponse {
    fn from(row: ContentSnapshotRow) -> Self {
        Self {
            id: row.id,
            url_id: row.url_id,
            content_hash: row.content_hash,
            captured_at: row.captured_at,
        }
    }
}

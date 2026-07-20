use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    database::Database,
    http::incident::{OpenIncident, ResolveIncident},
};

/// Row shape of the `incidents` table.
#[derive(Debug, FromRow)]
pub struct IncidentRow {
    pub id: Uuid,
    pub url_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub cause: Option<String>,
}

impl Database {
    pub async fn open_incident(&self, input: OpenIncident) -> sqlx::Result<IncidentRow> {
        sqlx::query_as::<_, IncidentRow>(
            r#"
            INSERT INTO incidents (url_id, started_at, cause)
            VALUES ($1, now(), $2)
            RETURNING id, url_id, started_at, resolved_at, resolved, cause
            "#,
        )
        .bind(input.url_id)
        .bind(input.cause)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn resolve_incident(
        &self,
        id: Uuid,
        input: ResolveIncident,
    ) -> sqlx::Result<Option<IncidentRow>> {
        sqlx::query_as::<_, IncidentRow>(
            r#"
            UPDATE incidents SET
                resolved = true,
                resolved_at = now(),
                cause = COALESCE($2, cause)
            WHERE id = $1
            RETURNING id, url_id, started_at, resolved_at, resolved, cause
            "#,
        )
        .bind(id)
        .bind(input.cause)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_open_incidents_for_url(&self, url_id: Uuid) -> sqlx::Result<Vec<IncidentRow>> {
        sqlx::query_as::<_, IncidentRow>(
            "SELECT * FROM incidents WHERE url_id = $1 AND resolved = false ORDER BY started_at DESC",
        )
        .bind(url_id)
        .fetch_all(&self.pool)
        .await
    }
}

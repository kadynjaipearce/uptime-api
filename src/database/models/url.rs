use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    database::Database,
    http::url::{CreateUrl, UpdateUrl},
};

/// Row shape of the `url` table.
#[derive(Debug, FromRow)]
pub struct UrlRow {
    pub id: Uuid,
    pub domain: String,
    pub name: String,
    pub check_interval_seconds: i32,
    pub expected_content: Option<String>,
    pub is_active: bool,
    pub next_check_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Database {
    pub async fn create_url(&self, input: CreateUrl) -> sqlx::Result<UrlRow> {
        sqlx::query_as::<_, UrlRow>(
            r#"
            INSERT INTO url (domain, name, check_interval_seconds, expected_content)
            VALUES ($1, $2, COALESCE($3, 60), $4)
            RETURNING id, domain, name, check_interval_seconds, expected_content,
                      is_active, next_check_at, created_at
            "#,
        )
        .bind(input.domain)
        .bind(input.name)
        .bind(input.check_interval_seconds)
        .bind(input.expected_content)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_url(&self, id: Uuid) -> sqlx::Result<Option<UrlRow>> {
        sqlx::query_as::<_, UrlRow>("SELECT * FROM url WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_urls(&self) -> sqlx::Result<Vec<UrlRow>> {
        sqlx::query_as::<_, UrlRow>("SELECT * FROM url ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_url(&self, id: Uuid, input: UpdateUrl) -> sqlx::Result<Option<UrlRow>> {
        sqlx::query_as::<_, UrlRow>(
            r#"
            UPDATE url SET
                domain = COALESCE($2, domain),
                name = COALESCE($3, name),
                check_interval_seconds = COALESCE($4, check_interval_seconds),
                expected_content = COALESCE($5, expected_content),
                is_active = COALESCE($6, is_active)
            WHERE id = $1
            RETURNING id, domain, name, check_interval_seconds, expected_content,
                      is_active, next_check_at, created_at
            "#,
        )
        .bind(id)
        .bind(input.domain)
        .bind(input.name)
        .bind(input.check_interval_seconds)
        .bind(input.expected_content)
        .bind(input.is_active)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_url(&self, id: Uuid) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM url WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

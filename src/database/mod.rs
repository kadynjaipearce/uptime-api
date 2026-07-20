use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};

pub mod models;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await
            .context("failed to connect to database")?;

        tracing::info!("database connected");

        Ok(Self { pool })
    }
}

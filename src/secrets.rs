use anyhow::{Context, Result};

pub struct Secrets {
    pub database_url: String,
}

impl Secrets {
    /// Loads secrets from a `.env` file (if present) and the environment.
    /// Environment variables always take precedence over `.env`.
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

        Ok(Self { database_url })
    }
}

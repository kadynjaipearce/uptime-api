use anyhow::{Context, Result};

pub struct Secrets {
    pub database_url: String,
}

impl Secrets {
    pub fn load() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

        Ok(Self { database_url })
    }
}

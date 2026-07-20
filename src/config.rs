use anyhow::{Context, Result};
use std::str::FromStr;

pub struct Config {
    pub max_connections: u32,
    pub port: u16,
    pub region: String,
    pub frontend_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self {
            max_connections: env_or("MAX_CONNECTIONS", 1)?,
            port: env_or("PORT", 8080)?,
            region: env_required("REGION")?,
            frontend_url: env_required("FRONTEND_URL")?,
        })
    }
}

/// Reads `key` from the environment and parses it, falling back to `default` when unset.
fn env_or<T>(key: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(key) {
        Ok(raw) => raw.parse().with_context(|| format!("{key} is invalid")),
        Err(_) => Ok(default),
    }
}

fn env_required(key: &str) -> Result<String> {
    std::env::var(key).with_context(|| format!("{key} must be set"))
}

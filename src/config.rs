use anyhow::{Context, Result};
use std::str::FromStr;

pub struct Config {
    pub version: String,
    pub max_connections: u32,
    pub port: u16,
    /// Regions the scheduler dispatches checks to. One check job is enqueued
    pub regions: Vec<String>,
    pub frontend_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self {
            version: env_required("VERSION_SLUG")?,
            max_connections: env_or("MAX_CONNECTIONS", 1)?,
            port: env_or("PORT", 8080)?,
            regions: env_list("REGIONS")?,
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

fn env_list(key: &str) -> Result<Vec<String>> {
    let raw = env_required(key)?;
    let items: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    if items.is_empty() {
        anyhow::bail!("{key} must contain at least one entry");
    }

    Ok(items)
}

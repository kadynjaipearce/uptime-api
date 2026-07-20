use std::sync::Arc;

use anyhow::Result;

use crate::{config::Config, core::AppState, database::Database, secrets::Secrets};

mod config;
mod core;
mod database;
mod error;
mod http;
mod response;
mod secrets;
mod telemetry;

#[tokio::main]
async fn main() {
    telemetry::init();
    telemetry::install_error_hooks().expect("failed to install error hooks");

    if let Err(err) = run().await {
        tracing::error!("{err:?}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let config = Config::load()?;
    let secrets = Secrets::load()?;
    let db = Database::connect(&secrets.database_url, 5).await?;
    let state = Arc::new(AppState { db, secrets });

    let app = http::router(&config, state)?;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

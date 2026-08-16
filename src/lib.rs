use std::sync::Arc;

use anyhow::Result;

use crate::{
    config::Config,
    core::{
        AppState,
        scheduler::{Scheduler, TICK_INTERVAL},
        worker::{POLL_INTERVAL, Worker},
    },
    database::Database,
    secrets::Secrets,
};

pub mod config;
pub mod core;
pub mod database;
pub mod error;
pub mod extract;
pub mod http;
pub mod response;
pub mod secrets;
pub mod telemetry;

pub async fn run() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::load()?;
    let secrets = Secrets::load()?;
    let db = Database::connect(&secrets.database_url, 5).await?;

    db.migrate().await?;

    if config.scheduler_enabled {
        let scheduler = Scheduler::new(db.clone(), TICK_INTERVAL, config.regions.clone());
        tokio::spawn(async move { scheduler.run().await });
    }

    let worker = Worker::new(
        db.clone(),
        config.region.clone(),
        config.dns_servers.clone(),
        POLL_INTERVAL,
    );
    tokio::spawn(async move { worker.run().await });

    let state = Arc::new(AppState { db, secrets });
    let app = http::router(&config, state)?;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

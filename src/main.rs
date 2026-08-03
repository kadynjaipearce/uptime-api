use uptime_api::{run, telemetry};

#[tokio::main]
async fn main() {
    telemetry::init();

    if let Err(err) = telemetry::install_error_hooks() {
        tracing::error!("failed to install error hooks: {err:?}");
        std::process::exit(1);
    }

    if let Err(err) = run().await {
        tracing::error!("{err:?}");
        std::process::exit(1);
    }
}

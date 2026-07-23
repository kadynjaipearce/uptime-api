use uptime_api::{run, telemetry};

#[tokio::main]
async fn main() {
    telemetry::init();
    telemetry::install_error_hooks().expect("failed to install error hooks");

    if let Err(err) = run().await {
        tracing::error!("{err:?}");
        std::process::exit(1);
    }
}

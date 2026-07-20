use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_ansi(true).with_target(true))
        .init();
}

pub fn install_error_hooks() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        tracing::error!("{}", panic_hook.panic_report(panic_info));
    }));

    Ok(())
}

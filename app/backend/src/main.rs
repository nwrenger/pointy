#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    logging();

    pointy_lib::run()
}

fn logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

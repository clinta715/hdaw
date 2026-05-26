#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod ui;
mod project;
mod utils;

use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("Starting HDAW v{}", env!("CARGO_PKG_VERSION"));

    let app = app::HdawApp::new();
    app.run();
}
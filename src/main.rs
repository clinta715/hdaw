#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod project;
mod utils;
#[cfg(feature = "qt")]
mod ui_qt;

use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("Starting HDAW v{}", env!("CARGO_PKG_VERSION"));

    let app = app::HdawApp::new();

    #[cfg(feature = "qt")]
    {
        use crate::ui_qt::state;
        state::init(app.app_state());

        std::thread::spawn(|| {
            use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

            let mut app = QGuiApplication::new();
            let mut engine = QQmlApplicationEngine::new();

            if let Some(engine) = engine.as_mut() {
                engine.load(&QUrl::from("qrc:/qt/qml/com/hdaw/src/ui_qt/main.qml"));
            }

            app.as_mut().unwrap().exec();
        });

        info!("Qt main window launched on separate thread");
    }

    app.run();
}
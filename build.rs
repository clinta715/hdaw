fn main() {
    #[cfg(feature = "qt")]
    {
        use cxx_qt_build::QmlModule;

        cxx_qt_build::CxxQtBuilder::new_qml_module(
            QmlModule::new("com.hdaw").qml_file("src/ui_qt/main.qml"),
        )
        .qt_module("Quick")
        .files([
            "src/ui_qt/mod.rs",
            "src/ui_qt/shortcut_handler.rs",
            "src/ui_qt/pool.rs",
            "src/ui_qt/effects.rs",
            "src/ui_qt/mixer.rs",
            "src/ui_qt/timeline.rs",
            "src/ui_qt/state_bridge.rs",
        ])
        .build();
    }
}

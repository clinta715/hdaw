use crate::audio::engine::AudioEngine;
use crate::project::undo::{execute_command, EditCommand, UndoStack};
use crate::project::Project;
use crate::ui::main_window::MainWindow;
use crate::ui::timeline::{self, sync_project_to_timeline};
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct HdawApp {
    window: MainWindow,
    project: Arc<Mutex<Project>>,
    undo_stack: Arc<Mutex<UndoStack>>,
    _audio_engine: Arc<Mutex<AudioEngine>>,
}

impl HdawApp {
    pub fn new() -> Self {
        info!("Initializing HDAW application");

        let project = Arc::new(Mutex::new(Project::new()));
        let undo_stack = Arc::new(Mutex::new(UndoStack::new()));
        let audio_engine = Arc::new(Mutex::new(AudioEngine::new()));

        let window = MainWindow::new().expect("Failed to create main window");

        let app = Self {
            window,
            project,
            undo_stack,
            _audio_engine: audio_engine,
        };

        app.setup_undo_callbacks();
        app.setup_timeline();
        app.sync_timeline();

        info!("HDAW initialized successfully");
        app
    }

    fn setup_undo_callbacks(&self) {
        let project = self.project.clone();
        let undo_stack = self.undo_stack.clone();
        let window_weak = self.window.as_weak();

        self.window.on_undo(move || {
            let mut p = match project.lock() {
                Ok(p) => p,
                Err(e) => { tracing::error!("lock: {}", e); return; }
            };
            let mut stack = match undo_stack.lock() {
                Ok(s) => s,
                Err(e) => { tracing::error!("lock: {}", e); return; }
            };
            stack.undo(&mut p);
            drop(stack);
            if let Some(w) = window_weak.upgrade() {
                sync_project_to_timeline(&p, &w);
            }
        });

        let project = self.project.clone();
        let undo_stack = self.undo_stack.clone();
        let window_weak = self.window.as_weak();

        self.window.on_redo(move || {
            let mut p = match project.lock() {
                Ok(p) => p,
                Err(e) => { tracing::error!("lock: {}", e); return; }
            };
            let mut stack = match undo_stack.lock() {
                Ok(s) => s,
                Err(e) => { tracing::error!("lock: {}", e); return; }
            };
            stack.redo(&mut p);
            drop(stack);
            if let Some(w) = window_weak.upgrade() {
                sync_project_to_timeline(&p, &w);
            }
        });
    }

    fn setup_timeline(&self) {
        timeline::setup_timeline_callbacks(&self.window, self.project.clone());
    }

    fn sync_timeline(&self) {
        if let Ok(project) = self.project.lock() {
            sync_project_to_timeline(&project, &self.window);
        }
    }

    pub fn run(self) {
        info!("Running HDAW main loop");
        self.window.run().unwrap();
    }
}

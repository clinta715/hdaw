pub mod automation;
pub mod clip;
pub mod editing;
pub mod io;
pub mod track;
pub mod undo;

use crate::project::track::Track;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub sample_rate: u32,
    pub bpm: f64,
    pub time_signature: (u8, u8),
    pub tracks: Vec<Track>,
}

impl Project {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Untitled Project".to_string(),
            sample_rate: 44100,
            bpm: 120.0,
            time_signature: (4, 4),
            tracks: Vec::new(),
        }
    }
}

impl Default for Project {
    fn default() -> Self { Self::new() }
}
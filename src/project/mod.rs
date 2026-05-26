pub mod automation;
pub mod bus;
pub mod clip;
pub mod editing;
pub mod io;
pub mod track;
pub mod undo;

use crate::project::track::Track;
use crate::project::bus::Bus;
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
    pub buses: Vec<Bus>,
    pub master_volume: f32,
    pub master_pan: f32,
    pub master_mute: bool,
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
            buses: Vec::new(),
            master_volume: 1.0,
            master_pan: 0.0,
            master_mute: false,
        }
    }
}

impl Default for Project {
    fn default() -> Self { Self::new() }
}
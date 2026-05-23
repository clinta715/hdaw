use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStretchParams {
    pub ratio: f32,
    pub algorithm: StretchAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StretchAlgorithm {
    Fast,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchShiftParams {
    pub semitones: f32,
    pub algorithm: PitchAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PitchAlgorithm {
    Fast,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: Uuid,
    pub source_path: String,
    pub name: String,
    pub position: u64,
    pub offset: u64,
    pub length: u64,
    pub fade_in: u64,
    pub fade_out: u64,
    pub gain: f32,
    pub time_stretch: Option<TimeStretchParams>,
    pub pitch_shift: Option<PitchShiftParams>,
    pub color: (u8, u8, u8),
}

impl AudioClip {
    pub fn new(source_path: String, name: String, position: u64, length: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_path,
            name,
            position,
            offset: 0,
            length,
            fade_in: 0,
            fade_out: 0,
            gain: 1.0,
            time_stretch: None,
            pitch_shift: None,
            color: (150, 150, 150),
        }
    }

    pub fn end_position(&self) -> u64 {
        self.position + self.length
    }

    pub fn overlaps(&self, start: u64, end: u64) -> bool {
        self.position < end && self.end_position() > start
    }

    pub fn set_trim(&mut self, new_offset: u64, new_length: u64) {
        self.offset = new_offset;
        self.length = new_length;
    }

    pub fn move_by(&mut self, delta: i64) {
        if delta < 0 {
            self.position = self.position.saturating_sub((-delta) as u64);
        } else {
            self.position += delta as u64;
        }
    }

    pub fn source_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.source_path)
    }
}
use crate::audio::effects::Effect;
use crate::project::clip::AudioClip;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
    pub color: (u8, u8, u8),
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub armed: bool,
    pub clips: Vec<AudioClip>,
    pub effects_chain: Vec<EffectInstance>,
    pub automation: HashMap<String, AutomationLane>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInstance {
    pub id: Uuid,
    pub effect_type: String,
    pub bypass: bool,
    pub parameters: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    pub parameter_path: String,
    pub points: Vec<AutomationPoint>,
    pub enabled: bool,
    pub color: (u8, u8, u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationPoint {
    pub time: f64,
    pub value: f32,
    pub curve: CurveType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    Bezier(f32, f32),
    Step,
}

impl Track {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color: (100, 150, 200),
            volume: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            armed: false,
            clips: Vec::new(),
            effects_chain: Vec::new(),
            automation: HashMap::new(),
        }
    }

    pub fn add_clip(&mut self, clip: AudioClip) {
        self.clips.push(clip);
        self.clips.sort_by_key(|c| c.position);
    }

    pub fn remove_clip(&mut self, clip_id: Uuid) {
        self.clips.retain(|c| c.id != clip_id);
    }

    pub fn get_clips_in_range(&self, start: u64, end: u64) -> Vec<&AudioClip> {
        self.clips
            .iter()
            .filter(|c| c.position < end && c.position + c.length > start)
            .collect()
    }
}

impl Default for Track {
    fn default() -> Self {
        Self::new(String::from("Track 1"))
    }
}
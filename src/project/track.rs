use crate::audio::effects::Effect;
use crate::project::clip::AudioClip;
use crate::project::automation::AutomationLane;
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
    pub input_monitoring: bool,
    pub output_id: Option<Uuid>,
    pub sends: Vec<AuxSend>,
    pub clips: Vec<AudioClip>,
    pub effects_chain: Vec<EffectInstance>,
    pub automation: HashMap<String, AutomationLane>,
    pub selected_automation_param: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxSend {
    pub id: Uuid,
    pub target_id: Uuid,
    pub level: f32,
    pub is_active: bool,
    pub pre_fader: bool,
}

impl AuxSend {
    pub fn new(target_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            target_id,
            level: 1.0,
            is_active: true,
            pre_fader: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInstance {
    pub id: Uuid,
    pub effect_type: String,
    pub bypass: bool,
    pub parameters: HashMap<String, f32>,
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
            input_monitoring: false,
            output_id: None,
            sends: Vec::new(),
            clips: Vec::new(),
            effects_chain: Vec::new(),
            automation: HashMap::new(),
            selected_automation_param: None,
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
use crate::project::track::{EffectInstance, AuxSend};
use crate::project::automation::AutomationLane;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    pub id: Uuid,
    pub name: String,
    pub color: (u8, u8, u8),
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub output_id: Option<Uuid>, // None means Master
    pub sends: Vec<AuxSend>,
    pub effects_chain: Vec<EffectInstance>,
    pub automation: HashMap<String, AutomationLane>,
}

impl Bus {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color: (150, 150, 150),
            volume: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            output_id: None,
            sends: Vec::new(),
            effects_chain: Vec::new(),
            automation: HashMap::new(),
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new(String::from("Bus 1"))
    }
}

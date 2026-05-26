pub mod eq;
pub mod compressor;
pub mod reverb;
pub mod delay;
pub mod factory;

use crate::audio::buffer::AudioBuffer;

pub trait Effect: Send + Sync {
    fn process(&mut self, buffer: &mut AudioBuffer);
    fn get_parameter(&self, name: &str) -> Option<f32>;
    fn set_parameter(&mut self, name: &str, value: f32);
    fn get_name(&self) -> &str;
    fn is_bypassed(&self) -> bool;
    fn set_bypassed(&mut self, bypassed: bool);
    fn get_gain_reduction(&self) -> f32 { 0.0 }
}

pub mod params {
    pub const EQ_LOW_FREQ: &str = "eq_low_freq";
    pub const EQ_LOW_GAIN: &str = "eq_low_gain";
    pub const EQ_MID_FREQ: &str = "eq_mid_freq";
    pub const EQ_MID_GAIN: &str = "eq_mid_gain";
    pub const EQ_MID_Q: &str = "eq_mid_q";
    pub const EQ_HIGH_FREQ: &str = "eq_high_freq";
    pub const EQ_HIGH_GAIN: &str = "eq_high_gain";

    pub const COMP_THRESHOLD: &str = "comp_threshold";
    pub const COMP_RATIO: &str = "comp_ratio";
    pub const COMP_ATTACK: &str = "comp_attack";
    pub const COMP_RELEASE: &str = "comp_release";
    pub const COMP_MAKEUP: &str = "comp_makeup";

    pub const REVERB_ROOM_SIZE: &str = "reverb_room_size";
    pub const REVERB_DAMPING: &str = "reverb_damping";
    pub const REVERB_WET_DRY: &str = "reverb_wet_dry";

    pub const DELAY_TIME: &str = "delay_time";
    pub const DELAY_FEEDBACK: &str = "delay_feedback";
    pub const DELAY_MIX: &str = "delay_mix";
}
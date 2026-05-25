use crate::audio::effects::{Effect, eq, compressor, reverb, delay};
use crate::project::track::EffectInstance;

pub fn create_effect(instance: &EffectInstance, sample_rate: u32) -> Box<dyn Effect> {
    let effect_type = instance.effect_type.as_str();
    let mut effect: Box<dyn Effect> = match effect_type {
        "Equalizer" => Box::new(eq::Equalizer::new(sample_rate)),
        "Compressor" => Box::new(compressor::Compressor::new(sample_rate)),
        "Reverb" => Box::new(reverb::Reverb::new(sample_rate)),
        "Delay" => Box::new(delay::Delay::new(sample_rate)),
        _ => {
            tracing::warn!("Unknown effect type: {}", effect_type);
            return Box::new(eq::Equalizer::new(sample_rate));
        }
    };

    for (name, value) in &instance.parameters {
        effect.set_parameter(name, *value);
    }

    if instance.bypass {
        effect.set_bypassed(true);
    }

    effect
}

pub fn create_effect_chain(instances: &[EffectInstance], sample_rate: u32) -> Vec<Box<dyn Effect>> {
    instances.iter().map(|i| create_effect(i, sample_rate)).collect()
}

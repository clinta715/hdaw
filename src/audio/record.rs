use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct RecordState {
    pub recording: bool,
    pub record_start_pos: u64,
    pub armed_track_ids: HashSet<Uuid>,
    pub track_buffers: HashMap<Uuid, Vec<f32>>,
}

impl RecordState {
    pub fn new() -> Self {
        Self {
            recording: false,
            record_start_pos: 0,
            armed_track_ids: HashSet::new(),
            track_buffers: HashMap::new(),
        }
    }

    pub fn start(&mut self, armed_ids: &HashSet<Uuid>, position: u64, sample_rate: u32) {
        self.recording = true;
        self.record_start_pos = position;
        self.armed_track_ids = armed_ids.clone();
        self.track_buffers.clear();

        let prealloc_samples = (sample_rate as usize).saturating_mul(600) * 2;
        for track_id in armed_ids {
            self.track_buffers
                .insert(*track_id, Vec::with_capacity(prealloc_samples));
        }
    }

    pub fn stop(&mut self) -> (HashMap<Uuid, Vec<f32>>, u64) {
        self.recording = false;
        let buffers = std::mem::take(&mut self.track_buffers);
        (buffers, self.record_start_pos)
    }

    pub fn append_samples(&mut self, samples: &[f32]) {
        for buf in self.track_buffers.values_mut() {
            buf.extend_from_slice(samples);
        }
    }

    pub fn get_record_start_pos(&self) -> u64 {
        self.record_start_pos
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }
}

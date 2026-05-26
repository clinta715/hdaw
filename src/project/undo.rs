use crate::project::automation::AutomationPoint;
use crate::project::bus::Bus;
use crate::project::clip::AudioClip;
use crate::project::track::{EffectInstance, Track};
use crate::project::Project;
use uuid::Uuid;

pub type ClipSnapshot = AudioClip;
pub type TrackSnapshot = Track;
pub type BusSnapshot = Bus;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FadeEdge {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoalesceKey {
    Volume(Uuid),
    Pan(Uuid),
    Fade(Uuid, FadeEdge),
    TimeStretch(Uuid),
    BusVolume(Uuid),
    BusPan(Uuid),
    EffectParam(Uuid, bool, usize, String),
}

#[derive(Debug, Clone)]
pub enum EditCommand {
    MoveClip {
        track_id: Uuid,
        clip_id: Uuid,
        old_pos: u64,
        new_pos: u64,
    },
    MoveClipToTrack {
        source_track_id: Uuid,
        dest_track_id: Uuid,
        clip_snapshot: ClipSnapshot,
    },
    ResizeClip {
        track_id: Uuid,
        clip_id: Uuid,
        old_start: u64,
        old_end: u64,
        new_start: u64,
        new_end: u64,
    },
    SplitClip {
        track_id: Uuid,
        original_clip: ClipSnapshot,
        new_clips: (ClipSnapshot, ClipSnapshot),
    },
    DeleteClips {
        track_id: Uuid,
        clips: Vec<ClipSnapshot>,
    },
    ChangeVolume {
        track_id: Uuid,
        old_val: f32,
        new_val: f32,
    },
    ChangePan {
        track_id: Uuid,
        old_val: f32,
        new_val: f32,
    },
    ChangeBusVolume {
        bus_id: Uuid,
        old_val: f32,
        new_val: f32,
    },
    ChangeBusPan {
        bus_id: Uuid,
        old_val: f32,
        new_val: f32,
    },
    ChangeMasterVolume {
        old_val: f32,
        new_val: f32,
    },
    ChangeMasterPan {
        old_val: f32,
        new_val: f32,
    },
    ToggleMasterMute,
    ChangeTrackOutput {
        track_id: Uuid,
        old_output: Option<Uuid>,
        new_output: Option<Uuid>,
    },
    ChangeBusOutput {
        bus_id: Uuid,
        old_output: Option<Uuid>,
        new_output: Option<Uuid>,
    },
    AddSend {
        source_id: Uuid,
        is_track: bool,
        send: crate::project::track::AuxSend,
    },
    RemoveSend {
        source_id: Uuid,
        is_track: bool,
        send: crate::project::track::AuxSend,
    },
    SetFade {
        track_id: Uuid,
        clip_id: Uuid,
        edge: FadeEdge,
        old_dur: u64,
        new_dur: u64,
    },
    TimeStretch {
        track_id: Uuid,
        clip_id: Uuid,
        old_length: u64,
        old_stretch: Option<crate::project::clip::TimeStretchParams>,
        new_length: u64,
        new_stretch: Option<crate::project::clip::TimeStretchParams>,
    },
    AddTrack {
        snapshot: TrackSnapshot,
        index: usize,
    },
    RemoveTrack {
        snapshot: TrackSnapshot,
        index: usize,
    },
    RemoveBus {
        snapshot: BusSnapshot,
        index: usize,
    },
    AddClip {
        track_id: Uuid,
        clip: ClipSnapshot,
    },
    RemoveClip {
        track_id: Uuid,
        clip: ClipSnapshot,
    },

    AddEffect {
        target_id: Uuid,
        is_track: bool,
        effect: EffectInstance,
        index: usize,
    },
    RemoveEffect {
        target_id: Uuid,
        is_track: bool,
        effect: EffectInstance,
        index: usize,
    },
    ChangeEffectParam {
        target_id: Uuid,
        is_track: bool,
        effect_index: usize,
        param_name: String,
        old_val: f32,
        new_val: f32,
    },
    MoveEffect {
        target_id: Uuid,
        is_track: bool,
        from_index: usize,
        to_index: usize,
    },
    ToggleEffectBypass {
        target_id: Uuid,
        is_track: bool,
        effect_index: usize,
    },
    AddAutomationPoint {
        track_id: Uuid,
        parameter_name: String,
        point: AutomationPoint,
    },
    RemoveAutomationPoint {
        track_id: Uuid,
        parameter_name: String,
        point: AutomationPoint,
        index: usize,
    },
    MoveAutomationPoint {
        track_id: Uuid,
        parameter_name: String,
        old_point: AutomationPoint,
        new_point: AutomationPoint,
        index: usize,
    },
}

impl EditCommand {
    pub fn coalesce_key(&self) -> Option<CoalesceKey> {
        match self {
            EditCommand::ChangeVolume { track_id, .. } => Some(CoalesceKey::Volume(*track_id)),
            EditCommand::ChangePan { track_id, .. } => Some(CoalesceKey::Pan(*track_id)),
            EditCommand::SetFade { track_id, clip_id: _, edge, .. } => {
                Some(CoalesceKey::Fade(*track_id, edge.clone()))
            }
            EditCommand::TimeStretch { track_id, clip_id: _, .. } => {
                Some(CoalesceKey::TimeStretch(*track_id))
            }
            EditCommand::ChangeBusVolume { bus_id, .. } => Some(CoalesceKey::BusVolume(*bus_id)),
            EditCommand::ChangeBusPan { bus_id, .. } => Some(CoalesceKey::BusPan(*bus_id)),
            EditCommand::ChangeEffectParam { target_id, is_track, effect_index, param_name, .. } => {
                Some(CoalesceKey::EffectParam(*target_id, *is_track, *effect_index, param_name.clone()))
            }
            _ => None,
        }
    }
}

fn find_track_mut(project: &mut Project, track_id: Uuid) -> Option<&mut Track> {
    project.tracks.iter_mut().find(|t| t.id == track_id)
}

fn find_bus_mut(project: &mut Project, bus_id: Uuid) -> Option<&mut Bus> {
    project.buses.iter_mut().find(|b| b.id == bus_id)
}

fn get_effects_chain_mut<'a>(project: &'a mut Project, target_id: Uuid, is_track: bool) -> Option<&'a mut Vec<EffectInstance>> {
    if is_track {
        find_track_mut(project, target_id).map(|t| &mut t.effects_chain)
    } else {
        find_bus_mut(project, target_id).map(|b| &mut b.effects_chain)
    }
}

pub fn execute_command(project: &mut Project, cmd: &EditCommand) {
    match cmd {
        EditCommand::MoveClip { track_id, clip_id, new_pos, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    clip.position = *new_pos;
                }
            }
        }
        EditCommand::MoveClipToTrack { source_track_id, dest_track_id, clip_snapshot } => {
            if let Some(track) = find_track_mut(project, *source_track_id) {
                track.clips.retain(|c| c.id != clip_snapshot.id);
            }
            if let Some(track) = find_track_mut(project, *dest_track_id) {
                track.add_clip(clip_snapshot.clone());
            }
        }
        EditCommand::ResizeClip { track_id, clip_id, new_start, new_end, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    clip.offset = *new_start;
                    clip.length = new_end.saturating_sub(*new_start);
                }
            }
        }
        EditCommand::SplitClip { track_id, original_clip, new_clips } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.clips.retain(|c| c.id != original_clip.id);
                let (left, right) = new_clips.clone();
                track.add_clip(left);
                track.add_clip(right);
            }
        }
        EditCommand::DeleteClips { track_id, clips } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                let ids: Vec<Uuid> = clips.iter().map(|c| c.id).collect();
                track.clips.retain(|c| !ids.contains(&c.id));
            }
        }
        EditCommand::ChangeVolume { track_id, new_val, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.volume = *new_val;
            }
        }
        EditCommand::ChangePan { track_id, new_val, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.pan = *new_val;
            }
        }
        EditCommand::SetFade { track_id, clip_id, edge, new_dur, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    match edge {
                        FadeEdge::In => clip.fade_in = *new_dur,
                        FadeEdge::Out => clip.fade_out = *new_dur,
                    }
                }
            }
        }
        EditCommand::TimeStretch { track_id, clip_id, new_length, new_stretch, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    clip.length = *new_length;
                    clip.time_stretch = new_stretch.clone();
                }
            }
        }
        EditCommand::AddTrack { snapshot, index } => {
            let mut track = snapshot.clone();
            track.clips = snapshot.clips.clone();
            let idx = (*index).min(project.tracks.len());
            project.tracks.insert(idx, track);
        }
        EditCommand::RemoveTrack { index, snapshot: _ } => {
            if *index < project.tracks.len() {
                project.tracks.remove(*index);
            }
        }
        EditCommand::RemoveBus { index, snapshot: _ } => {
            if *index < project.buses.len() {
                project.buses.remove(*index);
            }
        }
        EditCommand::AddClip { track_id, clip } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.add_clip(clip.clone());
            }
        }
        EditCommand::RemoveClip { track_id, clip } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.clips.retain(|c| c.id != clip.id);
            }
        }
        EditCommand::AddEffect { target_id, is_track, effect, index } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                let idx = (*index).min(chain.len());
                chain.insert(idx, effect.clone());
            }
        }
        EditCommand::RemoveEffect { target_id, is_track, effect, .. } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                chain.retain(|e| e.id != effect.id);
            }
        }
        EditCommand::ChangeEffectParam { target_id, is_track, effect_index, param_name, new_val, .. } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                if let Some(effect) = chain.get_mut(*effect_index) {
                    effect.parameters.insert(param_name.clone(), *new_val);
                }
            }
        }
        EditCommand::MoveEffect { target_id, is_track, from_index, to_index } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                if *from_index < chain.len() && *to_index < chain.len() {
                    let effect = chain.remove(*from_index);
                    chain.insert(*to_index, effect);
                }
            }
        }
        EditCommand::ToggleEffectBypass { target_id, is_track, effect_index } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                if let Some(effect) = chain.get_mut(*effect_index) {
                    effect.bypass = !effect.bypass;
                }
            }
        }
        EditCommand::ChangeBusVolume { bus_id, new_val, .. } => {
            if let Some(bus) = find_bus_mut(project, *bus_id) {
                bus.volume = *new_val;
            }
        }
        EditCommand::ChangeBusPan { bus_id, new_val, .. } => {
            if let Some(bus) = find_bus_mut(project, *bus_id) {
                bus.pan = *new_val;
            }
        }
        EditCommand::ChangeMasterVolume { new_val, .. } => { project.master_volume = *new_val; }
        EditCommand::ChangeMasterPan { new_val, .. } => { project.master_pan = *new_val; }
        EditCommand::ToggleMasterMute => { project.master_mute = !project.master_mute; }
        EditCommand::ChangeTrackOutput { track_id, new_output, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) { track.output_id = *new_output; }
        }
        EditCommand::ChangeBusOutput { bus_id, new_output, .. } => {
            if let Some(bus) = find_bus_mut(project, *bus_id) { bus.output_id = *new_output; }
        }
        EditCommand::AddSend { source_id, is_track, send } => {
            if *is_track { if let Some(track) = find_track_mut(project, *source_id) { track.sends.push(send.clone()); } }
            else { if let Some(bus) = find_bus_mut(project, *source_id) { bus.sends.push(send.clone()); } }
        }
        EditCommand::RemoveSend { source_id, is_track, send } => {
            if *is_track { if let Some(track) = find_track_mut(project, *source_id) { track.sends.retain(|s| s.id != send.id); } }
            else { if let Some(bus) = find_bus_mut(project, *source_id) { bus.sends.retain(|s| s.id != send.id); } }
        }
        EditCommand::AddAutomationPoint { track_id, parameter_name, point } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                let lane = track.automation.entry(parameter_name.clone()).or_insert_with(|| crate::project::automation::AutomationLane::new(parameter_name.clone()));
                lane.add_point(point.time, point.value);
            }
        }
        EditCommand::RemoveAutomationPoint { track_id, parameter_name, index, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(lane) = track.automation.get_mut(parameter_name.as_str()) {
                    if *index < lane.points.len() { lane.points.remove(*index); }
                }
            }
        }
        EditCommand::MoveAutomationPoint { track_id, parameter_name, new_point, index, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(lane) = track.automation.get_mut(parameter_name.as_str()) {
                    if *index < lane.points.len() { lane.points[*index] = new_point.clone(); }
                }
            }
        }
    }
}

pub fn undo_command(project: &mut Project, cmd: &EditCommand) {
    match cmd {
        EditCommand::MoveClip { track_id, clip_id, old_pos, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    clip.position = *old_pos;
                }
            }
        }
        EditCommand::MoveClipToTrack { source_track_id, dest_track_id, clip_snapshot } => {
            if let Some(track) = find_track_mut(project, *dest_track_id) {
                track.clips.retain(|c| c.id != clip_snapshot.id);
            }
            if let Some(track) = find_track_mut(project, *source_track_id) {
                track.add_clip(clip_snapshot.clone());
            }
        }
        EditCommand::ResizeClip { track_id, clip_id, old_start, old_end, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    clip.offset = *old_start;
                    clip.length = old_end.saturating_sub(*old_start);
                }
            }
        }
        EditCommand::SplitClip { track_id, original_clip, new_clips } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                let (left, right) = new_clips;
                track.clips.retain(|c| c.id != left.id && c.id != right.id);
                track.add_clip(original_clip.clone());
            }
        }
        EditCommand::DeleteClips { track_id, clips } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                for clip in clips {
                    track.add_clip(clip.clone());
                }
            }
        }
        EditCommand::ChangeVolume { track_id, old_val, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.volume = *old_val;
            }
        }
        EditCommand::ChangePan { track_id, old_val, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.pan = *old_val;
            }
        }
        EditCommand::SetFade { track_id, clip_id, edge, old_dur, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    match edge {
                        FadeEdge::In => clip.fade_in = *old_dur,
                        FadeEdge::Out => clip.fade_out = *old_dur,
                    }
                }
            }
        }
        EditCommand::TimeStretch { track_id, clip_id, old_length, old_stretch, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == *clip_id) {
                    clip.length = *old_length;
                    clip.time_stretch = old_stretch.clone();
                }
            }
        }
        EditCommand::AddTrack { index, .. } => {
            if *index < project.tracks.len() {
                project.tracks.remove(*index);
            }
        }
        EditCommand::RemoveTrack { snapshot, index } => {
            let idx = (*index).min(project.tracks.len());
            let mut track = snapshot.clone();
            track.clips = snapshot.clips.clone();
            project.tracks.insert(idx, track);
        }
        EditCommand::RemoveBus { snapshot, index } => {
            let idx = (*index).min(project.buses.len());
            project.buses.insert(idx, snapshot.clone());
        }
        EditCommand::AddClip { track_id, clip } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.clips.retain(|c| c.id != clip.id);
            }
        }
        EditCommand::RemoveClip { track_id, clip } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                track.add_clip(clip.clone());
            }
        }
        EditCommand::AddEffect { target_id, is_track, effect, .. } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                chain.retain(|e| e.id != effect.id);
            }
        }
        EditCommand::RemoveEffect { target_id, is_track, effect, index } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                let idx = (*index).min(chain.len());
                chain.insert(idx, effect.clone());
            }
        }
        EditCommand::ChangeEffectParam { target_id, is_track, effect_index, param_name, old_val, .. } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                if let Some(effect) = chain.get_mut(*effect_index) {
                    effect.parameters.insert(param_name.clone(), *old_val);
                }
            }
        }
        EditCommand::MoveEffect { target_id, is_track, from_index, to_index } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                if *to_index < chain.len() && *from_index < chain.len() {
                    let effect = chain.remove(*to_index);
                    let idx = (*from_index).min(chain.len());
                    chain.insert(idx, effect);
                }
            }
        }
        EditCommand::ToggleEffectBypass { target_id, is_track, effect_index } => {
            if let Some(chain) = get_effects_chain_mut(project, *target_id, *is_track) {
                if let Some(effect) = chain.get_mut(*effect_index) {
                    effect.bypass = !effect.bypass;
                }
            }
        }
        EditCommand::ChangeBusVolume { bus_id, old_val, .. } => {
            if let Some(bus) = find_bus_mut(project, *bus_id) {
                bus.volume = *old_val;
            }
        }
        EditCommand::ChangeBusPan { bus_id, old_val, .. } => {
            if let Some(bus) = find_bus_mut(project, *bus_id) {
                bus.pan = *old_val;
            }
        }
        EditCommand::ChangeMasterVolume { old_val, .. } => { project.master_volume = *old_val; }
        EditCommand::ChangeMasterPan { old_val, .. } => { project.master_pan = *old_val; }
        EditCommand::ToggleMasterMute => { project.master_mute = !project.master_mute; }
        EditCommand::ChangeTrackOutput { track_id, old_output, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) { track.output_id = *old_output; }
        }
        EditCommand::ChangeBusOutput { bus_id, old_output, .. } => {
            if let Some(bus) = find_bus_mut(project, *bus_id) { bus.output_id = *old_output; }
        }
        EditCommand::AddSend { source_id, is_track, send } => {
            if *is_track { if let Some(track) = find_track_mut(project, *source_id) { track.sends.retain(|s| s.id != send.id); } }
            else { if let Some(bus) = find_bus_mut(project, *source_id) { bus.sends.retain(|s| s.id != send.id); } }
        }
        EditCommand::RemoveSend { source_id, is_track, send } => {
            if *is_track { if let Some(track) = find_track_mut(project, *source_id) { track.sends.push(send.clone()); } }
            else { if let Some(bus) = find_bus_mut(project, *source_id) { bus.sends.push(send.clone()); } }
        }
        EditCommand::AddAutomationPoint { track_id, parameter_name, point } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(lane) = track.automation.get_mut(parameter_name.as_str()) {
                    lane.points.retain(|p| (p.time - point.time).abs() > 0.000001);
                }
            }
        }
        EditCommand::RemoveAutomationPoint { track_id, parameter_name, point, index } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                let lane = track.automation.entry(parameter_name.clone()).or_insert_with(|| crate::project::automation::AutomationLane::new(parameter_name.clone()));
                let idx = (*index).min(lane.points.len());
                lane.points.insert(idx, point.clone());
            }
        }
        EditCommand::MoveAutomationPoint { track_id, parameter_name, old_point, index, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(lane) = track.automation.get_mut(parameter_name.as_str()) {
                    if *index < lane.points.len() { lane.points[*index] = old_point.clone(); }
                }
            }
        }
    }
}

const MAX_UNDO_DEPTH: usize = 100;

#[derive(Debug, Clone)]
pub struct UndoStack {
    commands: Vec<EditCommand>,
    cursor: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(MAX_UNDO_DEPTH),
            cursor: 0,
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.cursor = 0;
    }

    pub fn push(&mut self, cmd: EditCommand) {
        let coalesce_key = cmd.coalesce_key();
        let can_coalesce = coalesce_key.is_some() && self.cursor == self.commands.len();

        if can_coalesce {
            if let Some(top) = self.commands.last() {
                if top.coalesce_key() == cmd.coalesce_key() {
                    return;
                }
            }
        }

        self.commands.truncate(self.cursor);
        self.commands.push(cmd);

        if self.commands.len() > MAX_UNDO_DEPTH {
            self.commands.remove(0);
        }

        self.cursor = self.commands.len();
    }

    pub fn undo(&mut self, project: &mut Project) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        if let Some(cmd) = self.commands.get(self.cursor) {
            undo_command(project, cmd);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, project: &mut Project) -> bool {
        if self.cursor >= self.commands.len() {
            return false;
        }
        if let Some(cmd) = self.commands.get(self.cursor) {
            execute_command(project, cmd);
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor < self.commands.len()
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

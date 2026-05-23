use crate::project::clip::AudioClip;
use crate::project::track::Track;
use crate::project::Project;
use uuid::Uuid;

pub type ClipSnapshot = AudioClip;
pub type TrackSnapshot = Track;

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
}

#[derive(Debug, Clone)]
pub enum EditCommand {
    MoveClip {
        track_id: Uuid,
        clip_id: Uuid,
        old_pos: u64,
        new_pos: u64,
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
            _ => None,
        }
    }
}

fn find_track_mut(project: &mut Project, track_id: Uuid) -> Option<&mut Track> {
    project.tracks.iter_mut().find(|t| t.id == track_id)
}

fn find_clip_mut<'a>(track: &'a mut Track, clip_id: Uuid) -> Option<&'a mut AudioClip> {
    track.clips.iter_mut().find(|c| c.id == clip_id)
}

pub fn execute_command(project: &mut Project, cmd: &EditCommand) {
    match cmd {
        EditCommand::MoveClip { track_id, clip_id, new_pos, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = find_clip_mut(track, *clip_id) {
                    clip.position = *new_pos;
                }
            }
        }
        EditCommand::ResizeClip { track_id, clip_id, new_start, new_end, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = find_clip_mut(track, *clip_id) {
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
                if let Some(clip) = find_clip_mut(track, *clip_id) {
                    match edge {
                        FadeEdge::In => clip.fade_in = *new_dur,
                        FadeEdge::Out => clip.fade_out = *new_dur,
                    }
                }
            }
        }
        EditCommand::TimeStretch { track_id, clip_id, new_length, new_stretch, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = find_clip_mut(track, *clip_id) {
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
        EditCommand::RemoveTrack { .. } => {}
    }
}

pub fn undo_command(project: &mut Project, cmd: &EditCommand) {
    match cmd {
        EditCommand::MoveClip { track_id, clip_id, old_pos, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = find_clip_mut(track, *clip_id) {
                    clip.position = *old_pos;
                }
            }
        }
        EditCommand::ResizeClip { track_id, clip_id, old_start, old_end, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = find_clip_mut(track, *clip_id) {
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
                if let Some(clip) = find_clip_mut(track, *clip_id) {
                    match edge {
                        FadeEdge::In => clip.fade_in = *old_dur,
                        FadeEdge::Out => clip.fade_out = *old_dur,
                    }
                }
            }
        }
        EditCommand::TimeStretch { track_id, clip_id, old_length, old_stretch, .. } => {
            if let Some(track) = find_track_mut(project, *track_id) {
                if let Some(clip) = find_clip_mut(track, *clip_id) {
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

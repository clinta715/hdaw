use crate::project::automation::AutomationPoint;
use crate::project::clip::StretchAlgorithm;
use crate::project::editing::{clips_overlap, create_auto_crossfade};
use crate::project::track::Track;
use crate::project::undo::{EditCommand, UndoStack, FadeEdge as FE};
use crate::project::Project;
use crate::ui::timeline::{pixels_to_samples, sync_project_to_timeline_with_waveforms};
use crate::utils::waveform::WaveformPeaks;
use crate::ui::main_window::MainWindow;
use slint::Model;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct DragState {
    pub clip_id: Uuid,
    pub track_id: Uuid,
    pub track_index: i32,
    pub original_pos: u64,
    pub original_length: u64,
    pub original_fade_in: u64,
    pub original_fade_out: u64,
    pub click_offset: u64,
    pub drag_edge: Option<DragEdge>,
    pub destination_track_id: Option<Uuid>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum DragEdge { Left, Right, Stretch, FadeIn, FadeOut }

#[derive(Clone)]
pub(crate) struct AutomationDragState {
    pub track_id: Uuid,
    pub parameter_name: String,
    pub point_index: usize,
    pub original_point: AutomationPoint,
    pub click_offset_time: f64,
    pub click_offset_value: f32,
}

pub(crate) fn commit_drag(
    state: &DragState,
    project: &Arc<Mutex<Project>>,
    window: &MainWindow,
    undo_stack: &Arc<Mutex<UndoStack>>,
    waveform_peaks: &Arc<Mutex<HashMap<String, WaveformPeaks>>>,
) {
    if let Ok(mut p) = project.lock() {
        let sr = p.sample_rate;
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == state.track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == state.clip_id) {
                let clips_model = window.get_clips();
                let clip_info = (0..clips_model.row_count()).find_map(|i| {
                    let ci = clips_model.row_data(i)?;
                    if ci.id.as_str() == state.clip_id.to_string().as_str() { Some(ci) } else { None }
                });
                if let Some(ci) = clip_info {
                    let pps = window.get_pixels_per_second();
                    let new_pos = pixels_to_samples(ci.x, pps, sr);
                    let new_len = pixels_to_samples(ci.width.max(4.0), pps, sr).max(1);
                    let new_fade_in = pixels_to_samples(ci.fade_in_width, pps, sr);
                    let new_fade_out = pixels_to_samples(ci.fade_out_width, pps, sr);
                    if state.drag_edge.is_none() && state.destination_track_id.is_some() {
                        let dest_id = state.destination_track_id.unwrap();
                        let clip_snapshot = clip.clone();
                        let src_id = state.track_id;
                        if let Some(src_track) = p.tracks.iter_mut().find(|t| t.id == src_id) {
                            src_track.clips.retain(|c| c.id != state.clip_id);
                        }
                        let mut moved = clip_snapshot.clone();
                        moved.position = new_pos;
                        if let Some(dest_track) = p.tracks.iter_mut().find(|t| t.id == dest_id) {
                            dest_track.add_clip(moved.clone());
                            apply_auto_crossfades(dest_track);
                        }
                        if let Ok(mut stack) = undo_stack.lock() {
                            stack.push(EditCommand::MoveClipToTrack {
                                source_track_id: src_id,
                                dest_track_id: dest_id,
                                clip_snapshot,
                            });
                        }
                        drop(p);
                        if let Ok(p) = project.lock() {
                            if let Ok(cache) = waveform_peaks.lock() {
                                sync_project_to_timeline_with_waveforms(&p, window, &cache);
                            }
                        }
                        return;
                    }
                    match state.drag_edge {
                        None => {
                            if new_pos != state.original_pos {
                                if let Ok(mut stack) = undo_stack.lock() {
                                    stack.push(EditCommand::MoveClip {
                                        track_id: state.track_id,
                                        clip_id: state.clip_id,
                                        old_pos: state.original_pos,
                                        new_pos,
                                    });
                                }
                            }
                        }
                        Some(DragEdge::Left) | Some(DragEdge::Right) => {
                            let old_start = state.original_pos;
                            let old_end = state.original_pos + state.original_length;
                            let new_start = if matches!(state.drag_edge, Some(DragEdge::Left)) {
                                new_pos
                            } else {
                                clip.position
                            };
                            let new_end = new_start + new_len;
                            if new_start != old_start || new_end != old_end {
                                if let Ok(mut stack) = undo_stack.lock() {
                                    stack.push(EditCommand::ResizeClip {
                                        track_id: state.track_id,
                                        clip_id: state.clip_id,
                                        old_start,
                                        old_end,
                                        new_start,
                                        new_end,
                                    });
                                }
                            }
                        }
                        Some(DragEdge::Stretch) => {
                            let old_len = state.original_length;
                            let old_stretch = clip.time_stretch.clone();
                            let ratio = new_len as f64 / old_len as f64;
                            let new_stretch = if (ratio - 1.0).abs() > 0.001 {
                                Some(crate::project::clip::TimeStretchParams {
                                    ratio: ratio as f32,
                                    algorithm: StretchAlgorithm::Normal,
                                })
                            } else {
                                None
                            };
                            if new_len != old_len {
                                if let Ok(mut stack) = undo_stack.lock() {
                                    stack.push(EditCommand::TimeStretch {
                                        track_id: state.track_id,
                                        clip_id: state.clip_id,
                                        old_length: old_len,
                                        old_stretch,
                                        new_length: new_len,
                                        new_stretch,
                                    });
                                }
                            }
                        }
                        Some(DragEdge::FadeIn) => {
                            let old_dur = state.original_fade_in;
                            if new_fade_in != old_dur {
                                if let Ok(mut stack) = undo_stack.lock() {
                                    stack.push(EditCommand::SetFade {
                                        track_id: state.track_id,
                                        clip_id: state.clip_id,
                                        edge: FE::In,
                                        old_dur,
                                        new_dur: new_fade_in,
                                    });
                                }
                            }
                        }
                        Some(DragEdge::FadeOut) => {
                            let old_dur = state.original_fade_out;
                            if new_fade_out != old_dur {
                                if let Ok(mut stack) = undo_stack.lock() {
                                    stack.push(EditCommand::SetFade {
                                        track_id: state.track_id,
                                        clip_id: state.clip_id,
                                        edge: FE::Out,
                                        old_dur,
                                        new_dur: new_fade_out,
                                    });
                                }
                            }
                        }
                    }
                    clip.position = new_pos;
                    clip.length = new_len;
                    clip.fade_in = new_fade_in;
                    clip.fade_out = new_fade_out;
                    if let Some(DragEdge::Stretch) = state.drag_edge {
                        let ratio = new_len as f64 / state.original_length as f64;
                        if (ratio - 1.0).abs() > 0.001 {
                            clip.time_stretch = Some(crate::project::clip::TimeStretchParams {
                                ratio: ratio as f32,
                                algorithm: StretchAlgorithm::Normal,
                            });
                        } else {
                            clip.time_stretch = None;
                        }
                    }
                }
            }
            apply_auto_crossfades(track);
        }
    }
    if let Ok(p) = project.lock() {
        if let Ok(cache) = waveform_peaks.lock() {
            sync_project_to_timeline_with_waveforms(&p, window, &cache);
        }
    }
}

pub(crate) fn apply_auto_crossfades(track: &mut Track) {
    if track.clips.len() < 2 { return; }
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..track.clips.len() - 1 {
        if clips_overlap(&track.clips[i], &track.clips[i + 1]) {
            pairs.push((i, i + 1));
        }
    }
    for (i, j) in pairs {
        let (left, right) = track.clips.split_at_mut(j);
        if let (Some(ca), Some(cb)) = (left.get_mut(i), right.first_mut()) {
            create_auto_crossfade(ca, cb);
        }
    }
}

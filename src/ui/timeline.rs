use crate::project::Project;
use crate::ui::main_window::{ClipInfo, MainWindow, TrackInfo};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::sync::{Arc, Mutex};

pub fn sync_project_to_timeline(project: &Project, window: &MainWindow) {
    let sample_rate = project.sample_rate;
    let px_per_sec = window.get_pixels_per_second() as f64;

    let clip_infos: Vec<ClipInfo> = project.tracks.iter().enumerate().flat_map(|(track_idx, track)| {
        track.clips.iter().map(|clip| {
            ClipInfo {
                id: clip.id.to_string().into(),
                track_index: track_idx as i32,
                x: (clip.position as f64 / sample_rate as f64 * px_per_sec) as f32,
                width: ((clip.length as f64 / sample_rate as f64 * px_per_sec) as f32).max(4.0),
                name: clip.name.as_str().into(),
                color: slint::Color::from_rgb_u8(clip.color.0, clip.color.1, clip.color.2).into(),
                fade_in_width: (clip.fade_in as f64 / sample_rate as f64 * px_per_sec) as f32,
                fade_out_width: (clip.fade_out as f64 / sample_rate as f64 * px_per_sec) as f32,
                selected: false,
                track_id: track.id.to_string().into(),
            }
        }).collect::<Vec<_>>()
    }).collect();

    let track_infos: Vec<TrackInfo> = project.tracks.iter().enumerate().map(|(i, track)| {
        TrackInfo {
            id: track.id.to_string().into(),
            index: i as i32,
            label: track.name.as_str().into(),
        }
    }).collect();

    window.set_clips(ModelRc::new(VecModel::from(clip_infos)));
    window.set_tracks(ModelRc::new(VecModel::from(track_infos)));
}

pub fn setup_timeline_callbacks(window: &MainWindow, project: Arc<Mutex<Project>>) {
    let window_weak = window.as_weak();
    let project_zoom = project.clone();
    window.on_zoom_in(move || {
        if let Some(w) = window_weak.upgrade() {
            let current = w.get_pixels_per_second();
            w.set_pixels_per_second((current * 1.3).min(500.0));
            if let Ok(p) = project_zoom.lock() {
                sync_project_to_timeline(&p, &w);
            }
        }
    });

    let window_weak2 = window.as_weak();
    window.on_zoom_out(move || {
        if let Some(w) = window_weak2.upgrade() {
            let current = w.get_pixels_per_second();
            w.set_pixels_per_second((current / 1.3).max(5.0));
            if let Ok(p) = project.lock() {
                sync_project_to_timeline(&p, &w);
            }
        }
    });
}

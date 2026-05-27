use crate::project::automation::AutomationPoint;
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

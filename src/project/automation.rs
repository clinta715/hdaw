use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    pub parameter_id: String,
    pub points: Vec<AutomationPoint>,
    pub enabled: bool,
    pub read_enabled: bool,
    pub write_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationPoint {
    pub time: f64,
    pub value: f32,
    pub curve_type: CurveType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    Bezier(BezierControl),
    Step,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BezierControl {
    pub handle_in_x: f64,
    pub handle_in_y: f32,
    pub handle_out_x: f64,
    pub handle_out_y: f32,
}

impl AutomationLane {
    pub fn new(parameter_id: String) -> Self {
        Self {
            parameter_id,
            points: Vec::new(),
            enabled: true,
            read_enabled: true,
            write_enabled: false,
        }
    }

    pub fn add_point(&mut self, time: f64, value: f32) {
        self.points.push(AutomationPoint {
            time,
            value,
            curve_type: CurveType::Linear,
        });
        self.points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn remove_point_at(&mut self, time: f64, tolerance: f64) {
        self.points.retain(|p| (p.time - time).abs() > tolerance);
    }

    pub fn get_value_at(&self, time: f64) -> Option<f32> {
        if self.points.is_empty() {
            return None;
        }

        if time <= self.points[0].time {
            return Some(self.points[0].value);
        }

        if time >= self.points.last()?.time {
            return Some(self.points.last()?.value);
        }

        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];

            if time >= p1.time && time <= p2.time {
                let t = (time - p1.time) / (p2.time - p1.time);
                return Some(match p1.curve_type {
                    CurveType::Linear => p1.value + (p2.value - p1.value) * t as f32,
                    CurveType::Step => p1.value,
                    CurveType::Bezier(_) => {
                        p1.value + (p2.value - p1.value) * t as f32
                    }
                });
            }
        }

        None
    }
}

impl Default for AutomationLane {
    fn default() -> Self {
        Self::new(String::new())
    }
}
use crate::jump::types::JumpPhase;
use engine::oxide::StaticImage;

#[derive(Debug, Clone)]
pub struct JumpRenderFrame {
    pub(crate) back_layer: StaticImage,
    pub(crate) front_layer: StaticImage,
    pub(crate) snow_pixels: engine::oxide::PointBatches,
    pub(crate) phase: JumpPhase,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) sx: i32,
    pub(crate) sy: i32,
    pub(crate) body_x: i32,
    pub(crate) body_y: i32,
    pub(crate) frame_counter: i32,
    pub(crate) body_anim: u16,
    pub(crate) ski_anim: u16,
    pub(crate) wind_value: i32,
    pub(crate) start_gate: i32,
    pub(crate) distance: f64,
    pub(crate) score: i32,
    pub(crate) style_points: [i32; 5],
    pub(crate) style_revealed: [bool; 5],
    pub(crate) hill_record_marker: Option<(i32, i32)>,
    pub(crate) goal_marker: Option<(i32, i32)>,
    pub(crate) is_hill_record: bool,
    pub(crate) hr_shake_position: Option<(i32, i32)>,
    pub(crate) bar_gag_position: Option<(i32, i32)>,
}

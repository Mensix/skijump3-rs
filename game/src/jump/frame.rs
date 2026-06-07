use crate::jump::types::JumpPhase;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct JumpRenderFrame {
    pub(crate) viewport: Rc<[u8]>,
    /// Indexed-pixel mask for snow position checking (`WIDTH * HEIGHT` bytes).
    /// Generated alongside the RGBA `viewport` by `HillTerrain::viewport_rgba_and_mask()`.
    pub(crate) snow_mask: Rc<[u8]>,
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
    pub(crate) distance: i32,
    pub(crate) score: i32,
    pub(crate) style_points: [i32; 5],
    pub(crate) style_revealed: [bool; 5],
    pub(crate) hill_record_marker: Option<(i32, i32)>,
}

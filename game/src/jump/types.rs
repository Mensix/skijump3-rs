#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpPhase {
    Info,
    OnBar,
    Inrun,
    Flight,
    Landing,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlightWind {
    pub(crate) value: i32,
    pub(crate) windy: i32,
    pub(crate) strength: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpInput {
    LeaveInfo,
    Start,
    Takeoff,
    LeanForward,
    LeanBack,
    Telemark,
    TwoFooted,
    AdjustGate(i32),
    ShowResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpOutcome {
    pub(crate) distance: i32,
    pub(crate) score: i32,
    pub(crate) style_points: [i32; 5],
    pub(crate) landing_style: u8,
    pub(crate) fall_type: u8,
    pub(crate) aborted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JumpSnapshot {
    pub(crate) phase: JumpPhase,
    pub(crate) frame: i32,
    pub(crate) x: i32,
    pub(crate) table_distance: f64,
    pub(crate) y: i32,
    pub(crate) height: i32,
    pub(crate) delta_height_sum: i32,
    pub(crate) slope_angle: i32,
    pub(crate) distance: i32,
    pub(crate) body_angle: i32,
    pub(crate) ski_angle: i32,
    pub(crate) speed: f64,
    pub(crate) start_gate: i32,
}

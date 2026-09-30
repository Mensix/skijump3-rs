pub(crate) const DEFAULT_START_GATE: i32 = 15;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JumpPhase {
    Info,
    OnBar,
    Inrun,
    Flight,
    Landing,
    Result,
    Disqualified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FlightWind {
    pub(crate) value: i32,
    pub(crate) windy: i32,
    pub(crate) strength: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpInput {
    LeaveInfo,
    Start,
    BarAnimation(BarAnimation),
    Takeoff,
    LeanForward,
    LeanBack,
    Telemark,
    TwoFooted,
    AdjustGate(i32),
    ShowResult,
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarAnimation {
    Up,
    Left,
    Telemark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandingStyle {
    None,
    Telemark,
    TwoFooted,
}

impl LandingStyle {
    pub(crate) const fn offset(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Telemark => 1,
            Self::TwoFooted => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FallType {
    None,
    Normal,
    TwoFooted,
    Crash,
}

impl FallType {
    pub(crate) const fn as_grade(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Normal => 1,
            Self::TwoFooted => 2,
            Self::Crash => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JumpTelemetry {
    pub grade: u8,
    pub height: u8,
    pub takeoff_timing: u8,
    pub body_angle: u8,
}

impl JumpTelemetry {
    pub fn new(grade: u8, height: u8, takeoff_timing: u8, body_angle: u8) -> Self {
        Self {
            grade,
            height,
            takeoff_timing,
            body_angle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JumpOutcome {
    pub(crate) distance: f64,
    pub(crate) score: f64,
    pub(crate) style_points: [f64; 5],
    pub(crate) landing_style: LandingStyle,
    pub(crate) fall_type: FallType,
    pub(crate) injury: u8,
    pub(crate) aborted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JumpSnapshot {
    pub(crate) phase: JumpPhase,
    pub(crate) frame: i32,
    pub(crate) x: i32,
    pub(crate) table_distance: f64,
    pub(crate) y: i32,
    pub(crate) height: i32,
    pub(crate) delta_height_sum: i32,
    pub(crate) slope_angle: i32,
    pub(crate) distance: f64,
    pub(crate) body_angle: i32,
    pub(crate) ski_angle: i32,
    pub(crate) speed: f64,
    pub(crate) start_gate: i32,
    pub(crate) bar_animation_active: bool,
    pub(crate) landing_style: LandingStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkiSwing {
    None,
    LateUp,
    ReturnUp,
    GustUp,
    LateDown,
    ReturnDown,
    GustDown,
}

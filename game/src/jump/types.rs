#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JumpPhase {
    Info,
    OnBar,
    Inrun,
    Flight,
    Landing,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FlightWind {
    pub(crate) value: i32,
    pub(crate) windy: i32,
    pub(crate) strength: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JumpInput {
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
pub(crate) struct JumpOutcome {
    pub(crate) distance: i32,
    pub(crate) score: i32,
    pub(crate) style_points: [i32; 5],
    pub(crate) landing_style: u8,
    pub(crate) fall_type: u8,
    pub(crate) aborted: bool,
}

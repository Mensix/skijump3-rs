#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum JumperControl {
    Human,
    Computer,
    Replay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JumpPolicy {
    pub(crate) allow_start_gate_adjust: bool,
    pub(crate) allow_wind_reset: bool,
    pub(crate) count_onbar_frames: bool,
    pub(crate) save_hill_records: bool,
    pub(crate) control: JumperControl,
}

impl JumpPolicy {
    pub(crate) const fn training() -> Self {
        Self {
            allow_start_gate_adjust: true,
            allow_wind_reset: true,
            count_onbar_frames: false,
            save_hill_records: false,
            control: JumperControl::Human,
        }
    }

    pub(crate) const fn competition() -> Self {
        Self {
            allow_start_gate_adjust: false,
            allow_wind_reset: false,
            count_onbar_frames: true,
            save_hill_records: true,
            control: JumperControl::Human,
        }
    }
}

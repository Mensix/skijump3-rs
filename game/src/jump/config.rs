use crate::data::hill_profile::HillTerrain;
use crate::data::records::HillInfo;
use crate::jump::policy::{JumpPolicy, JumperControl};
use crate::jump::snow::SnowSystem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JumpParticipant {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) real_name: String,
    pub(crate) suit_color: u8,
    pub(crate) ski_color: u8,
    pub(crate) team: Option<usize>,
    pub(crate) control: JumperControl,
}

impl JumpParticipant {
    pub(crate) fn trainee() -> Self {
        Self {
            id: 0,
            name: "TRAINEE".to_string(),
            real_name: String::new(),
            suit_color: 0,
            ski_color: 0,
            team: None,
            control: JumperControl::Human,
        }
    }

    pub(crate) fn display_name(&self) -> &str {
        if self.real_name.is_empty() {
            &self.name
        } else {
            &self.real_name
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct JumpConfig {
    pub(crate) hill_idx: usize,
    pub(crate) hill: Option<HillInfo>,
    pub(crate) terrain: Result<HillTerrain, String>,
    pub(crate) start_gate: i32,
    pub(crate) snow: SnowSystem,
    pub(crate) participant: JumpParticipant,
    pub(crate) policy: JumpPolicy,
    pub(crate) record_distance: i32,
    pub(crate) phase_label: String,
}

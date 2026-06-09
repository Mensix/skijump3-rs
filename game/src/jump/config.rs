use crate::data::hill::HillInfo;
use crate::data::hill_profile::HillTerrain;
use crate::error::AssetError;
use crate::jump::policy::{JumpPolicy, JumperControl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpParticipant {
    pub(crate) id: usize,
    pub(crate) ai_id: usize,
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
            ai_id: 0,
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
pub struct JumpConfig {
    pub(crate) hill_idx: usize,
    pub(crate) hill: Option<HillInfo>,
    pub(crate) terrain: Result<HillTerrain, AssetError>,
    pub(crate) start_gate: i32,
    pub(crate) snow_count: u16,
    pub(crate) participant: JumpParticipant,
    pub(crate) policy: JumpPolicy,
    pub(crate) record_distance: i32,
    pub(crate) phase_label: String,
    pub(crate) team_name: String,
}

use crate::data::hill::HillInfo;
use crate::data::hill_profile::HillTerrain;
use crate::gfx::color::Rgb6;
use crate::jump::policy::{JumpPolicy, JumperControl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpParticipant {
    pub(crate) id: usize,
    pub(crate) ai_id: usize,
    pub(crate) name: String,
    pub(crate) real_name: String,
    pub(crate) suit_color: Rgb6,
    pub(crate) ski_color: Rgb6,
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
            suit_color: Rgb6([53, 17, 53]),
            ski_color: Rgb6([63, 63, 32]),
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
    pub(crate) is_custom_hill: bool,
    pub(crate) terrain: HillTerrain,
    pub(crate) start_gate: i32,
    pub(crate) snow_count: u16,
    pub(crate) participant: JumpParticipant,
    pub(crate) policy: JumpPolicy,
    pub(crate) record_distance: f64,
    pub(crate) goal_distance: f64,
    pub(crate) phase_label: String,
    pub(crate) team_name: String,
    pub(crate) replay_competition_code: Option<i32>,
}

use crate::competition::types::Participant;
use crate::data::hill_profile::HillTerrain;
use crate::data::records::HillInfo;
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

impl From<&Participant> for JumpParticipant {
    fn from(p: &Participant) -> Self {
        Self {
            id: p.id,
            ai_id: p.ai_id,
            name: p.name.clone(),
            real_name: p.real_name.clone(),
            suit_color: p.suit_color,
            ski_color: p.ski_color,
            team: p.team,
            control: if p.is_computer {
                JumperControl::Computer
            } else {
                JumperControl::Human
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct JumpConfig {
    pub(crate) hill_idx: usize,
    pub(crate) hill: Option<HillInfo>,
    pub(crate) terrain: Result<HillTerrain, String>,
    pub(crate) start_gate: i32,
    pub(crate) snow_count: u16,
    pub(crate) participant: JumpParticipant,
    pub(crate) policy: JumpPolicy,
    pub(crate) record_distance: i32,
    pub(crate) phase_label: String,
}

use crate::competition::core::competitor::Competitor;
use crate::gfx::color::Rgb6;
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::types::FallType;
use serde::{Deserialize, Serialize};

pub(crate) const MAX_CUSTOM_CUP_HILLS: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct CompetitionJumpOutcome {
    pub(crate) score: f64,
    pub(crate) distance: f64,
    pub(crate) fall_type: FallType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CupStyle {
    WorldCup,
    CustomCup,
    FourHills,
    TeamCup,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomCupScoring {
    #[default]
    WorldCupPoints,
    AggregateJumpPoints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompetitionPhase {
    Setup,
    Training(usize),
    Qualification,
    QualificationResults,
    Round1,
    Round1Results,
    Round2,
    Round2Results,
    FourHillsStandings,
    WorldCupStandings,
    EventComplete,
    SeasonComplete,
}

impl CompetitionPhase {
    pub const fn is_jump_phase(self) -> bool {
        matches!(
            self,
            Self::Training(_) | Self::Qualification | Self::Round1 | Self::Round2
        )
    }

    pub const fn is_result_phase(self) -> bool {
        matches!(
            self,
            Self::QualificationResults
                | Self::Round1Results
                | Self::Round2Results
                | Self::FourHillsStandings
                | Self::WorldCupStandings
                | Self::SeasonComplete
        )
    }

    pub const fn auto_advances_when_empty(self) -> bool {
        matches!(
            self,
            Self::Training(_)
                | Self::Setup
                | Self::Qualification
                | Self::Round1
                | Self::Round2
                | Self::EventComplete
        )
    }

    pub const fn needs_event_results(self) -> bool {
        matches!(self, Self::Qualification | Self::Round1 | Self::Round2)
    }

    pub const fn result_round_number(self) -> Option<usize> {
        match self {
            Self::Round1Results => Some(1),
            Self::Round2Results => Some(2),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualificationStatus {
    #[default]
    NotQualified,
    Qualified,
    PreQualified,
    LuckyLoser,
    KoSeed(usize),
    Eliminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventHistoryReason {
    Injury(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EventJumpHistory {
    pub distance: f64,
    pub points: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndividualEventHistory {
    pub event_index: usize,
    pub hill_idx: usize,
    pub qualification: Option<EventJumpHistory>,
    pub qualification_status: QualificationStatus,
    pub round1: Option<EventJumpHistory>,
    pub round2: Option<EventJumpHistory>,
    pub event_points: Option<f64>,
    pub placing: Option<usize>,
    pub wc_points_awarded: i32,
    pub running_rank: usize,
    pub status: QualificationStatus,
    pub reason: Option<EventHistoryReason>,
}

impl QualificationStatus {
    pub const fn can_jump(&self) -> bool {
        matches!(
            self,
            Self::Qualified | Self::PreQualified | Self::LuckyLoser | Self::KoSeed(_)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: usize,
    pub ai_id: usize,
    pub name: String,
    pub real_name: String,
    pub suit_color: Rgb6,
    pub ski_color: Rgb6,
    pub team: Option<usize>,
    pub is_computer: bool,
    pub skip_qualification: u8,
    pub profile_idx: Option<usize>,

    pub wc_points: i32,
    pub four_hills_points: f64,
    pub injury: u8,

    pub points: Option<f64>,
    pub rank: usize,
    pub round1_rank: usize,
    pub qual: QualificationStatus,
    pub round1_len: f64,
    pub round1_score: f64,
    pub round2_len: f64,
    pub qual_len: f64,
    pub leg_wins: usize,
    pub event_history: Vec<IndividualEventHistory>,

    pub(crate) qual_score: Option<f64>,
    pub(crate) qualification_result: QualificationStatus,
    pub(crate) round2_score: f64,
}

impl Participant {
    pub fn from_competitor(competitor: Competitor) -> Self {
        Self {
            id: competitor.id,
            ai_id: competitor.ai_id,
            name: competitor.name,
            real_name: competitor.real_name,
            suit_color: competitor.suit_color,
            ski_color: competitor.ski_color,
            team: competitor.team,
            is_computer: competitor.is_computer,
            skip_qualification: 0,
            profile_idx: competitor.profile_idx,
            wc_points: 0,
            four_hills_points: 0.0,
            injury: 0,
            points: None,
            rank: 0,
            round1_rank: 0,
            qual: QualificationStatus::NotQualified,
            round1_len: 0.0,
            round1_score: 0.0,
            round2_len: 0.0,
            qual_len: 0.0,
            leg_wins: 0,
            event_history: Vec::new(),
            qual_score: None,
            qualification_result: QualificationStatus::NotQualified,
            round2_score: 0.0,
        }
    }

    pub fn computer(id: usize, ai_id: usize, name: String) -> Self {
        Self::from_competitor(Competitor::computer(id, ai_id, name, None))
    }

    pub fn display_name(&self) -> &str {
        if self.real_name.is_empty() {
            &self.name
        } else {
            &self.real_name
        }
    }

    pub fn to_jump_participant(&self) -> JumpParticipant {
        JumpParticipant {
            id: self.id,
            ai_id: self.ai_id,
            name: self.name.clone(),
            real_name: self.real_name.clone(),
            suit_color: self.suit_color,
            ski_color: self.ski_color,
            team: self.team,
            control: if self.is_computer {
                JumperControl::Computer
            } else {
                JumperControl::Human
            },
        }
    }

    pub const fn reset_event(&mut self) {
        self.points = None;
        self.rank = 0;
        self.round1_rank = 0;
        self.qual = QualificationStatus::NotQualified;
        self.round1_len = 0.0;
        self.round1_score = 0.0;
        self.round2_len = 0.0;
        self.qual_len = 0.0;
        self.qual_score = None;
        self.qualification_result = QualificationStatus::NotQualified;
        self.round2_score = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_jump_returns_true_for_qualified() {
        assert!(QualificationStatus::Qualified.can_jump());
        assert!(QualificationStatus::PreQualified.can_jump());
        assert!(QualificationStatus::LuckyLoser.can_jump());
        assert!(QualificationStatus::KoSeed(5).can_jump());
        assert!(!QualificationStatus::NotQualified.can_jump());
        assert!(!QualificationStatus::Eliminated.can_jump());
    }

    #[test]
    fn reset_event_clears_per_event_state() {
        let mut p = Participant::computer(0, 0, "Test".into());
        p.points = Some(500.0);
        p.rank = 1;
        p.round1_len = 120.0;
        p.round2_len = 130.0;
        p.qual = QualificationStatus::Qualified;

        p.reset_event();

        assert_eq!(p.points, None);
        assert_eq!(p.rank, 0);
        assert_eq!(p.round1_len, 0.0);
        assert_eq!(p.round2_len, 0.0);
        assert_eq!(p.qual, QualificationStatus::NotQualified);
        assert!(p.event_history.is_empty());
    }
}

use crate::competition::core::competitor::Competitor;
use crate::jump::types::FallType;
use serde::{Deserialize, Serialize};

/// A jump result as recorded into the competition. Carries only the fields
/// the competition cares about: score, distance, and whether the jumper
/// crashed (which may cause an injury).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct CompetitionJumpOutcome {
    pub(crate) score: f64,
    pub(crate) distance: f64,
    pub(crate) fall_type: FallType,
}

/// Identifies the scoring/ruleset for a competition series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CupStyle {
    WorldCup,
    CustomCup,
    FourHills,
    TeamCup,
}

/// High-level phase of a single competition event.
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
    #[must_use]
    pub const fn is_jump_phase(self) -> bool {
        matches!(
            self,
            Self::Training(_) | Self::Qualification | Self::Round1 | Self::Round2
        )
    }

    #[must_use]
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

    /// Phases that auto-advance when the start list is empty
    /// (no human interaction needed).
    #[must_use]
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

    /// Phases whose overlay needs live event results (top5, gap-to-leader).
    #[must_use]
    pub const fn needs_event_results(self) -> bool {
        matches!(self, Self::Qualification | Self::Round1 | Self::Round2)
    }

    /// Result phase that corresponds to a numbered round (1 or 2).
    #[must_use]
    pub const fn result_round_number(self) -> Option<usize> {
        match self {
            Self::Round1Results => Some(1),
            Self::Round2Results => Some(2),
            _ => None,
        }
    }
}

/// Whether a jumper made the cut for the current round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualificationStatus {
    NotQualified,
    Qualified,
    PreQualified,
    LuckyLoser,
    KoSeed(usize),
    Eliminated,
}

impl QualificationStatus {
    #[must_use]
    pub const fn can_jump(&self) -> bool {
        matches!(
            self,
            Self::Qualified | Self::PreQualified | Self::LuckyLoser | Self::KoSeed(_)
        )
    }
}

/// One jumper in the Pascal 75-slot competition roster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: usize,
    pub ai_id: usize,
    pub name: String,
    pub real_name: String,
    pub suit_color: u8,
    pub ski_color: u8,
    pub team: Option<usize>,
    pub is_computer: bool,
    /// 0=never skip, 1=skip unless 4H, 2=always skip
    pub skip_quali: u8,
    /// Index into ProfileStore.profiles for human participants, None for computers.
    pub profile_idx: Option<usize>,

    // Season-wide state
    pub wc_points: i32,
    pub four_hills_points: f64,
    pub injury: u8,

    // Per-event state. None = hasn't started (DNS)
    pub points: Option<f64>,
    /// Current live rank in the event standings (updated by `sort_field` after each jump).
    pub rank: usize,
    /// Frozen Round 1 rank stored before Round 2 starts (Pascal's `sija` from `luett`).
    pub round1_rank: usize,
    pub qual: QualificationStatus,
    pub round1_len: f64,
    pub round1_score: f64,
    pub round2_len: f64,
    pub qual_len: f64,
}

impl Participant {
    #[must_use]
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
            skip_quali: 0,
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
        }
    }

    #[must_use]
    pub fn computer(id: usize, ai_id: usize, name: String) -> Self {
        Self::from_competitor(Competitor::computer(id, ai_id, name, None))
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        if self.real_name.is_empty() {
            &self.name
        } else {
            &self.real_name
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computer_participant_defaults() {
        let p = Participant::computer(1, 0, "Test".into());
        assert_eq!(p.id, 1);
        assert_eq!(p.ai_id, 0);
        assert!(p.is_computer);
        assert_eq!(p.wc_points, 0);
        assert!(!p.qual.can_jump());
    }

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
    }
}

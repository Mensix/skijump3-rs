/// Identifies the scoring/ruleset for a competition series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CupStyle {
    WorldCup,
    CustomCup,
    FourHills,
    TeamCup,
}

/// High-level phase of a single competition event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionPhase {
    Setup,
    Training(usize),
    Qualification,
    QualificationResults,
    Round1,
    Round1Results,
    Round2,
    Round2Results,
    WorldCupStandings,
    EventComplete,
    SeasonComplete,
}

/// Whether a jumper made the cut for the current round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Self::Qualified
                | Self::PreQualified
                | Self::LuckyLoser
                | Self::KoSeed(_)
        )
    }
}

/// One jumper in the competition field (75 total: user profiles + computer opponents).
#[derive(Debug, Clone)]
pub struct Participant {
    pub id: usize,
    pub name: String,
    pub real_name: String,
    pub suit_color: u8,
    pub ski_color: u8,
    pub team: Option<usize>,
    pub is_computer: bool,

    // Season-wide state
    pub wc_points: i32,
    pub four_hills_points: i32,
    pub injury: u8,

    // Per-event state
    pub points: i32,
    pub rank: usize,
    pub qual: QualificationStatus,
    pub round1_len: i32,
    pub round2_len: i32,
    pub qual_len: i32,
}

impl Participant {
    #[must_use] 
    pub const fn computer(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            real_name: String::new(),
            suit_color: 0,
            ski_color: 0,
            team: None,
            is_computer: true,
            wc_points: 0,
            four_hills_points: 0,
            injury: 0,
            points: 0,
            rank: 0,
            qual: QualificationStatus::NotQualified,
            round1_len: 0,
            round2_len: 0,
            qual_len: 0,
        }
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
        self.points = 0;
        self.rank = 0;
        self.qual = QualificationStatus::NotQualified;
        self.round1_len = 0;
        self.round2_len = 0;
        self.qual_len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computer_participant_defaults() {
        let p = Participant::computer(1, "Test".into());
        assert_eq!(p.id, 1);
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
        let mut p = Participant::computer(0, "Test".into());
        p.points = 500;
        p.rank = 1;
        p.round1_len = 120;
        p.round2_len = 130;
        p.qual = QualificationStatus::Qualified;

        p.reset_event();

        assert_eq!(p.points, 0);
        assert_eq!(p.rank, 0);
        assert_eq!(p.round1_len, 0);
        assert_eq!(p.round2_len, 0);
        assert_eq!(p.qual, QualificationStatus::NotQualified);
    }
}

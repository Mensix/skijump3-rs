#![allow(dead_code)]

use crate::jump::config::JumpParticipant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CupStyle {
    WorldCup,
    CustomCup,
    FourHills,
    TeamCup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualificationStatus {
    NotQualified,
    Qualified,
    PreQualified,
    LuckyLoser,
    KoSeed(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartListEntry {
    pub participant_id: usize,
    pub team_id: Option<usize>,
    pub qualification: QualificationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumperStanding {
    pub participant_id: usize,
    pub rank: usize,
    pub points: i32,
    pub total_distance: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundState {
    pub round_index: i32,
    pub start_list: Vec<StartListEntry>,
    pub standings: Vec<JumperStanding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompetitionState {
    pub hill_idx: usize,
    pub round: RoundState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldCupState {
    pub style: CupStyle,
    pub hill_order: Vec<usize>,
    pub competition_index: usize,
    pub participants: Vec<JumpParticipant>,
    pub standings: Vec<JumperStanding>,
    pub competition: Option<CompetitionState>,
}

impl WorldCupState {
    pub fn new(
        style: CupStyle,
        hill_order: Vec<usize>,
        participants: Vec<JumpParticipant>,
    ) -> Self {
        Self {
            style,
            hill_order,
            competition_index: 0,
            participants,
            standings: Vec::new(),
            competition: None,
        }
    }
}

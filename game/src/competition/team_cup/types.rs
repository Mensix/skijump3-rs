use crate::competition::core::competitor::Competitor;
use crate::jump::config::JumpParticipant;

pub const NUM_TEAMS: usize = 15;
pub const MEMBERS_PER_TEAM: usize = 4;
pub const NUM_LEGS: usize = 6;

#[derive(Debug, Clone)]
pub struct TeamCupRuntime {
    pub teams: Vec<TeamCupTeam>,
    pub schedule: Vec<usize>,
    pub current_leg: usize,
    pub current_round: usize,
    pub current_jumper_slot: usize,
    pub current_team_order_pos: usize,
    pub team_order: Vec<usize>,
    pub phase: TeamCupPhase,
    pub human_teams: usize,
}

#[derive(Debug, Clone)]
pub struct TeamCupTeam {
    pub id: usize,
    pub name: String,
    pub members: Vec<TeamCupMember>,
    pub leg_score: i32,
    pub cup_points: i32,
    pub is_human_team: bool,
}

#[derive(Debug, Clone)]
pub struct TeamCupMember {
    pub competitor: Competitor,
    pub jumps: Vec<TeamCupJumpResult>,
}

impl TeamCupMember {
    pub fn to_jump_participant(&self) -> JumpParticipant {
        self.competitor.to_jump_participant()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamCupJumpResult {
    pub leg: usize,
    pub round: usize,
    pub distance: i32,
    pub score: i32,
    pub gate: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamCupPhase {
    Setup,
    Jumping,
    LegResults,
    TeamCupStandings,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamCupResultsKind {
    LegResults,
    Standings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamCupStandingsKind {
    Leg,
    Overall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamCupJumpContext {
    pub leg_idx: usize,
    pub round_idx: usize,
    pub team_idx: usize,
    pub member_idx: usize,
    pub team_name: String,
    pub jumper_name: String,
    pub jumper_in_team: usize,
}
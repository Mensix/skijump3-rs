use crate::competition::core::competitor::Competitor;
use crate::jump::config::JumpParticipant;
use crate::rng::Random;

#[derive(Debug, Clone)]
pub struct KothRuntime {
    pub participants: Vec<KothParticipant>,
    pub human_indices: Vec<usize>,
    pub hill_idx: usize,
    pub jump_rounds_per_elimination: u8,
    pub phase: KothPhase,
    pub current_elimination_round: u8,
    pub current_jump_round: u8,
    pub current_participant_pos: usize,
    pub rng: Random,
}

#[derive(Debug, Clone)]
pub struct KothParticipant {
    pub competitor: Competitor,
    pub total_points: f64,
    pub eliminated_in_round: u8,
    pub jumps: Vec<KothJumpResult>,
}

impl KothParticipant {
    pub fn to_jump_participant(&self) -> JumpParticipant {
        self.competitor.to_jump_participant()
    }

    pub fn is_alive(&self) -> bool {
        self.eliminated_in_round == 0
    }

    pub fn reset_points(&mut self) {
        self.total_points = 0.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KothJumpResult {
    pub elimination_round: u8,
    pub jump_round: u8,
    pub distance: f64,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KothPhase {
    Setup,
    Jumping,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KothResultsKind {
    Results,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KothJumpContext {
    pub participant_idx: usize,
    pub elimination_round: u8,
    pub jump_round: u8,
    pub starting_count: usize,
}

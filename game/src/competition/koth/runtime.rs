use super::types::{KothJumpContext, KothJumpResult, KothPhase, KothResultsKind, KothRuntime};
use crate::competition::runtime::{
    CompetitionDecision, CompetitionJumpMetadata, CompetitionRuntime,
};
use crate::jump::types::JumpOutcome;
use crate::rng::Random;

impl KothRuntime {
    pub fn new(
        participants: Vec<super::types::KothParticipant>,
        human_indices: Vec<usize>,
        hill_idx: usize,
        jump_rounds_per_elimination: u8,
        rng: Random,
    ) -> Self {
        Self {
            participants,
            human_indices,
            hill_idx,
            jump_rounds_per_elimination,
            phase: KothPhase::Setup,
            current_elimination_round: 0,
            current_jump_round: 0,
            current_participant_pos: 0,
            rng,
        }
    }

    fn count_alive(&self) -> usize {
        self.participants.iter().filter(|p| p.is_alive()).count()
    }

    fn reset_alive_points(&mut self) {
        for p in &mut self.participants {
            if p.is_alive() {
                p.reset_points();
            }
        }
    }

    fn eliminate_lowest(&mut self) {
        let alive: Vec<(usize, f64)> = self
            .participants
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_alive())
            .map(|(i, p)| (i, p.total_points))
            .collect();

        if alive.is_empty() || alive.len() == 1 {
            return;
        }

        let mut sorted = alive;
        sorted.sort_by(|(_, a), (_, b)| b.total_cmp(a));

        let mut i = 0;
        while i + 1 < sorted.len() {
            if (sorted[i].1 - sorted[i + 1].1).abs() < f64::EPSILON && self.rng.random_i32(2) == 0 {
                sorted.swap(i, i + 1);
            }
            i += 1;
        }

        if let Some(&(idx, _)) = sorted.last() {
            self.participants[idx].eliminated_in_round = self.current_elimination_round;
        }
    }
}

impl CompetitionRuntime for KothRuntime {
    type Context = KothJumpContext;
    type ResultsKind = KothResultsKind;

    fn decide_next_runtime(&mut self) -> CompetitionDecision<Self::Context, Self::ResultsKind> {
        loop {
            match self.phase {
                KothPhase::Setup => {
                    self.phase = KothPhase::Jumping;
                    self.current_elimination_round = 0;
                    self.current_jump_round = 0;
                    self.current_participant_pos = 0;
                    self.reset_alive_points();
                }

                KothPhase::Jumping => {
                    while self.current_participant_pos < self.participants.len() {
                        let p = &self.participants[self.current_participant_pos];
                        if p.is_alive() {
                            let is_human =
                                self.human_indices.contains(&self.current_participant_pos);
                            return CompetitionDecision::Jump {
                                participant: p.to_jump_participant(),
                                hill_idx: self.hill_idx,
                                context: KothJumpContext {
                                    participant_idx: self.current_participant_pos,
                                    start_order_pos: self.current_participant_pos,
                                    elimination_round: self.current_elimination_round,
                                    jump_round: self.current_jump_round,
                                    starting_count: self.participants.len(),
                                },
                                is_human,
                            };
                        }
                        self.current_participant_pos += 1;
                    }

                    if self.current_jump_round == 0 && self.jump_rounds_per_elimination > 1 {
                        self.current_jump_round = 1;
                        self.current_participant_pos = 0;
                        continue;
                    }

                    self.eliminate_lowest();

                    if self.count_alive() <= 1 {
                        self.phase = KothPhase::Complete;
                    }

                    return CompetitionDecision::ShowResults(KothResultsKind::Results);
                }

                KothPhase::Complete => {
                    return CompetitionDecision::Done;
                }
            }
        }
    }

    fn record_jump_runtime(&mut self, context: &Self::Context, outcome: JumpOutcome) {
        if let Some(p) = self.participants.get_mut(context.participant_idx) {
            p.jumps.push(KothJumpResult {
                elimination_round: context.elimination_round,
                jump_round: context.jump_round,
                distance: outcome.distance,
                score: outcome.score,
            });
            p.total_points += outcome.score;
        }
        self.current_participant_pos += 1;
    }

    fn advance_results_runtime(&mut self) {
        if self.phase == KothPhase::Complete {
            return;
        }

        self.current_elimination_round += 1;
        self.current_jump_round = 0;
        self.current_participant_pos = 0;
        self.reset_alive_points();
        self.phase = KothPhase::Jumping;
    }

    fn is_complete_runtime(&self) -> bool {
        self.phase == KothPhase::Complete
    }

    fn current_jump_context(&self) -> Self::Context {
        let idx = self
            .current_participant_pos
            .min(self.participants.len().saturating_sub(1));
        KothJumpContext {
            participant_idx: idx,
            start_order_pos: idx,
            elimination_round: self.current_elimination_round,
            jump_round: self.current_jump_round,
            starting_count: self.participants.len(),
        }
    }

    fn jump_metadata(&self, context: &Self::Context) -> CompetitionJumpMetadata {
        let competitor = self
            .participants
            .get(context.participant_idx)
            .map(|participant| &participant.competitor);
        CompetitionJumpMetadata {
            profile_idx: competitor.and_then(|competitor| competitor.profile_idx),
            hill_idx: self.hill_idx,
            jumper_name: competitor.map_or_else(String::new, |competitor| competitor.name.clone()),
            saves_hill_records: false,
            is_computer: competitor.is_some_and(|competitor| competitor.is_computer),
            is_real_world_cup: false,
        }
    }

    fn start_order_pos_for_context(&self, context: &Self::Context) -> usize {
        context.start_order_pos
    }

    fn event_idx(&self) -> usize {
        0
    }
}

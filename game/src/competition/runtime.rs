use crate::competition::machine::{Competition, StepDecision};
use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::jump::config::JumpParticipant;
use crate::jump::types::JumpOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompetitionDecision<C, R> {
    Jump {
        participant: JumpParticipant,
        hill_idx: usize,
        context: C,
        is_human: bool,
    },
    ShowResults(R),
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompetitionJumpMetadata {
    pub(crate) profile_idx: Option<usize>,
    pub(crate) hill_idx: usize,
    pub(crate) jumper_name: String,
    pub(crate) saves_hill_records: bool,
    pub(crate) is_computer: bool,
    pub(crate) is_real_world_cup: bool,
}

pub trait CompetitionRuntime {
    type Context: Clone;
    type ResultsKind: Copy + Eq;

    fn decide_next_runtime(&mut self) -> CompetitionDecision<Self::Context, Self::ResultsKind>;
    fn record_jump_runtime(&mut self, context: &Self::Context, outcome: JumpOutcome);
    fn advance_results_runtime(&mut self);
    fn is_complete_runtime(&self) -> bool;

    fn current_jump_context(&self) -> Self::Context;

    fn jump_metadata(&self, context: &Self::Context) -> CompetitionJumpMetadata;
    fn start_order_pos_for_context(&self, context: &Self::Context) -> usize;

    fn event_idx(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndividualJumpContext {
    pub event_idx: usize,
    pub phase: CompetitionPhase,
    pub participant_idx: usize,
    pub start_order_pos: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndividualResultsKind {
    Results,
}

impl CompetitionRuntime for Competition {
    type Context = IndividualJumpContext;
    type ResultsKind = IndividualResultsKind;

    fn decide_next_runtime(&mut self) -> CompetitionDecision<Self::Context, Self::ResultsKind> {
        loop {
            match self.decide_next() {
                StepDecision::ShowResults => {
                    return CompetitionDecision::ShowResults(IndividualResultsKind::Results);
                }
                StepDecision::AdvancePhase => {
                    self.advance();
                }
                StepDecision::Skip => {
                    self.skip_current_jumper();
                }
                StepDecision::Jump {
                    idx,
                    hill_idx,
                    is_human,
                } => {
                    let phase = self.phase();
                    return CompetitionDecision::Jump {
                        participant: self.participant(idx).to_jump_participant(),
                        hill_idx,
                        context: IndividualJumpContext {
                            event_idx: self.current_event,
                            phase,
                            participant_idx: idx,
                            start_order_pos: self.current_start_order_pos(),
                        },
                        is_human,
                    };
                }
            }
        }
    }

    fn record_jump_runtime(&mut self, _: &Self::Context, outcome: JumpOutcome) {
        self.apply_jump_outcome(outcome);
        self.advance();
    }

    fn advance_results_runtime(&mut self) {
        self.advance();
    }

    fn is_complete_runtime(&self) -> bool {
        self.is_over()
    }

    fn current_jump_context(&self) -> Self::Context {
        let participant_idx = self.current_jumper().unwrap_or(0);
        IndividualJumpContext {
            event_idx: self.current_event,
            phase: self.phase(),
            participant_idx,
            start_order_pos: self.current_start_order_pos(),
        }
    }

    fn jump_metadata(&self, context: &Self::Context) -> CompetitionJumpMetadata {
        let participant = self.participant(context.participant_idx);
        CompetitionJumpMetadata {
            profile_idx: participant.profile_idx,
            hill_idx: self.hill_order.get(context.event_idx).copied().unwrap_or(0),
            jumper_name: participant.name.clone(),
            saves_hill_records: !matches!(context.phase, CompetitionPhase::Training(_)),
            is_computer: participant.is_computer,
            is_real_world_cup: self.style() == CupStyle::WorldCup
                && !matches!(context.phase, CompetitionPhase::Training(_)),
        }
    }

    fn start_order_pos_for_context(&self, context: &Self::Context) -> usize {
        context.start_order_pos
    }

    fn event_idx(&self) -> usize {
        self.current_event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::competition::types::{CupStyle, Participant};

    #[test]
    fn human_training_jump_is_not_hidden() {
        let computer = Participant::computer(0, 0, "CPU".into());
        let mut human = Participant::computer(1, 1, "HUMAN".into());
        human.is_computer = false;
        let mut competition = Competition::new(CupStyle::WorldCup, vec![computer, human], vec![0]);
        competition.training_rounds = 1;

        let decision = competition.decide_next_runtime();

        match decision {
            CompetitionDecision::Jump {
                context,
                is_human,
                participant,
                ..
            } => {
                assert_eq!(context.phase, CompetitionPhase::Training(1));
                assert_eq!(participant.name, "HUMAN");
                assert!(is_human);
                assert_eq!(
                    competition.jump_metadata(&context),
                    CompetitionJumpMetadata {
                        profile_idx: None,
                        hill_idx: 0,
                        jumper_name: "HUMAN".into(),
                        saves_hill_records: false,
                        is_computer: false,
                        is_real_world_cup: false,
                    }
                );
            }
            _ => panic!("expected visible human training jump, got {decision:?}"),
        }
    }
}

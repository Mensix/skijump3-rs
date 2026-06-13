use crate::competition::machine::{Competition, StepDecision};
use crate::competition::types::{CompetitionPhase, Participant};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::types::JumpOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompetitionDecision<C, R> {
    Jump {
        participant: JumpParticipant,
        hill_idx: usize,
        context: C,
        is_human: bool,
        is_new_event: bool,
    },
    ShowResults(R),
    Done,
}

pub trait CompetitionRuntime {
    type Context: Clone;
    type ResultsKind: Copy + Eq;

    fn decide_next_runtime(&mut self) -> CompetitionDecision<Self::Context, Self::ResultsKind>;
    fn record_jump_runtime(&mut self, context: &Self::Context, outcome: JumpOutcome);
    fn advance_results_runtime(&mut self);
    fn is_complete_runtime(&self) -> bool;

    /// Whether the current jumper is human (needs UI).
    fn is_human_current(&self) -> bool;

    /// Context for the current jump (used by session to record outcome).
    fn current_jump_context(&self) -> Self::Context;

    /// Current event/leg index (for new-event detection).
    /// Returns 0 for single-event competitions.
    fn event_idx(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndividualJumpContext {
    pub event_idx: usize,
    pub phase: CompetitionPhase,
    pub participant_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndividualResultsKind {
    Results,
}

pub(crate) fn participant_to_jump(p: &Participant) -> JumpParticipant {
    JumpParticipant {
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
                    let is_training = matches!(phase, CompetitionPhase::Training(_));
                    return CompetitionDecision::Jump {
                        participant: participant_to_jump(self.participant(idx)),
                        hill_idx,
                        context: IndividualJumpContext {
                            event_idx: self.current_event,
                            phase,
                            participant_idx: idx,
                        },
                        is_human: is_human && !is_training,
                        is_new_event: false,
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

    fn is_human_current(&self) -> bool {
        self.is_human_current()
    }

    fn current_jump_context(&self) -> Self::Context {
        let participant_idx = self.current_jumper().unwrap_or(0);
        IndividualJumpContext {
            event_idx: self.current_event,
            phase: self.phase(),
            participant_idx,
        }
    }

    fn event_idx(&self) -> usize {
        self.current_event
    }
}

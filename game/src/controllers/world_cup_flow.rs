use crate::competition::machine::StepDecision;
use crate::competition::types::{CompetitionPhase, Participant};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::types::JumpOutcome;

use crate::store::{ResourcesRef, StoreRef};
use std::cell::Cell;

pub enum WorldCupCommand {
    HumanJump {
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
    },
    ShowResults,
    Done,
}

/// Convert a competition Participant to a jump-domain JumpParticipant.
/// Lives here (the boundary) so neither `jump` nor `competition` needs
/// to know about the other.
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

pub struct WorldCupFlow {
    last_event: Cell<usize>,
}

impl WorldCupFlow {
    pub(crate) const fn new() -> Self {
        Self {
            last_event: Cell::new(0),
        }
    }

    /// Advance competition state until the next user-visible moment.
    ///
    /// `simulate_computer` is called for each computer jumper encountered
    /// in the loop.  The flow records the outcome and advances the
    /// competition internally — the caller need only provide the
    /// simulation plumbing.
    pub(crate) fn drive(
        &self,
        resources: &ResourcesRef,
        store: &StoreRef,
        simulate_computer: &mut dyn FnMut(JumpParticipant, usize) -> JumpOutcome,
    ) -> WorldCupCommand {
        loop {
            let comp = store.competition.borrow();
            let Some(c) = comp.as_ref() else {
                return WorldCupCommand::Done;
            };

            match c.decide_next() {
                StepDecision::Done => return WorldCupCommand::Done,
                StepDecision::ShowResults => return WorldCupCommand::ShowResults,
                StepDecision::AdvancePhase => {
                    drop(comp);
                    store.competition.borrow_mut().as_mut().unwrap().advance();
                    continue;
                }
                StepDecision::Jump {
                    idx,
                    hill_idx,
                    is_human: true,
                } => {
                    let participant = participant_to_jump(c.field.get(idx));
                    let label = Self::phase_label(resources, c.phase);
                    drop(comp);
                    return WorldCupCommand::HumanJump {
                        participant,
                        hill_idx,
                        phase_label: label,
                    };
                }
                StepDecision::Jump {
                    idx,
                    hill_idx,
                    is_human: false,
                } => {
                    let participant = participant_to_jump(c.field.get(idx));
                    drop(comp);

                    let outcome = simulate_computer(participant, hill_idx);

                    let mut comp = store.competition.borrow_mut();
                    let c = comp.as_mut().unwrap();
                    c.record_jump(outcome.score, outcome.distance);
                    c.advance();

                    if c.is_over() {
                        self.note_event_change(store);
                        return WorldCupCommand::Done;
                    }

                    // Check if the next jumper in line is human —
                    // if so hand control to the view immediately.
                    if let Some(next_idx) = c.current_jumper() {
                        if !c.field.get(next_idx).is_computer {
                            let next_participant = participant_to_jump(c.field.get(next_idx));
                            let label = Self::phase_label(resources, c.phase);
                            self.note_event_change(store);
                            drop(comp);
                            return WorldCupCommand::HumanJump {
                                participant: next_participant,
                                hill_idx,
                                phase_label: label,
                            };
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn dismiss_display(&self, store: &StoreRef) {
        if let Some(c) = store.competition.borrow_mut().as_mut() {
            c.advance();
        }
    }

    /// Call before rebuilding the jump runner for a human jumper.
    pub(crate) fn note_event_change(&self, store: &StoreRef) {
        let comp = store.competition.borrow();
        if let Some(c) = comp.as_ref() {
            if c.current_event != self.last_event.get() {
                store.first_event.set(true);
                self.last_event.set(c.current_event);
            }
        }
    }

    fn phase_label(resources: &ResourcesRef, phase: CompetitionPhase) -> String {
        match phase {
            CompetitionPhase::Training(n) => format!("{} {}", resources.langbase.lstr(52), n),
            CompetitionPhase::Qualification => resources.langbase.lstr(53).to_string(),
            CompetitionPhase::Round1 => resources.langbase.lstr(54).to_string(),
            CompetitionPhase::Round2 => resources.langbase.lstr(55).to_string(),
            _ => resources.langbase.lstr(51).to_string(),
        }
    }
}

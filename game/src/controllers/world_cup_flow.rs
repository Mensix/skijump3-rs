use crate::competition::types::CompetitionPhase;
use crate::jump::config::JumpParticipant;
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
    /// competition internally -- the caller need only provide the
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

            // Display-list phases
            if matches!(
                c.phase,
                CompetitionPhase::QualificationResults
                    | CompetitionPhase::Round1Results
                    | CompetitionPhase::Round2Results
                    | CompetitionPhase::WorldCupStandings
                    | CompetitionPhase::SeasonComplete
            ) {
                return WorldCupCommand::ShowResults;
            }

            // No jumper scheduled yet -- auto-advance or show results
            if c.current_jumper().is_none() {
                let auto = matches!(
                    c.phase,
                    CompetitionPhase::Training(_)
                        | CompetitionPhase::Setup
                        | CompetitionPhase::EventComplete
                );
                drop(comp);
                if auto {
                    store.competition.borrow_mut().as_mut().unwrap().advance();
                    continue;
                }
                store.competition.borrow_mut().as_mut().unwrap().advance();
                return WorldCupCommand::ShowResults;
            }

            // Human jumper -- hand control to the view
            if c.is_human_current() {
                let jumper_idx = c.current_jumper().unwrap();
                let participant = JumpParticipant::from(c.field.get(jumper_idx));
                let hill_idx = c.hill_order.get(c.current_event).copied().unwrap_or(0);
                let label = Self::phase_label(resources, c.phase);
                drop(comp);
                return WorldCupCommand::HumanJump {
                    participant,
                    hill_idx,
                    phase_label: label,
                };
            }

            // Computer jumper -- simulate via callback, record, continue
            let jumper_idx = c.current_jumper().expect("computer jumper exists");
            let participant = JumpParticipant::from(c.field.get(jumper_idx));
            let hill_idx = c.hill_order.get(c.current_event).copied().unwrap_or(0);
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

            let human_next = c
                .current_jumper()
                .is_some_and(|idx| !c.field.get(idx).is_computer);

            if human_next {
                let jumper_idx = c.current_jumper().unwrap();
                let next_participant = JumpParticipant::from(c.field.get(jumper_idx));
                let label = Self::phase_label(resources, c.phase);
                drop(comp);
                self.note_event_change(store);
                return WorldCupCommand::HumanJump {
                    participant: next_participant,
                    hill_idx,
                    phase_label: label,
                };
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

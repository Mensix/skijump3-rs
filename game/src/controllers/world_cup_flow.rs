use crate::competition::types::CompetitionPhase;
use crate::controllers::jump_scene::JumpScene;
use crate::jump::config::JumpParticipant;

use crate::store::{ResourcesRef, StoreRef};
use std::cell::Cell;

pub enum WorldCupScreenState {
    HumanJump,
    DisplayList,
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

    pub(crate) fn drive(
        &self,
        resources: &ResourcesRef,
        store: &StoreRef,
        scene: &JumpScene,
    ) -> WorldCupScreenState {
        self.record_finished_human_jump(store, scene);

        if self.advance_to_human_or_display(resources, store, scene) {
            WorldCupScreenState::HumanJump
        } else {
            WorldCupScreenState::DisplayList
        }
    }

    pub(crate) fn dismiss_display(&self, store: &StoreRef) {
        if let Some(c) = store.competition.borrow_mut().as_mut() {
            c.advance();
        }
    }

    fn record_finished_human_jump(&self, store: &StoreRef, scene: &JumpScene) {
        let Some(outcome) = scene.outcome() else {
            return;
        };

        let comp = store.competition.borrow();
        let is_human = comp
            .as_ref()
            .is_some_and(super::super::competition::machine::Competition::is_human_current);
        drop(comp);
        if !is_human {
            return;
        }

        let mut comp = store.competition.borrow_mut();
        if let Some(c) = comp.as_mut() {
            c.record_jump(outcome.score, outcome.distance);
        }
    }

    fn advance_to_human_or_display(
        &self,
        resources: &ResourcesRef,
        store: &StoreRef,
        scene: &JumpScene,
    ) -> bool {
        loop {
            let comp = store.competition.borrow();
            let Some(c) = comp.as_ref() else {
                return false;
            };

            if matches!(
                c.phase,
                CompetitionPhase::QualificationResults
                    | CompetitionPhase::Round1Results
                    | CompetitionPhase::Round2Results
                    | CompetitionPhase::WorldCupStandings
                    | CompetitionPhase::SeasonComplete
            ) {
                return false;
            }

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
                return false;
            }

            if c.is_human_current() {
                let jumper_idx = c.current_jumper().unwrap();
                let label = Self::phase_label(resources, c.phase);
                drop(comp);
                if scene.participant_id() != jumper_idx {
                    self.rebuild_runner(store, scene);
                }
                scene.set_phase_label(label);
                return true;
            }

            let jumper_idx = c.current_jumper().expect("computer jumper exists");
            let participant = JumpParticipant::from(c.field.get(jumper_idx));
            let hill_idx = c.hill_order.get(c.current_event).copied().unwrap_or(0);
            drop(comp);

            scene.set_hill(hill_idx);
            scene.set_participant(participant);
            scene.reset_state(15);

            let outcome = scene.simulate_to_completion();

            let mut comp = store.competition.borrow_mut();
            let c = comp.as_mut().unwrap();
            c.record_jump(outcome.score, outcome.distance);
            c.advance();

            if c.is_over() {
                self.rebuild_runner(store, scene);
                return false;
            }

            let human_next = c
                .current_jumper()
                .is_some_and(|idx| !c.field.get(idx).is_computer);

            if human_next {
                drop(comp);
                self.rebuild_runner(store, scene);
                return true;
            }
        }
    }

    fn rebuild_runner(&self, store: &StoreRef, scene: &JumpScene) {
        let event_changed = {
            let comp = store.competition.borrow();
            comp.as_ref()
                .is_some_and(|c| c.current_event != self.last_event.get())
        };
        if event_changed {
            store.first_event.set(true);
            let comp = store.competition.borrow();
            self.last_event
                .set(comp.as_ref().map_or(0, |c| c.current_event));
        }
        let comp = store.competition.borrow();
        let Some(c) = comp.as_ref() else {
            scene.rebuild_for_competition(0, 15, JumpParticipant::trainee(), String::new());
            return;
        };
        let Some(&hill_idx) = c.hill_order.get(c.current_event) else {
            scene.rebuild_for_competition(0, 15, JumpParticipant::trainee(), String::new());
            return;
        };
        let Some(jumper_idx) = c.current_jumper() else {
            scene.rebuild_for_competition(hill_idx, 15, JumpParticipant::trainee(), String::new());
            return;
        };
        let participant = JumpParticipant::from(c.field.get(jumper_idx));
        scene.rebuild_for_competition(hill_idx, 15, participant, String::new());
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

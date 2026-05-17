use crate::competition::types::{CompetitionPhase, Participant};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::{JumpPolicy, JumpRunner};
use crate::store::{ResourcesRef, StoreRef};
use std::cell::{Cell, RefCell};

pub(crate) enum CompetitionRenderState {
    HumanJump,
    DisplayList,
}

pub(crate) struct CompetitionJumpController {
    last_event: Cell<usize>,
}

impl CompetitionJumpController {
    pub(crate) fn new() -> Self {
        Self {
            last_event: Cell::new(0),
        }
    }

    pub(crate) fn initial_runner(resources: &ResourcesRef, store: &StoreRef) -> JumpRunner {
        store.eka.set(true);
        Self::build_runner(resources, store)
    }

    pub(crate) fn drive(
        &self,
        resources: &ResourcesRef,
        store: &StoreRef,
        runner: &RefCell<JumpRunner>,
    ) -> CompetitionRenderState {
        self.record_finished_human_jump(store, runner);

        if self.advance_to_human_or_display(resources, store, runner) {
            CompetitionRenderState::HumanJump
        } else {
            CompetitionRenderState::DisplayList
        }
    }

    pub(crate) fn dismiss_display(&self, store: &StoreRef) {
        if let Some(c) = store.competition.borrow_mut().as_mut() {
            c.advance();
        }
    }

    fn record_finished_human_jump(&self, store: &StoreRef, runner: &RefCell<JumpRunner>) {
        if runner.borrow().outcome().is_none() {
            return;
        }

        let comp = store.competition.borrow();
        let is_human = comp.as_ref().is_some_and(|c| c.is_human_current());
        drop(comp);
        if !is_human {
            return;
        }

        let (points, length) = {
            let runner = runner.borrow();
            let outcome = runner.outcome().expect("human jump should have outcome");
            (outcome.score, outcome.distance)
        };
        let mut comp = store.competition.borrow_mut();
        if let Some(c) = comp.as_mut() {
            c.record_jump(points, length);
        }
    }

    fn advance_to_human_or_display(
        &self,
        resources: &ResourcesRef,
        store: &StoreRef,
        runner: &RefCell<JumpRunner>,
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
                if runner.borrow().participant_id() != jumper_idx {
                    self.rebuild_runner(resources, store, runner);
                }
                runner.borrow_mut().set_phase_label(label);
                return true;
            }

            let jumper_idx = c.current_jumper().expect("computer jumper exists");
            let participant = to_jump_participant(c.field.get(jumper_idx));
            let hill_idx = c.hill_order.get(c.current_event).copied().unwrap_or(0);
            let record_distance = store
                .records
                .borrow()
                .hill_record(hill_idx)
                .map_or(0, |r| r.len as i32);
            drop(comp);

            {
                let mut runner = runner.borrow_mut();
                runner.set_hill(hill_idx, resources);
                runner.set_participant(participant);
                runner.reset_state(15, record_distance);
            }

            let outcome = {
                let mut runner = runner.borrow_mut();
                let mut rng = store.rng.borrow_mut();
                let mut wind = store.wind.borrow_mut();
                runner.simulate_to_completion(&mut rng, &mut wind)
            };

            let mut comp = store.competition.borrow_mut();
            let c = comp.as_mut().unwrap();
            c.record_jump(outcome.score, outcome.distance);
            c.advance();

            if c.is_over() {
                self.rebuild_runner(resources, store, runner);
                return false;
            }

            let human_next = c
                .current_jumper()
                .is_some_and(|idx| !c.field.get(idx).is_computer);

            if human_next {
                drop(comp);
                self.rebuild_runner(resources, store, runner);
                return true;
            }
        }
    }

    fn rebuild_runner(
        &self,
        resources: &ResourcesRef,
        store: &StoreRef,
        runner: &RefCell<JumpRunner>,
    ) {
        let event_changed = {
            let comp = store.competition.borrow();
            comp.as_ref()
                .is_some_and(|c| c.current_event != self.last_event.get())
        };
        if event_changed {
            store.eka.set(true);
            let comp = store.competition.borrow();
            self.last_event
                .set(comp.as_ref().map_or(0, |c| c.current_event));
        }
        *runner.borrow_mut() = Self::build_runner(resources, store);
    }

    fn build_runner(resources: &ResourcesRef, store: &StoreRef) -> JumpRunner {
        let comp = store.competition.borrow();
        let Some(c) = comp.as_ref() else {
            return JumpRunner::new_with_env(
                0,
                15,
                JumpParticipant::trainee(),
                JumpPolicy::competition(),
                resources,
                store,
            );
        };
        let Some(&hill_idx) = c.hill_order.get(c.current_event) else {
            return JumpRunner::new_with_env(
                0,
                15,
                JumpParticipant::trainee(),
                JumpPolicy::competition(),
                resources,
                store,
            );
        };
        let Some(jumper_idx) = c.current_jumper() else {
            return JumpRunner::new_with_env(
                hill_idx,
                15,
                JumpParticipant::trainee(),
                JumpPolicy::competition(),
                resources,
                store,
            );
        };
        let participant = to_jump_participant(c.field.get(jumper_idx));
        JumpRunner::new_with_env(
            hill_idx,
            15,
            participant,
            JumpPolicy::competition(),
            resources,
            store,
        )
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

fn to_jump_participant(p: &Participant) -> JumpParticipant {
    JumpParticipant {
        id: p.id,
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

use crate::competition::types::{CompetitionPhase, Participant};
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::{JumpPolicy, JumpRunner};
use crate::palette_consts::FONT_DEFAULT;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::{Cell, RefCell};

pub(crate) struct CompetitionJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    runner: RefCell<JumpRunner>,
    last_event: Cell<usize>,
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

impl CompetitionJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        store.eka.set(true);
        let runner = Self::build_runner(&resources, &store);
        Self {
            resources,
            store,
            runner: RefCell::new(runner),
            last_event: Cell::new(0),
        }
    }

    fn build_runner(resources: &ResourcesRef, store: &StoreRef) -> JumpRunner {
        let comp = store.competition.borrow();
        let Some(c) = comp.as_ref() else {
            return JumpRunner::new_with_env(
                0, 15, JumpParticipant::trainee(), JumpPolicy::competition(), resources, store,
            );
        };
        let Some(&hill_idx) = c.hill_order.get(c.current_event) else {
            return JumpRunner::new_with_env(
                0, 15, JumpParticipant::trainee(), JumpPolicy::competition(), resources, store,
            );
        };
        let Some(jumper_idx) = c.current_jumper() else {
            return JumpRunner::new_with_env(
                hill_idx, 15, JumpParticipant::trainee(), JumpPolicy::competition(), resources, store,
            );
        };
        let participant = to_jump_participant(c.field.get(jumper_idx));
        JumpRunner::new_with_env(hill_idx, 15, participant, JumpPolicy::competition(), resources, store)
    }

    fn rebuild_runner(&self) {
        let event_changed = {
            let comp = self.store.competition.borrow();
            comp.as_ref().map_or(false, |c| c.current_event != self.last_event.get())
        };
        if event_changed {
            self.store.eka.set(true);
            let comp = self.store.competition.borrow();
            self.last_event.set(comp.as_ref().map_or(0, |c| c.current_event));
        }
        *self.runner.borrow_mut() = Self::build_runner(&self.resources, &self.store);
    }

    /// Drive the machine past all non-jump phases and computer jumpers.
    /// When a human jumper is reached the runner is set up and `true` returned.
    /// Returns `false` if the machine is in a display phase (Results/EventComplete/SeasonComplete).
    fn advance_machine(&self) -> bool {
        loop {
            let comp = self.store.competition.borrow();
            let Some(c) = comp.as_ref() else {
                return false;
            };

            // Display phases — auto-advance for MVP; stop at SeasonComplete
            if c.phase == CompetitionPhase::SeasonComplete {
                return false;
            }
            if matches!(c.phase, CompetitionPhase::Results | CompetitionPhase::EventComplete) {
                drop(comp);
                self.store.competition.borrow_mut().as_mut().unwrap().advance();
                continue;
            }

            // No current jumper — advance to next phase
            if c.current_jumper().is_none() {
                drop(comp);
                self.store.competition.borrow_mut().as_mut().unwrap().advance();
                continue;
            }

            // Human jumper — set up runner with correct phase label
            if c.is_human_current() {
                let label = match c.phase {
                    CompetitionPhase::Training(n) => {
                        format!("{} {}", self.resources.langbase.lstr(52), n)
                    }
                    CompetitionPhase::Qualification => self.resources.langbase.lstr(53).to_string(),
                    CompetitionPhase::Round1 => self.resources.langbase.lstr(54).to_string(),
                    CompetitionPhase::Round2 => self.resources.langbase.lstr(55).to_string(),
                    _ => self.resources.langbase.lstr(51).to_string(),
                };
                drop(comp);
                self.rebuild_runner();
                self.runner.borrow_mut().set_phase_label(label);
                return true;
            }

            // Computer jumper — fast-forward silently
            let jumper_idx = c.current_jumper().expect("computer jumper exists");
            let participant = to_jump_participant(c.field.get(jumper_idx));
            let hill_idx = c.hill_order.get(c.current_event).copied().unwrap_or(0);
            let record_distance = self.store.records
                .borrow()
                .hill_record(hill_idx)
                .map_or(0, |r| r.len as i32);
            drop(comp);

            {
                let mut runner = self.runner.borrow_mut();
                runner.set_participant(participant);
                runner.reset_state(15, record_distance);
            }

            let outcome = {
                let mut runner = self.runner.borrow_mut();
                let mut rng = self.store.rng.borrow_mut();
                let mut wind = self.store.wind.borrow_mut();
                runner.simulate_to_completion(&mut rng, &mut wind)
            };

            let mut comp = self.store.competition.borrow_mut();
            let c = comp.as_mut().unwrap();
            c.record_jump(outcome.score, outcome.distance);
            c.advance();

            if c.is_over() {
                self.rebuild_runner();
                return false;
            }

            let human_next = c
                .current_jumper()
                .is_some_and(|idx| !c.field.get(idx).is_computer);

            if human_next {
                drop(comp);
                self.rebuild_runner();
                return true;
            }
        }
    }
}

impl View<RouteTarget> for CompetitionJumpView {
    fn elements(&self) -> Vec<Element> {
        // Record completed human jump before advancing machine
        if self.runner.borrow().outcome().is_some() {
            let (points, length) = {
                let runner = self.runner.borrow();
                let o = runner.outcome().unwrap();
                (o.score, o.distance)
            };
            let mut comp = self.store.competition.borrow_mut();
            let c = comp.as_mut().expect("competition set");
            c.record_jump(points, length);
            c.advance();
        }

        // Drive machine through computers and phase transitions
        if !self.advance_machine() {
            let comp = self.store.competition.borrow();
            let msg = match comp.as_ref().map(|c| &c.phase) {
                Some(CompetitionPhase::SeasonComplete) => "Season Complete!",
                _ => "Processing...",
            };
            return vec![
                Element::fillbox(0, 0, 320, 200, 0),
                Element::text_color(msg, 20, 80, FONT_DEFAULT),
                Element::text_color("PRESS ESC", 20, 95, FONT_DEFAULT),
            ];
        }

        // Render human jump
        self.runner.borrow_mut().elements(&self.resources, &self.store)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::Back),
            Event::Keyboard(_) => None,
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if let Ok(mut runner) = self.runner.try_borrow_mut() {
            let wind = self.store.wind.borrow().value;
            runner.render_snow(framebuffer, wind);
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if let Ok(runner) = self.runner.try_borrow() {
            runner.apply_palette(palette);
        }
    }
}

use crate::competition::types::Participant;
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::{JumpPolicy, JumpRunner};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::RefCell;

pub(crate) struct CompetitionJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    runner: RefCell<JumpRunner>,
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
}

impl View<RouteTarget> for CompetitionJumpView {
    fn elements(&self) -> Vec<Element> {
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

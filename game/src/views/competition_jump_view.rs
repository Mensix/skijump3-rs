use crate::competition::types::Participant;
use crate::jump::config::JumpParticipant;
use crate::jump::policy::JumperControl;
use crate::jump::{JumpConfig, JumpPolicy, JumpRunner};
use crate::palette_consts::FONT_DEFAULT;
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use engine::palette::Palette;
use engine::ui::{Element, Event, Key, View};
use std::cell::RefCell;

pub(crate) struct CompetitionJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    runner: Option<RefCell<JumpRunner>>,
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
        let runner = Self::build_runner(&resources, &store).map(RefCell::new);
        Self {
            resources,
            store,
            runner,
        }
    }

    fn build_runner(resources: &ResourcesRef, store: &StoreRef) -> Option<JumpRunner> {
        let comp = store.competition.borrow();
        let c = comp.as_ref()?;
        let hill_idx = c.hill_order.get(c.current_event).copied()?;
        let jumper_idx = c.current_jumper()?;
        let participant = to_jump_participant(c.field.get(jumper_idx));

        let hill = resources.hills.hill(hill_idx).cloned();
        let terrain = resources.hill_terrain(hill_idx).map(|t| (*t).clone());
        let mut snow = SnowSystem::new();
        if terrain.is_ok() && hill.is_some() {
            let mut rng = store.rng.borrow_mut();
            let mut wind = store.wind.borrow_mut();
            wind.initialize(&mut rng, store.wind_place.get());
            snow.set_count(0, &mut rng);
            wind.sample(&mut rng);
        }
        let record_distance = store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map_or(0, |r| r.len as i32);

        Some(JumpRunner::new(JumpConfig {
            hill_idx,
            hill,
            terrain,
            start_gate: 15,
            snow,
            participant,
            policy: JumpPolicy::competition(),
            record_distance,
        }))
    }
}

impl View<RouteTarget> for CompetitionJumpView {
    fn elements(&self) -> Vec<Element> {
        if let Some(runner) = &self.runner {
            return runner.borrow_mut().elements(&self.resources, &self.store);
        }

        vec![
            Element::fillbox(0, 0, 320, 200, 0),
            Element::text_color("World Cup state not available", 20, 80, FONT_DEFAULT),
            Element::text_color("PRESS ESC", 20, 95, FONT_DEFAULT),
        ]
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::Back),
            Event::Keyboard(_) => None,
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if let Some(runner) = &self.runner {
            if let Ok(mut runner) = runner.try_borrow_mut() {
                let wind = self.store.wind.borrow().value;
                runner.render_snow(framebuffer, wind);
            }
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if let Some(runner) = &self.runner {
            if let Ok(runner) = runner.try_borrow() {
                runner.apply_palette(palette);
            }
        }
    }
}

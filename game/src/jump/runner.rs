use crate::jump::config::JumpConfig;
use crate::jump::presentation;
use crate::jump::replay::ReplayTrace;
use crate::jump::types::JumpOutcome;
use crate::jump::{
    ComputerInputProvider, JumpInputProvider, JumpPresentationContext, JumpSession, JumperControl,
    WindGaugePosition,
};
use crate::palette_consts::FONT_DEFAULT;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::palette::Palette;
use engine::ui::Element;

#[derive(Debug)]
pub(crate) struct JumpRunner {
    session: JumpSession,
    config: JumpConfig,
    computer_input: Option<ComputerInputProvider>,
}

impl JumpRunner {
    pub(crate) fn new(config: JumpConfig) -> Self {
        let session = JumpSession::new(config.clone());
        let computer_input = (config.participant.control == JumperControl::Computer)
            .then(ComputerInputProvider::new);
        Self {
            session,
            config,
            computer_input,
        }
    }

    pub(crate) fn hill_idx(&self) -> usize {
        self.config.hill_idx
    }

    pub(crate) fn session_mut(&mut self) -> &mut JumpSession {
        &mut self.session
    }

    pub(crate) fn outcome(&self) -> Option<JumpOutcome> {
        self.session.outcome()
    }

    pub(crate) fn replay_trace(&self) -> Option<ReplayTrace> {
        self.session.replay_trace()
    }

    pub(crate) fn reset_state(&mut self, start_gate: i32, record_distance: i32) {
        self.config.start_gate = start_gate;
        self.config.record_distance = record_distance;
        if let Some(hill) = &self.config.hill {
            self.session.reset_state(hill, start_gate, record_distance);
        }
        self.computer_input = (self.config.participant.control == JumperControl::Computer)
            .then(ComputerInputProvider::new);
    }

    pub(crate) fn elements(&mut self, resources: &ResourcesRef, store: &StoreRef) -> Vec<Element> {
        let Err(err) = self.session.terrain() else {
            if self.session.state().is_some() {
                return self.elements_for_loaded_session(resources, store);
            }
            return unavailable_elements("jump state not available");
        };

        unavailable_elements(err)
    }

    fn elements_for_loaded_session(
        &mut self,
        resources: &ResourcesRef,
        store: &StoreRef,
    ) -> Vec<Element> {
        if self.session.phase().is_none() {
            return unavailable_elements("jump state not available");
        }

        let mut rng = store.rng.borrow_mut();
        if let (Some(snapshot), Some(input)) =
            (self.session.snapshot(), self.computer_input.as_mut())
        {
            for jump_input in input.inputs(&snapshot, &mut rng) {
                self.session.handle_input(jump_input);
            }
        }
        let mut wind_store = store.wind.borrow_mut();
        let wind = self.session.tick_with_wind(&mut rng, &mut wind_store);
        drop(wind_store);
        drop(rng);

        let hill_name_k = resources
            .hills
            .hill(self.config.hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let wind_pos = store.wind.borrow().position();
        let records = store.records.borrow();
        let frame = self
            .session
            .render_frame(wind, WIDTH, HEIGHT)
            .expect("loaded jump render frame");
        let ctx = JumpPresentationContext {
            font: &resources.font,
            langbase: &resources.langbase,
            jumper_name: self.config.participant.display_name(),
            hill_name_k: &hill_name_k,
            hill_record: records.hill_record(self.config.hill_idx),
            wind_position: WindGaugePosition {
                x: wind_pos.x,
                y: wind_pos.y,
            },
        };
        presentation::elements(&frame, &ctx)
    }

    pub(crate) fn render_snow(&mut self, framebuffer: &mut [u8], wind: i32) {
        let draw = self.session.draws_snow();
        self.session.render_snow(framebuffer, wind, draw);
    }

    pub(crate) fn apply_palette(&self, palette: &mut Palette) {
        if let Ok(terrain) = self.session.terrain() {
            terrain.apply_hill_palette(palette);
        }
        palette.set(253, [10, 54, 10]);
        palette.set(254, [0, 47, 0]);
    }
}

fn unavailable_elements(message: &str) -> Vec<Element> {
    vec![
        Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0),
        Element::text_color(message, 20, 80, FONT_DEFAULT),
        Element::text_color("PRESS ESC", 20, 95, FONT_DEFAULT),
    ]
}

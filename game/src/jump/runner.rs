use crate::data::records::{HillCatalog, RecordStore};
use crate::gfx::palette::FONT_DEFAULT;
use crate::jump::config::JumpConfig;
use crate::jump::presentation;
use crate::jump::replay::ReplayTrace;
use crate::jump::snow::SnowSystem;
use crate::jump::types::{FlightWind, JumpOutcome, JumpPhase};
use crate::jump::wind::Wind;
use crate::jump::wind::WindPosition;
use crate::jump::{ComputerInputProvider, JumpPresentationContext, JumpSession, JumperControl};
use crate::parsers::langbase::LangBase;
use crate::rng::Random;
use engine::consts::{HEIGHT, WIDTH};
use engine::palette::Palette;
use engine::ui::{Element, Font};

pub(crate) struct JumpRunnerRenderEnv<'a> {
    pub(crate) font: &'a Font,
    pub(crate) langbase: &'a LangBase,
    pub(crate) hills: &'a HillCatalog,
    pub(crate) records: &'a RecordStore,
    pub(crate) wind: &'a Wind,
}

#[derive(Debug)]
pub struct JumpRunner {
    session: JumpSession,
    config: JumpConfig,
    snow: SnowSystem,
    prev_camera: (i32, i32),
    computer_input: Option<ComputerInputProvider>,
    computer_pre_ai_wind_done: bool,
    last_wind: FlightWind,
}

impl JumpRunner {
    pub(crate) fn new(config: JumpConfig, snow: SnowSystem) -> Self {
        let computer_input = (config.participant.control == JumperControl::Computer)
            .then(|| ComputerInputProvider::new(config.participant.ai_id));
        let session = JumpSession::new(config.clone());
        let prev_camera = session.camera().unwrap_or((0, 0));
        Self {
            session,
            config,
            snow,
            prev_camera,
            computer_input,
            computer_pre_ai_wind_done: false,
            last_wind: FlightWind::default(),
        }
    }

    pub(crate) const fn hill_idx(&self) -> usize {
        self.config.hill_idx
    }

    pub(crate) const fn participant_id(&self) -> usize {
        self.config.participant.id
    }

    pub(crate) const fn session_mut(&mut self) -> &mut JumpSession {
        &mut self.session
    }

    pub(crate) fn phase(&self) -> Option<JumpPhase> {
        self.session.phase()
    }

    pub(crate) fn outcome(&self) -> Option<JumpOutcome> {
        self.session.outcome()
    }

    pub(crate) fn frame_counter(&self) -> i32 {
        self.session.state().map_or(0, |s| s.frame)
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
            .then(|| ComputerInputProvider::new(self.config.participant.ai_id));
        self.computer_pre_ai_wind_done = false;
    }

    /// Fast-forward computer jump simulation to completion without rendering.
    /// Drives computer AI inputs and ticks physics until outcome is available.
    pub(crate) fn simulate_to_completion(
        &mut self,
        rng: &mut Random,
        wind: &mut Wind,
    ) -> JumpOutcome {
        // Pascal does one Tuuli.Hae before switching to non-draw mode
        wind.advance_without_sampling(rng);
        self.computer_pre_ai_wind_done = true;
        if let Some(input) = &mut self.computer_input {
            input.prepare_for_jump(rng);
        }
        self.session.prepare_silent_computer_jump();
        for _ in 0..100 {
            wind.advance_without_sampling(rng);
        }

        loop {
            if let Some(outcome) = self.session.outcome() {
                return outcome;
            }
            if let Some(snapshot) = self.session.snapshot() {
                if let Some(input) = &mut self.computer_input {
                    for inp in input.inputs(&snapshot, rng) {
                        self.session.handle_input(inp);
                    }
                }
            }
            self.session.tick_with_wind(rng, wind);
        }
    }

    pub(crate) fn set_phase_label(&mut self, label: String) {
        self.config.phase_label = label;
    }

    /// Advance physics, AI, and wind by one frame. Call once per frame
    /// before `elements()` so the rendering stays pure.
    pub(crate) fn update(&mut self, rng: &mut Random, wind: &mut Wind) {
        if self.computer_input.is_some() && !self.computer_pre_ai_wind_done {
            // Pascal samples wind once before computer skill/reflex are initialized.
            wind.advance_without_sampling(rng);
            self.computer_pre_ai_wind_done = true;
        }

        if let (Some(snapshot), Some(input)) =
            (self.session.snapshot(), self.computer_input.as_mut())
        {
            for jump_input in input.inputs(&snapshot, rng) {
                self.session.handle_input(jump_input);
            }
        }
        self.last_wind = self.session.tick_with_wind(rng, wind);
    }

    pub(crate) fn elements(&mut self, env: JumpRunnerRenderEnv<'_>) -> Vec<Element> {
        match self.session.terrain() {
            Err(err) => unavailable_elements(err),
            Ok(_) if self.session.state().is_some() => self.elements_for_loaded_session(env),
            _ => unavailable_elements("jump state not available"),
        }
    }

    fn elements_for_loaded_session(&mut self, env: JumpRunnerRenderEnv<'_>) -> Vec<Element> {
        if self.session.phase().is_none() {
            return unavailable_elements("jump state not available");
        }

        let hill_name_k = env
            .hills
            .hill(self.config.hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let wind_pos = env.wind.position();
        let frame = self
            .session
            .render_frame(self.last_wind, WIDTH, HEIGHT)
            .expect("loaded jump render frame");
        let ctx = JumpPresentationContext {
            font: env.font,
            langbase: env.langbase,
            jumper_name: self.config.participant.display_name(),
            hill_name_k: &hill_name_k,
            hill_record: env.records.hill_record(self.config.hill_idx),
            wind_position: WindPosition {
                x: wind_pos.x,
                y: wind_pos.y,
            },
            phase_label: &self.config.phase_label,
            allow_gate_adjust: self.config.policy.allow_start_gate_adjust,
        };
        presentation::elements(&frame, &ctx)
    }

    pub(crate) fn render_snow(&mut self, framebuffer: &mut [u8], wind: i32) {
        let draw = self.session.draws_snow();
        if let Some(camera) = self.session.camera() {
            let delta_x = self.prev_camera.0 - camera.0;
            let delta_y = self.prev_camera.1 - camera.1;
            self.prev_camera = camera;
            self.snow.update(framebuffer, delta_x, delta_y, wind, draw);
        }
    }

    pub(crate) fn apply_palette(&self, palette: &mut Palette) {
        if let Ok(terrain) = self.session.terrain() {
            terrain.apply_hill_palette(palette);
        }
        let is_dq = self.session.phase() == Some(JumpPhase::Disqualified);
        if is_dq {
            // Pascal MuutaLogo(4) — red start light
            palette.set(253, [54, 10, 10]);
            palette.set(254, [47, 0, 0]);
        } else {
            // Pascal MuutaLogo(6) — green start light
            palette.set(253, [10, 54, 10]);
            palette.set(254, [0, 47, 0]);
        }
    }
}

fn unavailable_elements(message: &str) -> Vec<Element> {
    vec![
        Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0),
        Element::text(message, 20, 80, FONT_DEFAULT, false),
        Element::text("PRESS ESC", 20, 95, FONT_DEFAULT, false),
    ]
}

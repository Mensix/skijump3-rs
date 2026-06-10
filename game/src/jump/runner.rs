use crate::data::hill::HillCatalog;
use crate::data::records::RecordStore;
use crate::gfx::palette::{BLACK, FONT_DEFAULT};
use crate::jump::config::JumpConfig;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::presentation;
use crate::jump::replay::ReplayTrace;
use crate::jump::snow::SnowSystem;
use crate::jump::types::{FlightWind, JumpOutcome, JumpPhase};
use crate::jump::wind::Wind;
use crate::jump::wind::WindPosition;
use crate::jump::{ComputerInputProvider, JumpPresentationContext, JumpSession, JumperControl};
use crate::rng::Random;
use crate::text::lang::LangBase;
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Font};
use std::cell::Cell;

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
    pub(crate) suppress_info_panel: Cell<bool>,
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
            suppress_info_panel: Cell::new(false),
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

    pub(crate) fn set_phase_label(&mut self, label: String) {
        self.config.phase_label = label;
    }

    pub(crate) fn set_team_name(&mut self, name: String) {
        self.config.team_name = name;
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
            Err(err) => unavailable_elements(&err.to_string()),
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
        let mut frame = self
            .session
            .render_frame(self.last_wind, WIDTH, HEIGHT)
            .expect("loaded jump render frame");
        self.apply_snow_to_viewport(&mut frame, env.wind.value);
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
            team_name: &self.config.team_name,
            allow_gate_adjust: self.config.policy.allow_start_gate_adjust,
            suppress_info_panel: self.suppress_info_panel.get(),
            suit_color: self.config.participant.suit_color as usize,
            ski_color: self.config.participant.ski_color as usize,
        };
        presentation::elements(&frame, &ctx)
    }

    fn apply_snow_to_viewport(&mut self, frame: &mut JumpRenderFrame, wind: i32) {
        let draw = self.session.draws_snow();
        if let Some(camera) = self.session.camera() {
            let delta_x = self.prev_camera.0 - camera.0;
            let delta_y = self.prev_camera.1 - camera.1;
            self.prev_camera = camera;
            let mut viewport = frame.viewport.to_vec();
            let mask = &frame.snow_mask;
            self.snow
                .update(&mut viewport, mask, delta_x, delta_y, wind, draw);
            frame.viewport = viewport.into();
        }
    }
}

fn unavailable_elements(message: &str) -> Vec<Element> {
    vec![
        Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, BLACK),
        Element::text(message, 20, 80, FONT_DEFAULT, false),
        Element::text("PRESS ESC", 20, 95, FONT_DEFAULT, false),
    ]
}

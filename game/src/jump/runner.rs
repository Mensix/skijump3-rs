use crate::jump::config::JumpConfig;
use crate::jump::policy::JumpPolicy;
use crate::jump::presentation;
use crate::jump::replay::ReplayTrace;
use crate::jump::types::JumpOutcome;
use crate::jump::JumpParticipant;
use crate::jump::{
    ComputerInputProvider, JumpInputProvider, JumpPresentationContext, JumpSession, JumperControl,
    WindGaugePosition,
};
use crate::palette_consts::FONT_DEFAULT;
use crate::pascal_random::PascalRandom;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use crate::wind::PascalWind;
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
    /// Create a runner with shared environment initialization:
    /// hill/terrain loading, wind init, Pascal snow init (eka gate), record distance.
    pub(crate) fn new_with_env(
        hill_idx: usize,
        start_gate: i32,
        participant: JumpParticipant,
        policy: JumpPolicy,
        resources: &ResourcesRef,
        store: &StoreRef,
    ) -> Self {
        let hill = resources.hills.hill(hill_idx).cloned();
        let terrain = resources.hill_terrain(hill_idx).map(|t| (*t).clone());
        let mut snow = SnowSystem::new();

        if terrain.is_ok() && hill.is_some() {
            let mut rng = store.rng.borrow_mut();
            let mut wind = store.wind.borrow_mut();
            wind.initialize(&mut rng, store.wind_place.get());

            if store.eka.get() {
                let lmaara = crate::snow::calculate_lmaara(&mut rng);
                snow.set_count(lmaara, &mut rng);
                wind.sample(&mut rng);
                store.eka.set(false);
            }
        }

        let record_distance = store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map_or(0, |r| r.len as i32);

        Self::new(JumpConfig {
            hill_idx,
            hill,
            terrain,
            start_gate,
            snow,
            participant,
            policy,
            record_distance,
            phase_label: String::new(),
        })
    }

    pub(crate) fn new(config: JumpConfig) -> Self {
        let session = JumpSession::new(config.clone());
        let computer_input = (config.participant.control == JumperControl::Computer)
            .then(|| ComputerInputProvider::new(config.participant.id));
        Self {
            session,
            config,
            computer_input,
        }
    }

    pub(crate) fn hill_idx(&self) -> usize {
        self.config.hill_idx
    }

    pub(crate) fn participant_id(&self) -> usize {
        self.config.participant.id
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
            .then(|| ComputerInputProvider::new(self.config.participant.id));
    }

    /// Fast-forward computer jump simulation to completion without rendering.
    /// Drives computer AI inputs and ticks physics until outcome is available.
    pub(crate) fn simulate_to_completion(
        &mut self,
        rng: &mut PascalRandom,
        wind: &mut PascalWind,
    ) -> JumpOutcome {
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

    /// Set the phase label shown in the info panel (e.g. "Qualification", "Round 1").
    /// Training mode shows the default langbase string; competition sets it explicitly.
    pub(crate) fn set_phase_label(&mut self, label: String) {
        self.config.phase_label = label;
    }

    /// Swap participant without recreating snow/wind state.
    /// Used by competition mode to reuse one runner across jumpers.
    #[allow(dead_code)]
    pub(crate) fn set_participant(&mut self, participant: JumpParticipant) {
        self.config.participant = participant;
        self.computer_input = (self.config.participant.control == JumperControl::Computer)
            .then(|| ComputerInputProvider::new(self.config.participant.id));
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
            phase_label: &self.config.phase_label,
            allow_gate_adjust: self.config.policy.allow_start_gate_adjust,
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

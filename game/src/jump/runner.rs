use crate::data::hill::HillCatalog;
use crate::data::hill_profile::HillTerrain;
use crate::data::records::RecordStore;
use crate::error::AssetError;
use crate::gfx::theme::{BLACK, FONT_BODY, FONT_GRAY};
use crate::jump::config::JumpConfig;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::math;
use crate::jump::presentation;
use crate::jump::replay::{ReplayMeta, ReplayRecorder, ReplayTrace};
use crate::jump::snow::SnowSystem;
use crate::jump::state::JumpState;
use crate::jump::types::{FlightWind, JumpInput, JumpOutcome, JumpPhase, JumpSnapshot};
use crate::jump::wind::Wind;
use crate::jump::wind::WindPosition;
use crate::jump::{ComputerInputProvider, JumpPresentationContext, JumperControl};
use crate::rng::Random;
use crate::text::lang::LangBase;
use engine::consts::{HEIGHT, WIDTH};
use engine::oxide::Font;
use engine::oxide::PaintCx;
use std::cell::Cell;

pub(crate) struct JumpRunnerRenderEnv<'a> {
    pub(crate) font: &'a Font,
    pub(crate) langbase: &'a LangBase,
    pub(crate) hills: &'a HillCatalog,
    pub(crate) records: &'a RecordStore,
    pub(crate) wind: &'a Wind,
}

fn find_hill_record_marker(
    terrain: &HillTerrain,
    pk: f64,
    record_distance: f64,
) -> Option<(i32, i32)> {
    if record_distance <= 0.0 {
        return None;
    }
    let tip_x = terrain.tip_x;
    let tip_y = terrain.height_at(tip_x);
    for x in tip_x..1024 {
        let dx = f64::from(x - tip_x);
        let dy = f64::from(terrain.height_at(x) - tip_y);
        let hp = math::round(dx.hypot(dy) * pk * 0.5) * 5;
        if f64::from(hp) / 10.0 >= record_distance {
            let ground_y = terrain.height_at(x);
            return Some((x, ground_y - 9));
        }
    }
    None
}

#[derive(Debug)]
pub struct JumpRunner {
    config: JumpConfig,
    state: Option<JumpState>,
    replay_prev_pos: Option<(i32, i32)>,
    replay: ReplayRecorder,
    last_phase: Option<JumpPhase>,
    record_marker: Option<(i32, i32)>,
    snow: SnowSystem,
    prev_camera: (i32, i32),
    computer_input: Option<ComputerInputProvider>,
    computer_pre_ai_wind_done: bool,
    last_wind: FlightWind,
    hr_shake_position: Cell<Option<(i32, i32)>>,
    pub(crate) suppress_info_panel: Cell<bool>,
    pub(crate) has_bib: Cell<bool>,
    keymap_shown: Cell<bool>,
}

impl JumpRunner {
    pub(crate) fn new(config: JumpConfig, snow: SnowSystem) -> Self {
        let computer_input = (config.participant.control == JumperControl::Computer)
            .then(|| ComputerInputProvider::new(config.participant.ai_id));
        let state = Self::new_state(&config);
        let replay_prev_pos = state.as_ref().map(|state| (state.x, state.y));
        let last_phase = state.as_ref().map(|state| state.phase);
        let mut replay = ReplayRecorder::default();
        if let Some(state) = &state {
            replay.start(Self::replay_meta(state, &config));
        }
        let record_marker = Self::record_marker(&config, state.as_ref());
        let prev_camera = state.as_ref().map_or((0, 0), |state| (state.sx, state.sy));
        Self {
            config,
            state,
            replay_prev_pos,
            replay,
            last_phase,
            record_marker,
            snow,
            prev_camera,
            computer_input,
            computer_pre_ai_wind_done: false,
            last_wind: FlightWind::default(),
            hr_shake_position: Cell::new(None),
            suppress_info_panel: Cell::new(false),
            has_bib: Cell::new(false),
            keymap_shown: Cell::new(false),
        }
    }

    fn new_state(config: &JumpConfig) -> Option<JumpState> {
        match (&config.terrain, config.hill.as_ref()) {
            (Ok(terrain), Some(hill)) => Some(JumpState::new(
                terrain,
                hill.vx_final as f64,
                hill.pk(),
                hill.kr as i32,
                hill.pl_save(),
                config.start_gate,
            )),
            _ => None,
        }
    }

    fn replay_meta(state: &JumpState, config: &JumpConfig) -> ReplayMeta {
        ReplayMeta {
            start_x: state.x,
            start_y: state.y,
            hill_idx: config.hill_idx,
            snow_count: config.snow_count,
            distance: 0,
            flight_start: 0,
            flight_stop: 0,
            hill_record_marker: None,
            hill_filename: "HILLBASE".to_string(),
            hill_profile: config
                .hill
                .as_ref()
                .and_then(|hill| i32::try_from(hill.profile_checksum).ok())
                .unwrap_or_default(),
            suit_color: config.participant.suit_color,
            ski_color: config.participant.ski_color,
            saved_at: String::new(),
            has_bib: false,
            author: String::new(),
            name: config.participant.display_name().to_string(),
            start_gate_or_competition: 100 - config.start_gate,
            frame_count: 0,
            checksum: 0,
            valid_checksum: true,
            intro: false,
        }
    }

    fn record_marker(config: &JumpConfig, state: Option<&JumpState>) -> Option<(i32, i32)> {
        let (Ok(terrain), Some(hill), Some(_)) = (&config.terrain, config.hill.as_ref(), state)
        else {
            return None;
        };
        find_hill_record_marker(terrain, hill.pk(), config.record_distance)
    }

    pub(crate) const fn hill_idx(&self) -> usize {
        self.config.hill_idx
    }

    pub(crate) const fn participant_id(&self) -> usize {
        self.config.participant.id
    }

    pub(crate) fn handle_input(&mut self, input: JumpInput) {
        if let Some(state) = &mut self.state {
            state.handle_input(input);
        }
    }

    pub(crate) fn handle_start_gate_adjust(&mut self, delta: i32) -> Option<i32> {
        if !self.config.policy.allow_start_gate_adjust {
            return self.start_gate();
        }
        self.handle_input(JumpInput::AdjustGate(delta));
        self.start_gate()
    }

    pub(crate) fn start_gate(&self) -> Option<i32> {
        self.state.as_ref().map(|state| state.start_gate)
    }

    pub(crate) const fn policy(&self) -> crate::jump::policy::JumpPolicy {
        self.config.policy
    }

    pub(crate) fn phase(&self) -> Option<JumpPhase> {
        self.state.as_ref().map(|state| state.phase)
    }

    pub(crate) fn outcome(&self) -> Option<JumpOutcome> {
        self.state.as_ref()?.outcome()
    }

    pub(crate) fn state(&self) -> Option<&JumpState> {
        self.state.as_ref()
    }

    pub(crate) fn frame_counter(&self) -> i32 {
        self.state.as_ref().map_or(0, |s| s.frame)
    }

    pub(crate) fn replay_trace(&self) -> Option<ReplayTrace> {
        self.replay.finish()
    }

    pub(crate) fn clone_snow(&self) -> SnowSystem {
        self.snow.clone()
    }

    pub(crate) fn reset_state(&mut self, start_gate: i32, record_distance: f64) {
        self.config.start_gate = start_gate;
        self.config.record_distance = record_distance;
        if self.config.hill.is_some() {
            self.state = Self::new_state(&self.config);
            self.record_marker = Self::record_marker(&self.config, self.state.as_ref());
            self.replay_prev_pos = self.state.as_ref().map(|state| (state.x, state.y));
            self.last_phase = self.state.as_ref().map(|state| state.phase);
            if let Some(state) = &self.state {
                self.replay.start(Self::replay_meta(state, &self.config));
            }
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

    pub(crate) fn set_has_bib(&self, val: bool) {
        self.has_bib.set(val);
    }

    /// Advance physics, AI, and wind by one frame. Call once per frame
    /// before `render()` so the rendering stays pure.
    pub(crate) fn update(&mut self, rng: &mut Random, wind: &mut Wind) {
        if self.computer_input.is_some() && !self.computer_pre_ai_wind_done {
            // Pascal samples wind once before computer skill/reflex are initialized.
            wind.advance_without_sampling(rng);
            self.computer_pre_ai_wind_done = true;
        }

        if let (Some(snapshot), Some(input)) = (self.snapshot(), self.computer_input.as_mut()) {
            for jump_input in input.inputs(&snapshot, rng) {
                self.handle_input(jump_input);
            }
        }
        self.last_wind = self.tick_with_wind(rng, wind);
        self.hr_shake_position.set(None);
        if self.phase() == Some(JumpPhase::Landing)
            && self.config.record_distance > 0.0
            && self
                .state
                .as_ref()
                .is_some_and(|state| state.distance > self.config.record_distance)
            && rng.random_i32(2) == 0
        {
            self.hr_shake_position
                .set(Some((308 - 1 + rng.random_i32(3), 32 + rng.random_i32(3))));
        }
    }

    pub(crate) fn render(&mut self, cx: &mut PaintCx<'_>, env: JumpRunnerRenderEnv<'_>) {
        match &self.config.terrain {
            Err(err) => unavailable_render(cx, &err.to_string()),
            Ok(_) if self.state.is_some() => self.render_loaded_session(cx, env),
            _ => unavailable_render(cx, "jump state not available"),
        }
    }

    fn render_loaded_session(&mut self, cx: &mut PaintCx<'_>, env: JumpRunnerRenderEnv<'_>) {
        if self.phase().is_none() {
            return unavailable_render(cx, "jump state not available");
        }

        let hill_name_k = env
            .hills
            .hill(self.config.hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let wind_pos = env.wind.position();
        let mut frame = self
            .render_frame(self.last_wind, WIDTH, HEIGHT)
            .expect("loaded jump render frame");
        frame.hr_shake_position = self.hr_shake_position.get();
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
            has_bib: self.has_bib.get(),
            suit_color: self.config.participant.suit_color as usize,
            ski_color: self.config.participant.ski_color as usize,
            show_keymap: if !self.keymap_shown.get()
                && matches!(frame.phase, JumpPhase::Info | JumpPhase::OnBar)
            {
                self.keymap_shown.set(true);
                true
            } else {
                false
            },
        };
        presentation::render(cx, &frame, &ctx);
    }

    fn apply_snow_to_viewport(&mut self, frame: &mut JumpRenderFrame, wind: i32) {
        if self.snow.count() == 0 {
            return;
        }
        let draw = self.draws_snow();
        if let Some(camera) = self.camera() {
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

    fn snapshot(&self) -> Option<JumpSnapshot> {
        match (&self.config.terrain, &self.state) {
            (Ok(terrain), Some(state)) => Some(state.snapshot_with_terrain(terrain)),
            (_, Some(state)) => Some(state.snapshot()),
            _ => None,
        }
    }

    fn tick(&mut self, wind: FlightWind, rng: &mut Random) {
        let phase_change =
            if let (Ok(terrain), Some(state)) = (&self.config.terrain, &mut self.state) {
                let previous_phase = state.phase;
                state.tick(terrain, wind, rng, self.config.policy.count_onbar_frames);
                Some((previous_phase, state.phase))
            } else {
                None
            };
        if let Some((previous_phase, current_phase)) = phase_change {
            self.update_replay_markers(previous_phase, current_phase);
        }
    }

    fn tick_with_wind(&mut self, rng: &mut Random, wind: &mut Wind) -> FlightWind {
        let phase = self.phase();
        let wind_value = if matches!(
            phase,
            Some(JumpPhase::Info | JumpPhase::Result | JumpPhase::Disqualified)
        ) {
            wind.value
        } else {
            wind.sample(rng)
        };
        let sampled = FlightWind {
            value: wind_value,
            windy: wind.windy,
            strength: wind.strength,
        };
        self.tick(sampled, rng);
        if self.phase() == Some(JumpPhase::Result) {
            self.tick(sampled, rng);
        }
        self.record_replay_frame(sampled);
        sampled
    }

    fn update_replay_markers(&mut self, previous_phase: JumpPhase, current_phase: JumpPhase) {
        if previous_phase != JumpPhase::Flight && current_phase == JumpPhase::Flight {
            self.replay.mark_flight_start();
        }
        if previous_phase == JumpPhase::Flight && current_phase == JumpPhase::Landing {
            self.replay.mark_flight_stop();
        }
        if let Some(state) = &self.state {
            if matches!(
                current_phase,
                JumpPhase::Landing | JumpPhase::Result | JumpPhase::Disqualified
            ) {
                self.replay.set_distance(math::round(state.distance * 10.0));
            }
        }
        self.last_phase = Some(current_phase);
    }

    fn record_replay_frame(&mut self, wind: FlightWind) {
        let (current_pos, body_anim, ski_anim, phase) = {
            let (terrain, state) = match (&self.config.terrain, &self.state) {
                (Ok(terrain), Some(state)) => (terrain, state),
                _ => return,
            };
            let (body_anim, ski_anim) = state.anims(terrain);
            ((state.x, state.y), body_anim, ski_anim, state.phase)
        };

        if matches!(phase, JumpPhase::Result | JumpPhase::Disqualified) {
            self.replay.stop();
            return;
        }

        let previous = self.replay_prev_pos.unwrap_or(current_pos);
        self.replay
            .record_frame(previous, current_pos, body_anim, ski_anim, wind.value);
        self.replay_prev_pos = Some(current_pos);
    }

    fn draws_snow(&self) -> bool {
        matches!(
            self.phase(),
            Some(
                JumpPhase::Info
                    | JumpPhase::OnBar
                    | JumpPhase::Inrun
                    | JumpPhase::Flight
                    | JumpPhase::Landing
                    | JumpPhase::Disqualified
            )
        )
    }

    fn camera(&self) -> Option<(i32, i32)> {
        self.state.as_ref().map(|state| (state.sx, state.sy))
    }

    fn render_frame(
        &self,
        wind: FlightWind,
        width: u32,
        height: u32,
    ) -> Result<JumpRenderFrame, AssetError> {
        let (terrain, state) = match (&self.config.terrain, &self.state) {
            (Ok(terrain), Some(state)) => (terrain, state),
            (Err(err), _) => return Err(err.clone()),
            _ => return Err(AssetError::Custom("jump state not available".to_string())),
        };
        let (viewport, snow_mask) =
            terrain.viewport_rgba_and_mask(state.sx, state.sy, width, height);
        let (body_x, body_y) = state.body_position();
        let (body_anim, ski_anim) = state.anims(terrain);
        Ok(JumpRenderFrame {
            viewport: viewport.into(),
            snow_mask: snow_mask.into(),
            phase: state.phase,
            x: state.x,
            y: state.y,
            sx: state.sx,
            sy: state.sy,
            body_x,
            body_y,
            frame_counter: state.frame,
            body_anim,
            ski_anim,
            wind_value: wind.value,
            start_gate: state.start_gate,
            distance: state.distance,
            score: state.score,
            style_points: state.style_points,
            style_revealed: state.style_revealed,
            hill_record_marker: self.record_marker,
            is_hill_record: self.config.policy.save_hill_records
                && self.config.record_distance > 0.0
                && state.distance > self.config.record_distance,
            hr_shake_position: None,
        })
    }
}

fn unavailable_render(cx: &mut PaintCx<'_>, message: &str) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.text((20, 80), FONT_BODY, message);
    cx.text((20, 95), FONT_GRAY, "PRESS ESC");
}

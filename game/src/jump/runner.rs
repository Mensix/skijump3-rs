use crate::data::hill::HillCatalog;
use crate::data::hill_profile::HillTerrain;
use crate::data::records::RecordStore;
use crate::gfx::theme::{BLACK, FONT_BODY, FONT_GRAY};
use crate::jump::config::JumpConfig;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::math;
use crate::jump::presentation::{self, JumpPresentationContext};
use crate::jump::replay::{LiveReplayRecorder, ReplayTrace};
use crate::jump::snow::SnowSystem;
use crate::jump::state::JumpState;
use crate::jump::types::{FlightWind, JumpInput, JumpOutcome, JumpPhase, JumpSnapshot};
use crate::jump::wind::Wind;
use crate::jump::wind::WindPosition;
use crate::jump::{ComputerInputProvider, JumperControl};
use crate::rng::Random;
use crate::text::lang::LangBase;
use engine::consts::{HEIGHT, WIDTH};
use engine::oxide::Font;
use engine::oxide::PaintCx;

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
struct JumpRenderRuntime {
    snow: SnowSystem,
    prev_camera: (i32, i32),
    hr_shake_position: Option<(i32, i32)>,
    keymap_shown: bool,
}

impl JumpRenderRuntime {
    fn new(snow: SnowSystem, state: Option<&JumpState>) -> Self {
        Self {
            snow,
            prev_camera: state.map_or((0, 0), |state| (state.sx, state.sy)),
            hr_shake_position: None,
            keymap_shown: false,
        }
    }

    fn clone_snow(&self) -> SnowSystem {
        self.snow.clone()
    }

    const fn clear_hr_shake(&mut self) {
        self.hr_shake_position = None;
    }

    const fn set_hr_shake(&mut self, position: (i32, i32)) {
        self.hr_shake_position = Some(position);
    }

    const fn hr_shake_position(&self) -> Option<(i32, i32)> {
        self.hr_shake_position
    }

    fn show_keymap(&mut self, phase: JumpPhase) -> bool {
        let show = !self.keymap_shown && matches!(phase, JumpPhase::Info | JumpPhase::OnBar);
        self.keymap_shown |= show;
        show
    }

    fn apply_snow(
        &mut self,
        frame: &mut JumpRenderFrame,
        camera: Option<(i32, i32)>,
        wind: i32,
        draws_snow: bool,
    ) {
        if let Some(camera) = camera {
            self.snow.render_to_viewport(
                &mut frame.viewport,
                &frame.snow_mask,
                self.prev_camera,
                camera,
                wind,
                draws_snow,
            );
            self.prev_camera = camera;
        }
    }
}

#[derive(Debug)]
pub struct JumpRunner {
    config: JumpConfig,
    state: Option<JumpState>,
    replay: LiveReplayRecorder,
    record_marker: Option<(i32, i32)>,
    goal_marker: Option<(i32, i32)>,
    render_runtime: JumpRenderRuntime,
    computer_input: Option<ComputerInputProvider>,
    computer_pre_ai_wind_done: bool,
    last_wind: FlightWind,
    suppress_info_panel: bool,
    has_bib: bool,
}

impl JumpRunner {
    pub(crate) fn new(config: JumpConfig, snow: SnowSystem) -> Self {
        let computer_input = (config.participant.control == JumperControl::Computer)
            .then(|| ComputerInputProvider::new(config.participant.ai_id));
        let state = Self::new_state(&config);
        let record_marker = Self::record_marker(&config, state.as_ref());
        let replay = LiveReplayRecorder::new(&config, state.as_ref(), record_marker);
        let goal_marker = Self::goal_marker(&config, state.as_ref());
        let render_runtime = JumpRenderRuntime::new(snow, state.as_ref());
        Self {
            config,
            state,
            replay,
            record_marker,
            goal_marker,
            render_runtime,
            computer_input,
            computer_pre_ai_wind_done: false,
            last_wind: FlightWind::default(),
            suppress_info_panel: false,
            has_bib: false,
        }
    }

    fn new_state(config: &JumpConfig) -> Option<JumpState> {
        config.hill.as_ref().map(|hill| {
            JumpState::new(
                &config.terrain,
                hill.vx_final as f64,
                hill.pk(),
                hill.kr as i32,
                hill.pl_save(),
                config.start_gate,
            )
        })
    }

    fn record_marker(config: &JumpConfig, _state: Option<&JumpState>) -> Option<(i32, i32)> {
        config.hill.as_ref().and_then(|hill| {
            find_hill_record_marker(&config.terrain, hill.pk(), config.record_distance)
        })
    }

    fn goal_marker(config: &JumpConfig, _state: Option<&JumpState>) -> Option<(i32, i32)> {
        config.hill.as_ref().and_then(|hill| {
            find_hill_record_marker(&config.terrain, hill.pk(), config.goal_distance)
        })
    }

    pub(crate) const fn hill_idx(&self) -> usize {
        self.config.hill_idx
    }

    pub(crate) const fn participant_id(&self) -> usize {
        self.config.participant.id
    }

    pub(crate) fn participant_is_computer(&self) -> bool {
        self.config.participant.control == JumperControl::Computer
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
        self.replay.trace()
    }

    pub(crate) fn clone_snow(&self) -> SnowSystem {
        self.render_runtime.clone_snow()
    }

    pub(crate) fn reset_state(
        &mut self,
        start_gate: i32,
        record_distance: f64,
        goal_distance: f64,
    ) {
        self.config.start_gate = start_gate;
        self.config.record_distance = record_distance;
        self.config.goal_distance = goal_distance;
        if self.config.hill.is_some() {
            self.state = Self::new_state(&self.config);
            self.record_marker = Self::record_marker(&self.config, self.state.as_ref());
            self.goal_marker = Self::goal_marker(&self.config, self.state.as_ref());
            self.replay
                .reset(&self.config, self.state.as_ref(), self.record_marker);
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

    pub(crate) fn set_suppress_info_panel(&mut self, suppress: bool) {
        self.suppress_info_panel = suppress;
    }

    pub(crate) fn set_has_bib(&mut self, val: bool) {
        self.has_bib = val;
    }

    pub(crate) fn update(&mut self, rng: &mut Random, wind: &mut Wind) {
        if self.computer_input.is_some() && !self.computer_pre_ai_wind_done {
            wind.advance_without_sampling(rng);
            self.computer_pre_ai_wind_done = true;
        }

        if let (Some(snapshot), Some(input)) = (self.snapshot(), self.computer_input.as_mut()) {
            for jump_input in input.inputs(&snapshot, rng) {
                self.handle_input(jump_input);
            }
        }
        self.last_wind = self.tick_with_wind(rng, wind);
        self.render_runtime.clear_hr_shake();
        if self.phase() == Some(JumpPhase::Landing)
            && self.config.record_distance > 0.0
            && self
                .state
                .as_ref()
                .is_some_and(|state| state.distance > self.config.record_distance)
            && rng.random_i32(2) == 0
        {
            self.render_runtime
                .set_hr_shake((308 - 1 + rng.random_i32(3), 32 + rng.random_i32(3)));
        }
    }

    pub(crate) fn render(
        &mut self,
        cx: &mut PaintCx<'_>,
        font: &Font,
        langbase: &LangBase,
        hills: &HillCatalog,
        records: &RecordStore,
        wind: &Wind,
    ) {
        if self.state.is_none() {
            return unavailable_render(cx, "jump state not available");
        }

        let hill_name_k = hills
            .hill(self.config.hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let wind_pos = if wind.place() > 10 {
            if let Some(ref st) = self.state {
                wind.position_for_jumper(st.x - st.sx, st.y - st.sy)
            } else {
                wind.position()
            }
        } else {
            wind.position()
        };
        let mut frame = self.render_frame(self.last_wind);
        frame.hr_shake_position = self.render_runtime.hr_shake_position();
        let camera = self.camera();
        let draws_snow = self.draws_snow();
        self.render_runtime
            .apply_snow(&mut frame, camera, wind.value, draws_snow);
        let show_keymap = self.render_runtime.show_keymap(frame.phase);
        let ctx = JumpPresentationContext {
            font,
            langbase,
            jumper_name: self.config.participant.display_name(),
            hill_name_k: &hill_name_k,
            hill_record: self
                .config
                .hill
                .as_ref()
                .and_then(|h| records.hill_record(&h.record_key)),
            wind_position: WindPosition {
                x: wind_pos.x,
                y: wind_pos.y,
            },
            phase_label: &self.config.phase_label,
            team_name: &self.config.team_name,
            allow_gate_adjust: self.config.policy.allow_start_gate_adjust,
            suppress_info_panel: self.suppress_info_panel,
            has_bib: self.has_bib,
            suit_color: self.config.participant.suit_color,
            ski_color: self.config.participant.ski_color,
            show_keymap,
        };
        presentation::render(cx, &frame, &ctx);
    }

    fn snapshot(&self) -> Option<JumpSnapshot> {
        self.state
            .as_ref()
            .map(|state| state.snapshot_with_terrain(&self.config.terrain))
    }

    fn tick(&mut self, wind: FlightWind, rng: &mut Random) {
        if let Some(state) = &mut self.state {
            let previous_phase = state.phase;
            state.tick(
                &self.config.terrain,
                wind,
                rng,
                self.config.policy.count_onbar_frames,
            );
            self.replay
                .on_phase_change(previous_phase, state.phase, state);
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
        if let Some(state) = &self.state {
            self.replay
                .record_frame(&self.config.terrain, state, sampled);
        }
        sampled
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

    fn render_frame(&self, wind: FlightWind) -> JumpRenderFrame {
        let state = self.state.as_ref().expect("jump state not available");
        let terrain = &self.config.terrain;
        let (viewport, snow_mask) = terrain.viewport_rgba_and_mask_with_back(
            state.sx,
            state.sy,
            WIDTH,
            HEIGHT,
            self.config.draw_back,
        );
        let (body_x, body_y) = state.body_position();
        let (body_anim, ski_anim) = state.anims(terrain);
        JumpRenderFrame {
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
            goal_marker: self.goal_marker,
            is_hill_record: self.config.policy.save_hill_records
                && self.config.record_distance > 0.0
                && state.fall_type == crate::jump::types::FallType::None
                && state.distance > self.config.record_distance,
            hr_shake_position: None,
        }
    }
}

fn unavailable_render(cx: &mut PaintCx<'_>, message: &str) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.text((20, 80), FONT_BODY, message);
    cx.text((20, 95), FONT_GRAY, "PRESS ESC");
}

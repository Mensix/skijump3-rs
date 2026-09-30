use crate::data::hill::HillCatalog;
use crate::data::hill_profile::HillTerrain;
use crate::data::records::RecordStore;
use crate::gfx::theme::{BLACK, FONT_BODY, FONT_GRAY};
use crate::jump::animation::select_jumper_sprites;
use crate::jump::config::JumpConfig;
use crate::jump::frame::JumpRenderFrame;
use crate::jump::math;
use crate::jump::presentation::{self, JumpPresentationContext};
use crate::jump::replay::{LiveReplayRecorder, ReplayTrace};
use crate::jump::snow::SnowSystem;
use crate::jump::state::JumpState;
use crate::jump::types::{FallType, FlightWind, JumpInput, JumpOutcome, JumpPhase, JumpSnapshot};
use crate::jump::visuals;
use crate::jump::wind::Wind;
use crate::jump::wind::WindPosition;
use crate::jump::{ComputerInputProvider, JumperControl};
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::rng::Random;
use crate::text::lang::LangBase;
use crate::ui::Font;
use crate::ui::UiCanvas;
use engine::color::Rgba;

fn qualifies_hill_record(
    saves_hill_records: bool,
    record_distance: f64,
    fall_type: FallType,
    distance: f64,
) -> bool {
    saves_hill_records
        && record_distance > 0.0
        && fall_type == FallType::None
        && distance > record_distance
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
struct JumpRenderRuntime {
    snow: SnowSystem,
    prev_camera: (i32, i32),
    hr_shake_position: Option<(i32, i32)>,
}

impl JumpRenderRuntime {
    fn new(snow: SnowSystem, state: Option<&JumpState>) -> Self {
        Self {
            snow,
            prev_camera: state.map_or((0, 0), |state| (state.sx, state.sy)),
            hr_shake_position: None,
        }
    }

    fn clone_snow(&self) -> SnowSystem {
        self.snow.clone()
    }

    fn replace_snow(&mut self, snow: SnowSystem) {
        self.snow = snow;
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

    fn advance_visuals(&mut self, camera: (i32, i32), wind: i32, advance_snow: bool) {
        if advance_snow {
            self.snow.advance(
                self.prev_camera.0 - camera.0,
                self.prev_camera.1 - camera.1,
                wind,
            );
        }
        self.prev_camera = camera;
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
    pre_jump_wind_done: bool,
    last_wind: FlightWind,
    suppress_info_panel: bool,
    has_bib: bool,
}

fn advance_pre_jump_wind_once(done: &mut bool, rng: &mut Random, wind: &mut Wind) {
    if !*done {
        wind.advance(rng);
        *done = true;
    }
}

impl JumpRunner {
    pub(crate) fn new(config: JumpConfig, snow: SnowSystem) -> Self {
        let computer_input = (config.participant.control == JumperControl::Computer)
            .then(|| ComputerInputProvider::new(config.participant.ai_id));
        let state = Self::new_state(&config);
        let record_marker = Self::record_marker(&config);
        let replay = LiveReplayRecorder::new(&config, state.as_ref(), record_marker);
        let goal_marker = Self::goal_marker(&config);
        let render_runtime = JumpRenderRuntime::new(snow, state.as_ref());
        let mut runner = Self {
            config,
            state,
            replay,
            record_marker,
            goal_marker,
            render_runtime,
            computer_input,
            pre_jump_wind_done: false,
            last_wind: FlightWind::default(),
            suppress_info_panel: false,
            has_bib: false,
        };
        runner.refresh_visuals(false);
        runner
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

    fn record_marker(config: &JumpConfig) -> Option<(i32, i32)> {
        config.hill.as_ref().and_then(|hill| {
            find_hill_record_marker(&config.terrain, hill.pk(), config.record_distance)
        })
    }

    fn goal_marker(config: &JumpConfig) -> Option<(i32, i32)> {
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

    pub(crate) fn participant(&self) -> &JumpParticipant {
        &self.config.participant
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

    pub(crate) const fn policy(&self) -> JumpPolicy {
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

    pub(crate) const fn pre_jump_wind_done(&self) -> bool {
        self.pre_jump_wind_done
    }

    pub(crate) fn replace_snow(&mut self, snow: SnowSystem) {
        self.render_runtime.replace_snow(snow);
        self.refresh_visuals(false);
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
            self.record_marker = Self::record_marker(&self.config);
            self.goal_marker = Self::goal_marker(&self.config);
            self.replay
                .reset(&self.config, self.state.as_ref(), self.record_marker);
        }
        self.computer_input = (self.config.participant.control == JumperControl::Computer)
            .then(|| ComputerInputProvider::new(self.config.participant.ai_id));
        self.pre_jump_wind_done = false;
        self.refresh_visuals(false);
    }

    pub(crate) fn set_phase_label(&mut self, label: String) {
        self.config.phase_label = label;
    }

    pub(crate) fn set_team_name(&mut self, name: String) {
        self.config.team_name = name;
    }

    pub(crate) fn team_name(&self) -> &str {
        &self.config.team_name
    }

    pub(crate) fn set_suppress_info_panel(&mut self, suppress: bool) {
        self.suppress_info_panel = suppress;
    }

    pub(crate) fn set_has_bib(&mut self, val: bool) {
        self.has_bib = val;
        self.replay.set_has_bib(val);
    }

    pub(crate) fn update(&mut self, rng: &mut Random, wind: &mut Wind) {
        advance_pre_jump_wind_once(&mut self.pre_jump_wind_done, rng, wind);

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
        self.refresh_visuals(self.draws_snow());
    }

    pub(crate) fn render(
        &self,
        cx: &mut dyn UiCanvas,
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
        let wind_pos = if wind.is_jumper_relative() {
            if let Some(ref st) = self.state {
                wind.position_for_jumper(st.x - st.sx, st.y - st.sy)
            } else {
                wind.position()
            }
        } else {
            wind.position()
        };
        let Some(mut frame) = self.render_frame(self.last_wind) else {
            return;
        };
        frame.hr_shake_position = self.render_runtime.hr_shake_position();
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
            goal_distance: (self.config.goal_distance > 0.0).then_some(self.config.goal_distance),
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
            is_computer: self.participant_is_computer(),
        };
        presentation::render(cx, &mut frame, &ctx);
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
        let sampled = wind.sample_flight_wind(self.phase(), rng);
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

    fn refresh_visuals(&mut self, advance_snow: bool) {
        let Some(camera) = self.camera() else {
            return;
        };
        self.render_runtime
            .advance_visuals(camera, self.last_wind.value, advance_snow);
    }

    fn render_frame(&self, wind: FlightWind) -> Option<JumpRenderFrame> {
        let state = self.state.as_ref()?;
        let terrain = &self.config.terrain;
        let (body_x, body_y) = state.body_position();
        let sprites = select_jumper_sprites(state.animation_context(terrain));
        Some(JumpRenderFrame {
            back_layer: terrain.back_layer(),
            front_layer: terrain.front_layer(),
            snow_pixels: self.render_runtime.snow.pixel_draws(),
            phase: state.phase,
            x: state.x,
            y: state.y,
            sx: state.sx,
            sy: state.sy,
            body_x,
            body_y,
            frame_counter: state.frame,
            body_anim: sprites.body,
            ski_anim: sprites.skis,
            wind_value: wind.value,
            start_gate: state.start_gate,
            distance: state.distance,
            score: state.score,
            style_points: state.style_points,
            style_revealed: state.style_revealed,
            hill_record_marker: self.record_marker,
            goal_marker: self.goal_marker,
            is_hill_record: qualifies_hill_record(
                self.config.policy.save_hill_records,
                self.config.record_distance,
                state.fall_type,
                state.distance,
            ),
            hr_shake_position: None,
            bar_gag_position: state.bar_gag_position,
        })
    }

    pub(crate) fn set_save_hill_records(&mut self, save: bool) {
        self.config.policy.save_hill_records = save;
    }

    pub(crate) fn render_hill_background(
        &self,
        cx: &mut dyn UiCanvas,
        terrain: Option<&HillTerrain>,
        modulation: Option<Rgba>,
    ) -> bool {
        let Some((scroll_x, scroll_y)) = self.camera() else {
            return false;
        };
        let terrain = terrain.unwrap_or(&self.config.terrain);
        let back_layer = terrain.back_layer();
        let front_layer = terrain.front_layer();
        visuals::push_hill_layers(
            cx,
            &back_layer,
            &front_layer,
            engine::oxide::PointBatches::empty(),
            scroll_x,
            scroll_y,
            modulation,
        );
        true
    }
}

fn unavailable_render(cx: &mut dyn UiCanvas, message: &str) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.text((20, 80), FONT_BODY, message);
    cx.text((20, 95), FONT_GRAY, "PRESS ESC");
}

#[cfg(test)]
mod tests {
    use super::{advance_pre_jump_wind_once, qualifies_hill_record};
    use crate::jump::types::FallType;
    use crate::jump::wind::Wind;
    use crate::rng::Random;

    #[test]
    fn visible_participant_gets_one_pre_jump_wind_advance() {
        let mut actual_rng = Random::new(42);
        let mut actual_wind = Wind::default();
        actual_wind.initialize(&mut actual_rng, 0);
        let mut done = false;

        advance_pre_jump_wind_once(&mut done, &mut actual_rng, &mut actual_wind);
        advance_pre_jump_wind_once(&mut done, &mut actual_rng, &mut actual_wind);

        let mut expected_rng = Random::new(42);
        let mut expected_wind = Wind::default();
        expected_wind.initialize(&mut expected_rng, 0);
        expected_wind.advance(&mut expected_rng);

        assert!(done);
        assert_eq!(
            actual_wind.sample(&mut actual_rng),
            expected_wind.sample(&mut expected_rng)
        );
        assert_eq!(
            actual_rng.random_i32(1_000_000),
            expected_rng.random_i32(1_000_000)
        );
    }

    #[test]
    fn record_indicator_obeys_runtime_record_policy() {
        assert!(qualifies_hill_record(true, 100.0, FallType::None, 101.0));
        assert!(!qualifies_hill_record(false, 100.0, FallType::None, 101.0));
        assert!(!qualifies_hill_record(true, 100.0, FallType::Normal, 101.0));
    }
}

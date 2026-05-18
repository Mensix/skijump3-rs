use crate::data::hill_profile::HillTerrain;
use crate::data::records::HillInfo;
use crate::jump::config::{JumpConfig, JumpParticipant};
use crate::jump::frame::JumpRenderFrame;
use crate::jump::math;
use crate::jump::policy::JumpPolicy;
use crate::jump::replay::{ReplayMeta, ReplayRecorder, ReplayTrace};
use crate::jump::types::{FlightWind, JumpInput, JumpOutcome, JumpPhase, JumpSnapshot};
use crate::jump::wind::Wind;
use crate::jump::JumpState;
use crate::rng::Random;

fn find_hill_record_marker(
    terrain: &HillTerrain,
    pk: f64,
    record_distance: i32,
) -> Option<(i32, i32)> {
    if record_distance <= 0 {
        return None;
    }
    let tip_x = terrain.tip_x;
    let tip_y = terrain.height_at(tip_x);
    for x in tip_x..1024 {
        let dx = f64::from(x - tip_x);
        let dy = f64::from(terrain.height_at(x) - tip_y);
        let hp = math::round(dx.hypot(dy) * pk * 0.5) * 5;
        if hp >= record_distance {
            let ground_y = terrain.height_at(x);
            return Some((x, ground_y - 9));
        }
    }
    None
}

#[derive(Debug)]
pub struct JumpSession {
    terrain: Result<HillTerrain, String>,
    state: Option<JumpState>,
    replay_prev_pos: Option<(i32, i32)>,
    replay: ReplayRecorder,
    hill_idx: usize,
    snow_count: u16,
    participant: JumpParticipant,
    last_phase: Option<JumpPhase>,
    policy: JumpPolicy,
    record_distance: i32,
    record_marker: Option<(i32, i32)>,
}

impl JumpSession {
    pub(crate) fn new(config: JumpConfig) -> Self {
        let JumpConfig {
            terrain,
            hill,
            hill_idx,
            start_gate,
            snow_count,
            participant,
            policy,
            record_distance,
            ..
        } = config;
        let state = match (&terrain, hill.as_ref()) {
            (Ok(terrain), Some(hill)) => Some(JumpState::new(
                terrain,
                hill.vx_final as f64,
                hill.pk(),
                hill.kr as i32,
                hill.pl_save(),
                start_gate,
            )),
            _ => None,
        };
        let replay_prev_pos = state.as_ref().map(|state| (state.x, state.y));
        let last_phase = state.as_ref().map(|state| state.phase);
        let mut replay = ReplayRecorder::default();
        if let Some(state) = &state {
            replay.start(Self::replay_meta(
                state,
                hill_idx,
                snow_count,
                &participant,
                start_gate,
            ));
        }

        let mut record_marker = None;
        if let (Ok(terrain), Some(hill)) = (&terrain, hill.as_ref()) {
            record_marker = find_hill_record_marker(terrain, hill.pk(), record_distance);
        }

        Self {
            terrain,
            state,
            replay_prev_pos,
            replay,
            hill_idx,
            snow_count,
            participant,
            last_phase,
            policy,
            record_distance,
            record_marker,
        }
    }

    fn replay_meta(
        state: &JumpState,
        hill_idx: usize,
        snow_count: u16,
        participant: &JumpParticipant,
        start_gate: i32,
    ) -> ReplayMeta {
        ReplayMeta {
            start_x: state.x,
            start_y: state.y,
            hill_idx,
            snow_count,
            distance: 0,
            flight_start: 0,
            flight_stop: 0,
            hill_record_marker: None,
            hill_filename: "HILLBASE".to_string(),
            hill_profile: 0,
            suit_color: participant.suit_color,
            ski_color: participant.ski_color,
            saved_at: String::new(),
            has_bib: false,
            author: String::new(),
            name: participant.display_name().to_string(),
            start_gate_or_competition: 100 - start_gate,
            frame_count: 0,
            checksum: 0,
            valid_checksum: true,
            intro: false,
        }
    }

    pub(crate) const fn terrain(&self) -> &Result<HillTerrain, String> {
        &self.terrain
    }

    pub(crate) const fn state(&self) -> Option<&JumpState> {
        self.state.as_ref()
    }

    pub(crate) fn snapshot(&self) -> Option<JumpSnapshot> {
        match (&self.terrain, &self.state) {
            (Ok(terrain), Some(state)) => Some(state.snapshot_with_terrain(terrain)),
            (_, Some(state)) => Some(state.snapshot()),
            _ => None,
        }
    }

    pub(crate) fn frame_counter(&self) -> i32 {
        self.state.as_ref().map_or(0, |state| state.frame)
    }

    pub(crate) fn phase(&self) -> Option<JumpPhase> {
        self.state.as_ref().map(|state| state.phase)
    }

    pub(crate) fn start_gate(&self) -> Option<i32> {
        self.state.as_ref().map(|state| state.start_gate)
    }

    pub(crate) const fn policy(&self) -> JumpPolicy {
        self.policy
    }

    pub(crate) fn reset_state(&mut self, hill: &HillInfo, start_gate: i32, record_distance: i32) {
        self.state = self.terrain.as_ref().ok().map(|terrain| {
            JumpState::new(
                terrain,
                hill.vx_final as f64,
                hill.pk(),
                hill.kr as i32,
                hill.pl_save(),
                start_gate,
            )
        });
        self.record_distance = record_distance;
        self.record_marker = match (&self.terrain, &self.state) {
            (Ok(terrain), Some(_)) => find_hill_record_marker(terrain, hill.pk(), record_distance),
            _ => None,
        };
        self.replay_prev_pos = self.state.as_ref().map(|state| (state.x, state.y));
        self.last_phase = self.state.as_ref().map(|state| state.phase);
        if let Some(state) = &self.state {
            self.replay.start(Self::replay_meta(
                state,
                self.hill_idx,
                self.snow_count,
                &self.participant,
                start_gate,
            ));
        }
    }

    pub(crate) fn prepare_silent_computer_jump(&mut self) {
        if let (Ok(terrain), Some(state)) = (&self.terrain, &mut self.state) {
            state.prepare_silent_computer_jump(terrain);
            self.replay_prev_pos = Some((state.x, state.y));
            self.last_phase = Some(state.phase);
        }
    }

    pub(crate) fn handle_input(&mut self, input: JumpInput) {
        if let Some(state) = &mut self.state {
            state.handle_input(input);
        }
    }

    pub(crate) fn handle_start_gate_adjust(&mut self, delta: i32) -> Option<i32> {
        if !self.policy.allow_start_gate_adjust {
            return self.start_gate();
        }
        self.handle_input(JumpInput::AdjustGate(delta));
        self.start_gate()
    }

    pub(crate) fn tick(&mut self, wind: FlightWind, rng: &mut Random) {
        let phase_change = if let (Ok(terrain), Some(state)) = (&self.terrain, &mut self.state) {
            let previous_phase = state.phase;
            state.tick(terrain, wind, rng, self.policy.count_onbar_frames);
            Some((previous_phase, state.phase))
        } else {
            None
        };
        if let Some((previous_phase, current_phase)) = phase_change {
            self.update_replay_markers(previous_phase, current_phase);
        }
    }

    pub(crate) fn tick_with_wind(&mut self, rng: &mut Random, wind: &mut Wind) -> FlightWind {
        let phase = self.phase();
        let wind_value = if matches!(phase, Some(JumpPhase::Info | JumpPhase::Result)) {
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
            if current_phase == JumpPhase::Landing || current_phase == JumpPhase::Result {
                self.replay.set_distance(state.distance);
            }
        }
        self.last_phase = Some(current_phase);
    }

    pub(crate) fn outcome(&self) -> Option<JumpOutcome> {
        self.state.as_ref()?.outcome()
    }

    pub(crate) fn draws_snow(&self) -> bool {
        matches!(
            self.phase(),
            Some(
                JumpPhase::Info
                    | JumpPhase::OnBar
                    | JumpPhase::Inrun
                    | JumpPhase::Flight
                    | JumpPhase::Landing
            )
        )
    }

    pub(crate) fn camera(&self) -> Option<(i32, i32)> {
        self.state.as_ref().map(|state| (state.sx, state.sy))
    }

    pub(crate) fn render_frame(
        &mut self,
        wind: FlightWind,
        width: u32,
        height: u32,
    ) -> Result<JumpRenderFrame, String> {
        let (frame, current_pos, body_anim, ski_anim) = {
            let (terrain, state) = match (&self.terrain, &mut self.state) {
                (Ok(terrain), Some(state)) => (terrain, state),
                (Err(err), _) => return Err(err.clone()),
                _ => return Err("jump state not available".to_string()),
            };
            let viewport = terrain.viewport_pixels(state.sx, state.sy, width, height);
            let (body_x, body_y) = state.body_position();
            let (body_anim, ski_anim) = state.anims(terrain);
            let current_pos = (state.x, state.y);
            let frame = JumpRenderFrame {
                viewport,
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
            };
            (frame, current_pos, body_anim, ski_anim)
        };
        let previous = self.replay_prev_pos.unwrap_or(current_pos);
        if frame.phase == JumpPhase::Result {
            self.replay.stop();
        } else {
            self.replay
                .record_frame(previous, current_pos, body_anim, ski_anim, wind.value);
            self.replay_prev_pos = Some(current_pos);
        }
        Ok(frame)
    }

    pub(crate) fn replay_trace(&self) -> Option<ReplayTrace> {
        self.replay.finish()
    }
}

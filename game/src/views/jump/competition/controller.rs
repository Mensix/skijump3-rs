use std::marker::PhantomData;

use crate::competition::active::ActiveCompetition;
use crate::competition::koth::types::KothRuntime;
use crate::competition::machine::Competition;
use crate::competition::runtime::{
    CompetitionDecision, CompetitionJumpMetadata, CompetitionRuntime,
};
use crate::competition::team_cup::types::TeamCupRuntime;
use crate::components::modal::Modal;
use crate::data::hill::FALLBACK_HILL_FILENAME;
use crate::data::hill_profile::HillProfileMismatch;
use crate::data::records::HillRecord;
use crate::jump::types::{FallType, JumpOutcome, JumpPhase};
use crate::jump::{JumpParticipant, JumperControl};
use crate::screen::{NavSignal, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::{current_timestamp, display_timestamp_now};
use crate::ui::UiCanvas;
use crate::ui::{Key, UiEvent};
use crate::views::jump::competition::flow::{
    handle_human_jump, handle_jump_scene_event, render_jump_scene_with_overlay,
    CompetitionFlowCommand, HumanJumpSetup, JumpEventOptions, JumpInputResult,
};
use crate::views::jump::competition::overlay::CompetitionOverlay;
use crate::views::jump::competition::ui_state::{CompetitionUiState, RenderMode};
use crate::views::jump::profile_updates::apply_profile_jump_side_effects;
use crate::views::jump::scene::JumpScene;
use crate::views::records::RecordNotification;

pub(crate) struct CompetitionJumpController<R>
where
    R: CompetitionRuntime + RuntimeAccess + 'static,
{
    resources: ResourcesRef,
    scene: Option<JumpScene>,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    event_weather: EventWeatherLifecycle,
    profiles_saved: bool,
    processed_hidden_this_update: bool,
    background_hill_idx: Option<usize>,
    profile_alert: Option<HillProfileMismatch>,
    _runtime: PhantomData<R>,
}

impl<R> CompetitionJumpController<R>
where
    R: CompetitionRuntime + RuntimeAccess + 'static,
{
    pub(crate) fn new(resources: ResourcesRef, compact: bool, scene: Option<JumpScene>) -> Self {
        let overlay = CompetitionOverlay::new(resources.clone());
        Self {
            resources,
            scene,
            ui_state: CompetitionUiState::new_with_compact(compact),
            overlay,
            event_weather: EventWeatherLifecycle::default(),
            profiles_saved: false,
            processed_hidden_this_update: false,
            background_hill_idx: None,
            profile_alert: None,
            _runtime: PhantomData,
        }
    }

    pub(crate) fn check_hill_profile(&mut self, hill_idx: usize) -> bool {
        if self.profile_alert.is_some() {
            return true;
        }
        if let Some(mismatch) = self.resources.verify_hill_profile(hill_idx) {
            self.profile_alert = Some(mismatch);
            return true;
        }
        false
    }

    pub(crate) const fn has_profile_alert(&self) -> bool {
        self.profile_alert.is_some()
    }

    pub(crate) fn handle_profile_alert_dismiss(&self, event: UiEvent) -> Option<bool> {
        self.profile_alert.as_ref()?;
        match event {
            UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::TextWithModifiers(_, _) => Some(true),
            UiEvent::Quit | UiEvent::Tick => Some(false),
        }
    }

    pub(crate) fn render_profile_alert(&self, cx: &mut dyn UiCanvas) {
        if let Some(mismatch) = &self.profile_alert {
            Modal::hill_profile_mismatch(&mismatch.front_index, mismatch.exiting_cup)
                .paint(cx, &self.resources.langbase);
        }
    }

    pub(crate) fn handle_alert_input(
        &self,
        notification: &mut Option<RecordNotification>,
        event: UiEvent,
    ) -> Option<NavSignal> {
        if let Some(dismiss) = self.handle_profile_alert_dismiss(event) {
            return Some(if dismiss {
                NavSignal::Back
            } else {
                NavSignal::None
            });
        }
        if notification.is_some() {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                *notification = None;
                return Some(NavSignal::Back);
            }
            return Some(NavSignal::None);
        }
        None
    }

    pub(crate) fn paint_alerts(
        &self,
        notification: &Option<RecordNotification>,
        cx: &mut dyn UiCanvas,
    ) -> bool {
        if self.has_profile_alert() {
            self.render_profile_alert(cx);
            return true;
        }
        if let Some(notification) = notification {
            notification.paint(cx, self.resources());
            return true;
        }
        false
    }

    pub(crate) fn resources(&self) -> &ResourcesRef {
        &self.resources
    }

    pub(crate) fn ui_state(&self) -> &CompetitionUiState {
        &self.ui_state
    }

    pub(crate) fn ui_state_mut(&mut self) -> &mut CompetitionUiState {
        &mut self.ui_state
    }

    pub(crate) fn scene(&self) -> Option<&JumpScene> {
        self.scene.as_ref()
    }

    pub(crate) fn scene_mut(&mut self) -> Option<&mut JumpScene> {
        self.scene.as_mut()
    }

    pub(crate) fn render_mode(&self) -> RenderMode {
        self.ui_state.render_mode()
    }

    pub(crate) fn drive(
        &mut self,
        state: &mut GameState,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        if self.profile_alert.is_some() {
            return None;
        }
        self.processed_hidden_this_update = false;
        self.ensure_scene(state);
        self.scene.as_ref()?;
        self.drive_competition(state)
    }

    pub(crate) fn record_acknowledged_human_jump(
        &mut self,
        state: &mut GameState,
        cx: &Persistence,
    ) -> bool {
        if self.profile_alert.is_some() {
            return false;
        }
        if !self.ui_state.is_result_acknowledged() || self.ui_state.is_outcome_recorded() {
            return false;
        }
        let outcome = self.scene.as_mut().and_then(|scene| {
            let outcome = scene.outcome()?;
            let phase = scene.phase();
            scene.collect_telemetry();
            Some((outcome, phase))
        });
        let Some((outcome, phase)) = outcome else {
            return false;
        };
        let recorded = {
            let mut side_effects = None;
            let recorded = R::runtime_mut(&mut state.active_competition)
                .map(|runtime| {
                    let ctx = runtime.current_jump_context();
                    side_effects = Some(runtime.jump_metadata(&ctx));
                    runtime.record_jump_runtime(&ctx, outcome);
                    true
                })
                .unwrap_or(false);
            if recorded {
                if let Some(side_effects) = side_effects {
                    let is_new_record = apply_post_jump_side_effects(
                        state,
                        &self.resources,
                        &side_effects,
                        outcome,
                        phase != Some(JumpPhase::Disqualified),
                    );
                    if is_new_record && state.config.auto_hill_record_replay != 0 {
                        if let Some(scene) = self.scene.as_ref() {
                            if let Some(mut trace) = scene.replay_trace() {
                                trace.meta.author.clone_from(&side_effects.jumper_name);
                                trace.meta.saved_at = display_timestamp_now();
                                let hill = self
                                    .resources
                                    .hills
                                    .hill(trace.meta.hill_idx)
                                    .map(|hill| (hill.name.clone(), hill.kr));
                                let (hill_name, hill_kr) =
                                    hill.unwrap_or_else(|| (FALLBACK_HILL_FILENAME.to_string(), 0));
                                trace.meta.name = format!("HILL RECORD AT {hill_name} K{hill_kr}");
                                let filename = automatic_hill_record_filename(&hill_name);
                                cx.write_file(&filename, &trace.to_sjr_bytes());
                            }
                        }
                    }
                }
            }
            recorded
        };
        if !recorded {
            return false;
        }
        self.ui_state.mark_outcome_recorded();
        true
    }

    pub(crate) fn update_scene(&mut self, state: &mut GameState) {
        if self.profile_alert.is_some() {
            return;
        }
        if self.processed_hidden_this_update {
            return;
        }
        if let Some(scene) = self.scene.as_mut() {
            scene.update(state);
        }
    }

    pub(crate) fn render_jump(&mut self, cx: &mut dyn UiCanvas, state: &GameState) {
        if self.profile_alert.is_some() {
            self.render_profile_alert(cx);
            return;
        }
        if let Some(scene) = self.scene.as_mut() {
            render_jump_scene_with_overlay(cx, scene, &self.overlay, &self.ui_state, state);
        }
    }

    pub(crate) fn render_results_background(
        &self,
        cx: &mut dyn UiCanvas,
        state: &GameState,
    ) -> bool {
        state.config.invisible_back != 0
            && self.scene.as_ref().is_some_and(|scene| {
                scene.render_darkened_hill_background(cx, self.background_hill_idx)
            })
    }

    pub(crate) fn handle_jump_scene_event(
        &mut self,
        event: UiEvent,
        consume_other_actions: bool,
        accepts_only_enter_escape: bool,
        acknowledge_only_unrecorded: bool,
        state: &mut GameState,
        cx: &Persistence,
    ) -> JumpInputResult {
        if matches!(event, UiEvent::KeyDown(Key::Escape)) && self.finish_visible_computer(state) {
            return JumpInputResult::Consumed;
        }
        let Some(scene) = self.scene.as_mut() else {
            return JumpInputResult::None;
        };
        handle_jump_scene_event(
            scene,
            &mut self.ui_state,
            event,
            JumpEventOptions {
                consume_other_actions,
                accepts_only_enter_escape,
                acknowledge_only_unrecorded,
            },
            state,
            cx,
        )
    }

    fn finish_visible_computer(&mut self, state: &mut GameState) -> bool {
        let Some(scene) = self.scene.as_ref() else {
            return false;
        };
        if !scene.participant_is_computer() || scene.outcome().is_some() {
            return false;
        }

        let participant = scene.participant().clone();
        let hill_idx = scene.hill_idx();
        let outcome = scene.simulate_hidden_after_visible(
            participant,
            hill_idx,
            &mut state.rng,
            &mut state.wind,
        );
        let side_effects = R::runtime_mut(&mut state.active_competition).map(|runtime| {
            let context = runtime.current_jump_context();
            let side_effects = runtime.jump_metadata(&context);
            runtime.record_jump_runtime(&context, outcome);
            side_effects
        });
        let Some(side_effects) = side_effects else {
            return false;
        };
        apply_post_jump_side_effects(state, &self.resources, &side_effects, outcome, true);
        self.ui_state.mark_outcome_recorded();
        true
    }

    pub(crate) fn prepare_human_jump(
        &mut self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
        team_name: Option<String>,
        saves_hill_records: bool,
        state: &mut GameState,
    ) {
        if self.check_hill_profile(hill_idx) {
            return;
        }
        handle_human_jump(
            &mut self.scene,
            &mut self.ui_state,
            &self.resources,
            state,
            HumanJumpSetup {
                participant,
                hill_idx,
                phase_label,
                team_name,
            },
        );
        if let Some(scene) = self.scene.as_mut() {
            scene.set_save_hill_records(saves_hill_records);
        }
        self.ui_state.enter_jump();
    }

    pub(crate) fn enter_results(&mut self) {
        self.ui_state.enter_results();
    }

    pub(crate) fn enter_done(&mut self) {
        self.ui_state.enter_done();
    }

    fn advance_results(
        &mut self,
        state: &mut GameState,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        self.scene.as_ref()?;
        self.advance_results_and_drive(state)
    }

    pub(crate) fn save_results(&mut self, state: &GameState, cx: &Persistence) {
        if self.profiles_saved {
            return;
        }
        self.profiles_saved = true;
        cx.save_players(&state.profiles);
        cx.save_records(&state.records);
    }

    pub(crate) fn dismiss_results_and_advance(
        &mut self,
        state: &mut GameState,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        self.ui_state.dismiss_results();
        self.advance_results(state)
    }

    fn ensure_scene(&mut self, state: &mut GameState) {
        if self.scene.is_some() {
            return;
        }
        self.scene = Some(JumpScene::new_competition_placeholder(
            self.resources.clone(),
            state,
        ));
    }

    #[allow(clippy::type_complexity)]
    fn drive_competition(
        &mut self,
        state: &mut GameState,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        let visible_computers = state.config.visible_computers;
        let wind_position = state.config.wind_position as u8;
        let low_detail = state.config.graphics_detail == 1;
        let mut pending_side_effects = Vec::new();
        let GameState {
            active_competition,
            rng,
            wind,
            ..
        } = state;
        let command = R::runtime_mut(active_competition).map(|runtime| loop {
            match runtime.decide_next_runtime() {
                CompetitionDecision::ShowResults(kind) => {
                    return Some(CompetitionFlowCommand::ShowResults(kind));
                }
                CompetitionDecision::Done => {
                    return Some(CompetitionFlowCommand::Done);
                }
                CompetitionDecision::Jump {
                    participant,
                    hill_idx,
                    context,
                    is_human,
                } => {
                    if self.check_hill_profile(hill_idx) {
                        pending_side_effects.clear();
                        return None;
                    }
                    self.background_hill_idx = Some(hill_idx);
                    let current_event = runtime.event_idx();
                    self.event_weather.prepare_if_needed(current_event, || {
                        let Some(scene) = self.scene.as_mut() else {
                            return;
                        };
                        scene.prepare_competition_event_weather(
                            rng,
                            wind,
                            wind_position,
                            low_detail,
                        );
                    });
                    if is_human
                        || self.should_show_computer_jump(
                            runtime,
                            &context,
                            &participant,
                            visible_computers,
                        )
                    {
                        return Some(CompetitionFlowCommand::HumanJump {
                            participant,
                            hill_idx,
                            context,
                        });
                    }

                    let Some(scene) = self.scene.as_ref() else {
                        break None;
                    };
                    let outcome = scene.simulate_hidden(participant, hill_idx, rng, wind);
                    self.processed_hidden_this_update = true;
                    pending_side_effects.push((runtime.jump_metadata(&context), outcome));
                    runtime.record_jump_runtime(&context, outcome);
                    if runtime.is_complete_runtime() {
                        return Some(CompetitionFlowCommand::Done);
                    }
                }
            }
        });

        if !pending_side_effects.is_empty() {
            for (side_effects, outcome) in pending_side_effects {
                apply_post_jump_side_effects(state, &self.resources, &side_effects, outcome, true);
            }
        }

        command.unwrap_or(None)
    }

    fn advance_results_and_drive(
        &mut self,
        state: &mut GameState,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        if let Some(runtime) = R::runtime_mut(&mut state.active_competition) {
            runtime.advance_results_runtime();
        }
        self.drive_competition(state)
    }

    fn should_show_computer_jump(
        &self,
        runtime: &R,
        context: &R::Context,
        participant: &JumpParticipant,
        visible_computers: i32,
    ) -> bool {
        if participant.control != JumperControl::Computer {
            return false;
        }
        match visible_computers {
            0..=234 => participant.ai_id == visible_computers as usize,
            235 => runtime.start_order_pos_for_context(context) < 1,
            236 => runtime.start_order_pos_for_context(context) < 3,
            237 => runtime.start_order_pos_for_context(context) < 5,
            238 => runtime.start_order_pos_for_context(context) < 10,
            239 => true,
            _ => false,
        }
    }
}

fn automatic_hill_record_filename(hill_name: &str) -> String {
    let short_name: String = hill_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(3)
        .collect();
    format!(
        "HR-{}.SJR",
        if short_name.is_empty() {
            "HIL"
        } else {
            &short_name
        }
    )
}

#[derive(Default)]
struct EventWeatherLifecycle {
    event: Option<usize>,
}

impl EventWeatherLifecycle {
    fn prepare_if_needed(&mut self, current_event: usize, prepare: impl FnOnce()) -> bool {
        let changed = self.event != Some(current_event);
        if changed {
            prepare();
            self.event = Some(current_event);
        }
        changed
    }
}

fn apply_post_jump_side_effects(
    state: &mut GameState,
    resources: &ResourcesRef,
    side_effects: &CompetitionJumpMetadata,
    outcome: JumpOutcome,
    started: bool,
) -> bool {
    if profile_attempt_counts(outcome, started) {
        if let Some(profile_idx) = side_effects.profile_idx {
            if let Some(profile) = state.profiles.profiles.get_mut(profile_idx) {
                apply_profile_jump_side_effects(
                    profile,
                    side_effects.hill_idx,
                    hill_file_name(resources, side_effects.hill_idx),
                    hill_display_name(resources, side_effects.hill_idx),
                    outcome,
                    started,
                    side_effects
                        .is_real_world_cup
                        .then(|| hill_display_name(resources, side_effects.hill_idx)),
                );
            }
        }
    }

    let computer_records_enabled =
        !side_effects.is_computer || state.config.computer_hill_records != 0;
    if computer_records_enabled
        && side_effects.saves_hill_records
        && outcome.fall_type == FallType::None
    {
        if let Some(hill) = resources.hills.hill(side_effects.hill_idx) {
            let key = &hill.record_key;
            let is_new = outcome.distance > state.records.hill_record(key).map_or(0.0, |r| r.len);
            if is_new {
                state.records.set_hill_record(
                    key,
                    HillRecord {
                        name: side_effects.jumper_name.clone(),
                        len: outcome.distance,
                        time: current_timestamp(),
                        is_computer: side_effects.is_computer,
                    },
                );
                return true;
            }
        }
    }
    false
}

fn profile_attempt_counts(outcome: JumpOutcome, started: bool) -> bool {
    started && !outcome.aborted
}

fn hill_file_name(resources: &ResourcesRef, hill_idx: usize) -> String {
    resources.hills.hill(hill_idx).map_or_else(
        || FALLBACK_HILL_FILENAME.to_string(),
        |hill| hill.terrain_id.clone(),
    )
}

fn hill_display_name(resources: &ResourcesRef, hill_idx: usize) -> String {
    resources
        .hills
        .hill(hill_idx)
        .map_or_else(String::new, |hill| hill.name.clone())
}

pub(crate) trait RuntimeAccess: CompetitionRuntime {
    fn runtime_mut(active_competition: &mut Option<ActiveCompetition>) -> Option<&mut Self>;
}

impl RuntimeAccess for Competition {
    fn runtime_mut(active_competition: &mut Option<ActiveCompetition>) -> Option<&mut Self> {
        active_competition.as_mut()?.individual_mut()
    }
}

impl RuntimeAccess for TeamCupRuntime {
    fn runtime_mut(active_competition: &mut Option<ActiveCompetition>) -> Option<&mut Self> {
        active_competition.as_mut()?.team_cup_runtime_mut()
    }
}

impl RuntimeAccess for KothRuntime {
    fn runtime_mut(active_competition: &mut Option<ActiveCompetition>) -> Option<&mut Self> {
        active_competition.as_mut()?.koth_runtime_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::{profile_attempt_counts, EventWeatherLifecycle};
    use crate::jump::snow::SnowSystem;
    use crate::jump::types::{FallType, JumpOutcome, LandingStyle};
    use crate::jump::wind::Wind;
    use crate::rng::Random;
    use crate::views::jump::scene::JumpScene;

    fn weather_after_first_participant() -> (u16, i32, i32, i32, i32) {
        let mut lifecycle = EventWeatherLifecycle::default();
        let mut snow = SnowSystem::default();
        let mut wind = Wind::default();
        let mut rng = Random::new(5489);

        let initialized = lifecycle.prepare_if_needed(0, || {
            JumpScene::initialize_event_weather(&mut snow, &mut rng, &mut wind, 0, false);
        });

        assert!(
            initialized,
            "event 0 must initialize before visibility selection"
        );
        (
            snow.count(),
            wind.strength,
            wind.windy,
            wind.value,
            rng.random_i32(1_000_000),
        )
    }

    #[test]
    fn visible_computers_does_not_control_event_weather_initialization() {
        assert_eq!(
            weather_after_first_participant(),
            weather_after_first_participant()
        );
    }

    #[test]
    fn event_weather_is_not_initialized_twice() {
        let expected = weather_after_first_participant();
        let mut lifecycle = EventWeatherLifecycle::default();
        let mut snow = SnowSystem::default();
        let mut wind = Wind::default();
        let mut rng = Random::new(5489);
        let mut initialization_count = 0;

        for _ in 0..2 {
            lifecycle.prepare_if_needed(0, || {
                initialization_count += 1;
                JumpScene::initialize_event_weather(&mut snow, &mut rng, &mut wind, 0, false);
            });
        }

        assert_eq!(initialization_count, 1);
        assert_eq!(
            (
                snow.count(),
                wind.strength,
                wind.windy,
                wind.value,
                rng.random_i32(1_000_000),
            ),
            expected
        );
    }

    #[test]
    fn koth_no_wind_event_keeps_zero_strength() {
        let mut snow = SnowSystem::default();
        let mut wind = Wind::default();
        wind.set_enabled(false);
        let mut rng = Random::new(5489);

        JumpScene::initialize_event_weather(&mut snow, &mut rng, &mut wind, 0, false);

        assert_eq!(wind.strength, 0);
        assert_eq!(wind.value, 0);
    }

    fn outcome(aborted: bool) -> JumpOutcome {
        JumpOutcome {
            distance: 100.0,
            score: 100.0,
            style_points: [18.0; 5],
            landing_style: LandingStyle::Telemark,
            fall_type: FallType::None,
            injury: 0,
            aborted,
        }
    }

    #[test]
    fn only_started_non_aborted_attempts_count_for_profile_side_effects() {
        assert!(profile_attempt_counts(outcome(false), true));
        assert!(!profile_attempt_counts(outcome(true), true));
        assert!(!profile_attempt_counts(outcome(false), false));
    }
}

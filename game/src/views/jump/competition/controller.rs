use std::marker::PhantomData;

use crate::competition::active::ActiveCompetition;
use crate::competition::koth::types::KothRuntime;
use crate::competition::machine::Competition;
use crate::competition::runtime::{CompetitionDecision, CompetitionRuntime};
use crate::competition::team_cup::types::TeamCupRuntime;
use crate::data::records::HillRecord;
use crate::jump::types::{FallType, JumpOutcome};
use crate::jump::{JumpParticipant, JumpPolicy, JumperControl};
use crate::save::SaveRef;
use crate::store::{GameState, ResourcesRef};
use crate::views::jump::competition::flow::{
    command_or_error, handle_human_jump, handle_jump_scene_event, render_jump_scene_with_overlay,
    CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::competition::overlay::CompetitionOverlay;
use crate::views::jump::competition::persistence;
use crate::views::jump::competition::ui_state::{CompetitionUiState, RenderMode};
use crate::views::jump::scene::{JumpScene, JumpSceneError};
use engine::oxide::input::UiEvent;
use engine::oxide::PaintCx;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CompetitionControllerError {
    #[error("AI simulation failed: {0}")]
    JumpScene(#[from] JumpSceneError),
}

pub(crate) struct CompetitionJumpController<R>
where
    R: CompetitionRuntime + RuntimeAccess + 'static,
{
    resources: ResourcesRef,
    save_manager: SaveRef,
    scene: Option<JumpScene>,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    last_event: usize,
    profiles_saved: bool,
    _runtime: PhantomData<R>,
}

impl<R> CompetitionJumpController<R>
where
    R: CompetitionRuntime + RuntimeAccess + 'static,
{
    pub(crate) fn new(
        resources: ResourcesRef,
        save_manager: SaveRef,
        compact: bool,
        scene: Option<JumpScene>,
    ) -> Self {
        let overlay = CompetitionOverlay::new(resources.clone());
        Self {
            resources,
            save_manager,
            scene,
            ui_state: CompetitionUiState::new_with_compact(compact),
            overlay,
            last_event: 0,
            profiles_saved: false,
            _runtime: PhantomData,
        }
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
        self.ensure_scene(state);
        self.scene.as_ref()?;
        let result = self.drive_competition(state);
        command_or_error(&mut self.ui_state, result)
    }

    pub(crate) fn record_acknowledged_human_jump(&mut self, state: &mut GameState) -> bool {
        if !self.ui_state.is_result_acknowledged() || self.ui_state.is_outcome_recorded() {
            return false;
        }
        let outcome = self.scene.as_mut().and_then(|scene| {
            let outcome = scene.outcome()?;
            scene.collect_telemetry();
            Some(outcome)
        });
        let Some(outcome) = outcome else {
            return false;
        };
        let recorded = {
            let mut side_effects = None;
            let recorded = R::runtime_mut(&mut state.active_competition)
                .map(|runtime| {
                    let ctx = runtime.current_jump_context();
                    side_effects = Some(PostJumpSideEffects {
                        profile_idx: runtime.profile_idx_for_context(&ctx),
                        hill_idx: runtime.hill_idx_for_context(&ctx),
                        jumper_name: runtime.jumper_name_for_context(&ctx),
                        saves_hill_records: runtime.saves_hill_records(&ctx),
                        is_computer: self
                            .scene
                            .as_ref()
                            .is_some_and(|scene| scene.participant_is_computer()),
                        is_real_world_cup: runtime.is_real_world_cup_context(&ctx),
                    });
                    runtime.record_jump_runtime(&ctx, outcome);
                    true
                })
                .unwrap_or(false);
            if recorded {
                if let Some(side_effects) = side_effects {
                    apply_post_jump_side_effects(state, &self.resources, &side_effects, outcome);
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
        if let Some(scene) = self.scene.as_mut() {
            scene.update(state);
        }
    }

    pub(crate) fn render_jump(&mut self, cx: &mut PaintCx<'_>, state: &GameState) {
        if let Some(scene) = self.scene.as_mut() {
            render_jump_scene_with_overlay(cx, scene, &self.overlay, &self.ui_state, state);
        }
    }

    pub(crate) fn handle_jump_scene_event(
        &mut self,
        event: UiEvent,
        consume_other_actions: bool,
        accepts_only_enter_escape: bool,
        acknowledge_only_unrecorded: bool,
        state: &GameState,
    ) -> JumpInputResult {
        let Some(scene) = self.scene.as_mut() else {
            return JumpInputResult::None;
        };
        handle_jump_scene_event(
            scene,
            &mut self.ui_state,
            event,
            consume_other_actions,
            accepts_only_enter_escape,
            acknowledge_only_unrecorded,
            state,
        )
    }

    pub(crate) fn prepare_human_jump(
        &mut self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
        team_name: Option<String>,
        state: &mut GameState,
    ) {
        handle_human_jump(
            &mut self.scene,
            &mut self.ui_state,
            &self.resources,
            state,
            participant,
            hill_idx,
            phase_label,
            team_name,
        );
        self.ui_state.enter_jump();
    }

    pub(crate) fn enter_results(&mut self) {
        self.ui_state.enter_results();
    }

    pub(crate) fn enter_done(&mut self) {
        self.ui_state.enter_done();
    }

    pub(crate) fn enter_error(&mut self, msg: impl Into<String>) {
        self.ui_state.enter_error(msg.into());
    }

    fn advance_results(
        &mut self,
        state: &mut GameState,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        self.scene.as_ref()?;
        let result = self.advance_results_and_drive(state);
        command_or_error(&mut self.ui_state, result)
    }

    pub(crate) fn save_results(&mut self, state: &GameState) {
        if self.profiles_saved {
            return;
        }
        persistence::save_profiles_and_records_once(
            &mut self.profiles_saved,
            &self.save_manager,
            state,
        );
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
        self.scene = Some(JumpScene::new(
            self.resources.clone(),
            state,
            0,
            15,
            JumpParticipant::trainee(),
            JumpPolicy::competition(),
        ));
    }

    #[allow(clippy::type_complexity)]
    fn drive_competition(
        &mut self,
        state: &mut GameState,
    ) -> Result<
        Option<CompetitionFlowCommand<R::Context, R::ResultsKind>>,
        CompetitionControllerError,
    > {
        let visible_computers = state.config.visible_computers;
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
                    return Ok(Some(CompetitionFlowCommand::ShowResults(kind)));
                }
                CompetitionDecision::Done => {
                    return Ok(Some(CompetitionFlowCommand::Done));
                }
                CompetitionDecision::Jump {
                    participant,
                    hill_idx,
                    context,
                    is_human,
                    is_new_event,
                } => {
                    if is_human
                        || self.should_show_computer_jump(
                            runtime,
                            &context,
                            &participant,
                            visible_computers,
                        )
                    {
                        let current_event = runtime.event_idx();
                        return Ok(Some(CompetitionFlowCommand::HumanJump {
                            participant,
                            hill_idx,
                            is_new_event: self.check_event_change(current_event, is_new_event),
                            context,
                        }));
                    }

                    let outcome = self
                        .scene
                        .as_ref()
                        .expect("competition scene initialized")
                        .simulate_hidden(participant, hill_idx, rng, wind)?;
                    pending_side_effects.push((
                        PostJumpSideEffects {
                            profile_idx: runtime.profile_idx_for_context(&context),
                            hill_idx: runtime.hill_idx_for_context(&context),
                            jumper_name: runtime.jumper_name_for_context(&context),
                            saves_hill_records: runtime.saves_hill_records(&context),
                            is_computer: true,
                            is_real_world_cup: runtime.is_real_world_cup_context(&context),
                        },
                        outcome,
                    ));
                    runtime.record_jump_runtime(&context, outcome);
                    if runtime.is_complete_runtime() {
                        return Ok(Some(CompetitionFlowCommand::Done));
                    }
                }
            }
        });

        if !pending_side_effects.is_empty() {
            for (side_effects, outcome) in pending_side_effects {
                apply_post_jump_side_effects(state, &self.resources, &side_effects, outcome);
            }
        }

        command.unwrap_or(Ok(None))
    }

    fn advance_results_and_drive(
        &mut self,
        state: &mut GameState,
    ) -> Result<
        Option<CompetitionFlowCommand<R::Context, R::ResultsKind>>,
        CompetitionControllerError,
    > {
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
            1..=234 => participant.ai_id == visible_computers as usize,
            235 => runtime.start_order_pos_for_context(context) < 1,
            236 => runtime.start_order_pos_for_context(context) < 3,
            237 => runtime.start_order_pos_for_context(context) < 5,
            238 => runtime.start_order_pos_for_context(context) < 10,
            239 => true,
            _ => false,
        }
    }

    fn check_event_change(&mut self, current_event: usize, _is_new_event: bool) -> bool {
        let changed = current_event != self.last_event;
        if changed {
            self.last_event = current_event;
        }
        changed
    }
}

#[derive(Clone, Debug)]
struct PostJumpSideEffects {
    profile_idx: Option<usize>,
    hill_idx: usize,
    jumper_name: String,
    saves_hill_records: bool,
    is_computer: bool,
    is_real_world_cup: bool,
}

fn apply_post_jump_side_effects(
    state: &mut GameState,
    resources: &ResourcesRef,
    side_effects: &PostJumpSideEffects,
    outcome: JumpOutcome,
) {
    let distance_tenths = (outcome.distance * 10.0).round().max(0.0) as usize;
    if let Some(profile_idx) = side_effects.profile_idx {
        if let Some(profile) = state.profiles.profiles.get_mut(profile_idx) {
            profile.total_jumps += 1;
            if side_effects.is_real_world_cup && distance_tenths > profile.best_wc_jump {
                profile.best_wc_jump = distance_tenths;
                profile.best_wc_hill_idx = side_effects.hill_idx;
                profile.best_wc_hill_display = hill_display_name(resources, side_effects.hill_idx);
            }
            if distance_tenths > profile.best_jump {
                profile.best_jump = distance_tenths;
                profile.besthill_idx = side_effects.hill_idx;
                profile.best_hill_file = hill_file_name(resources, side_effects.hill_idx);
                profile.best_hill_display = hill_display_name(resources, side_effects.hill_idx);
            }
        }
    }

    let computer_records_enabled =
        !side_effects.is_computer || state.config.computer_hill_records != 0;
    if computer_records_enabled
        && side_effects.saves_hill_records
        && outcome.fall_type == FallType::None
        && state
            .records
            .hill_records
            .get(side_effects.hill_idx)
            .is_some_and(|record| outcome.distance > record.len)
    {
        if let Some(record) = state.records.hill_records.get_mut(side_effects.hill_idx) {
            *record = HillRecord {
                name: side_effects.jumper_name.clone(),
                len: outcome.distance,
                time: current_record_time(),
            };
        }
    }
}

fn hill_file_name(resources: &ResourcesRef, hill_idx: usize) -> String {
    resources
        .hills
        .hill(hill_idx)
        .map_or_else(|| "HILLBASE".to_string(), |hill| hill.terrain_id.clone())
}

fn hill_display_name(resources: &ResourcesRef, hill_idx: usize) -> String {
    resources
        .hills
        .hill(hill_idx)
        .map_or_else(String::new, |hill| hill.name.clone())
}

fn current_record_time() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(|_| String::new(), |duration| duration.as_secs().to_string())
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

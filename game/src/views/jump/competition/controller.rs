use std::cell::Cell;
use std::marker::PhantomData;

use crate::competition::runtime::{CompetitionDecision, CompetitionRuntime};
use crate::jump::types::JumpOutcome;
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::store::{GameState, GameStateRef, HasRuntime, ResourcesRef};
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
    R: CompetitionRuntime + 'static,
    GameState: HasRuntime<R>,
{
    resources: ResourcesRef,
    state: GameStateRef,
    scene: Option<JumpScene>,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    last_event: Cell<usize>,
    profiles_saved: Cell<bool>,
    _runtime: PhantomData<R>,
}

impl<R> CompetitionJumpController<R>
where
    R: CompetitionRuntime + 'static,
    GameState: HasRuntime<R>,
{
    pub(crate) fn new(
        resources: ResourcesRef,
        state: GameStateRef,
        scene: Option<JumpScene>,
    ) -> Self {
        Self {
            resources: resources.clone(),
            state: state.clone(),
            scene,
            ui_state: CompetitionUiState::new(),
            overlay: CompetitionOverlay::new(resources.clone(), state.clone()),
            last_event: Cell::new(0),
            profiles_saved: Cell::new(false),
            _runtime: PhantomData,
        }
    }

    pub(crate) fn resources(&self) -> &ResourcesRef {
        &self.resources
    }

    pub(crate) fn state(&self) -> &GameStateRef {
        &self.state
    }

    pub(crate) fn ui_state(&self) -> &CompetitionUiState {
        &self.ui_state
    }

    pub(crate) fn scene(&self) -> Option<&JumpScene> {
        self.scene.as_ref()
    }

    pub(crate) fn render_mode(&self) -> RenderMode {
        self.ui_state.render_mode()
    }

    pub(crate) fn drive(&mut self) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        self.ensure_scene();
        let scene = self.scene.as_ref()?;
        command_or_error(&self.ui_state, self.drive_competition(scene))
    }

    pub(crate) fn record_acknowledged_human_jump(&self) -> bool {
        if !self.ui_state.is_result_acknowledged() || self.ui_state.is_outcome_recorded() {
            return false;
        }
        let Some(scene) = self.scene.as_ref() else {
            return false;
        };
        if !self.record_finished_human_jump(scene) {
            return false;
        }
        self.ui_state.mark_outcome_recorded();
        true
    }

    pub(crate) fn update_scene(&mut self) {
        if let Some(scene) = self.scene.as_mut() {
            scene.update();
        }
    }

    pub(crate) fn render_jump(&self, cx: &mut PaintCx<'_>) {
        if let Some(scene) = self.scene.as_ref() {
            render_jump_scene_with_overlay(cx, scene, &self.overlay, &self.ui_state);
        }
    }

    pub(crate) fn handle_jump_scene_event(
        &self,
        event: UiEvent,
        consume_other_actions: bool,
        accepts_only_enter_escape: bool,
        acknowledge_only_unrecorded: bool,
    ) -> JumpInputResult {
        let Some(scene) = self.scene.as_ref() else {
            return JumpInputResult::None;
        };
        handle_jump_scene_event(
            scene,
            &self.ui_state,
            event,
            consume_other_actions,
            accepts_only_enter_escape,
            acknowledge_only_unrecorded,
        )
    }

    pub(crate) fn prepare_human_jump(
        &mut self,
        participant: JumpParticipant,
        hill_idx: usize,
        phase_label: String,
        team_name: Option<String>,
    ) {
        handle_human_jump(
            &mut self.scene,
            &self.ui_state,
            &self.resources,
            &self.state,
            participant,
            hill_idx,
            phase_label,
            team_name,
        );
        self.ui_state.enter_jump();
    }

    pub(crate) fn enter_results(&self) {
        self.ui_state.enter_results();
    }

    pub(crate) fn enter_done(&self) {
        self.ui_state.enter_done();
    }

    pub(crate) fn enter_error(&self, msg: impl Into<String>) {
        self.ui_state.enter_error(msg.into());
    }

    fn advance_results(&mut self) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        let scene = self.scene.as_ref()?;
        command_or_error(&self.ui_state, self.advance_results_and_drive(scene))
    }

    pub(crate) fn save_results(&self) {
        if self.profiles_saved.get() {
            return;
        }
        persistence::save_profiles_and_records_once(
            &self.profiles_saved,
            &self.resources,
            &self.state,
        );
    }

    pub(crate) fn dismiss_results_and_advance(
        &mut self,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        self.ui_state.dismiss_results();
        self.advance_results()
    }

    fn ensure_scene(&mut self) {
        if self.scene.is_some() {
            return;
        }
        self.scene = Some(JumpScene::new(
            self.resources.clone(),
            self.state.clone(),
            0,
            15,
            JumpParticipant::trainee(),
            JumpPolicy::competition(),
        ));
    }

    fn record_finished_human_jump(&self, scene: &JumpScene) -> bool {
        let outcome = match scene.outcome() {
            Some(outcome) => outcome,
            None => return false,
        };
        scene.collect_telemetry();
        self.state
            .borrow_mut()
            .with_runtime_mut(|runtime: &mut R| {
                if !runtime.is_human_current() {
                    return false;
                }
                let ctx = runtime.current_jump_context();
                runtime.record_jump_runtime(&ctx, outcome);
                true
            })
            .unwrap_or(false)
    }

    #[allow(clippy::type_complexity)]
    fn drive_competition(
        &self,
        scene: &JumpScene,
    ) -> Result<
        Option<CompetitionFlowCommand<R::Context, R::ResultsKind>>,
        CompetitionControllerError,
    > {
        let command = self
            .state
            .borrow_mut()
            .with_runtime_mut(|runtime: &mut R| loop {
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
                        if is_human {
                            let current_event = runtime.event_idx();
                            return Ok(Some(CompetitionFlowCommand::HumanJump {
                                participant,
                                hill_idx,
                                is_new_event: self.check_event_change(current_event, is_new_event),
                                context,
                            }));
                        }

                        let outcome = self.simulate_computer(scene, participant, hill_idx)?;
                        runtime.record_jump_runtime(&context, outcome);
                        if runtime.is_complete_runtime() {
                            return Ok(Some(CompetitionFlowCommand::Done));
                        }
                    }
                }
            });

        command.unwrap_or(Ok(None))
    }

    fn advance_results_and_drive(
        &self,
        scene: &JumpScene,
    ) -> Result<
        Option<CompetitionFlowCommand<R::Context, R::ResultsKind>>,
        CompetitionControllerError,
    > {
        self.state.borrow_mut().with_runtime_mut(|runtime: &mut R| {
            runtime.advance_results_runtime();
        });
        self.drive_competition(scene)
    }

    fn simulate_computer(
        &self,
        scene: &JumpScene,
        participant: JumpParticipant,
        hill_idx: usize,
    ) -> Result<JumpOutcome, CompetitionControllerError> {
        Ok(scene.simulate_hidden(participant, hill_idx)?)
    }

    fn check_event_change(&self, current_event: usize, _is_new_event: bool) -> bool {
        let changed = current_event != self.last_event.get();
        if changed {
            self.last_event.set(current_event);
        }
        changed
    }
}

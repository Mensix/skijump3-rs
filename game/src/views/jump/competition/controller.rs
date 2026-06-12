use std::marker::PhantomData;

use crate::competition::runtime::CompetitionRuntime;
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::store::{HasRuntime, ResourcesRef, Store, StoreRef};
use crate::views::jump::competition::flow::{
    command_or_error, handle_human_jump, handle_jump_scene_event, record_acknowledged_human_jump,
    render_jump_scene_with_overlay, CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::competition::overlay::CompetitionOverlay;
use crate::views::jump::competition::session::CompetitionSession;
use crate::views::jump::competition::ui_state::{CompetitionUiState, RenderMode};
use crate::views::jump::scene::JumpScene;
use engine::oxide::PaintCx;
use engine::ui::Event;

pub(crate) struct CompetitionJumpController<R>
where
    R: CompetitionRuntime + 'static,
    Store: HasRuntime<R>,
{
    resources: ResourcesRef,
    store: StoreRef,
    scene: Option<JumpScene>,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    session: CompetitionSession,
    _runtime: PhantomData<R>,
}

impl<R> CompetitionJumpController<R>
where
    R: CompetitionRuntime + 'static,
    Store: HasRuntime<R>,
{
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef, scene: Option<JumpScene>) -> Self {
        Self {
            resources: ResourcesRef::clone(&resources),
            store: StoreRef::clone(&store),
            scene,
            ui_state: CompetitionUiState::new(),
            overlay: CompetitionOverlay::new(
                ResourcesRef::clone(&resources),
                StoreRef::clone(&store),
            ),
            session: CompetitionSession::new(resources, store),
            _runtime: PhantomData,
        }
    }

    pub(crate) fn resources(&self) -> &ResourcesRef {
        &self.resources
    }

    pub(crate) fn store(&self) -> &StoreRef {
        &self.store
    }

    pub(crate) fn ui_state(&self) -> &CompetitionUiState {
        &self.ui_state
    }

    pub(crate) fn session(&self) -> &CompetitionSession {
        &self.session
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
        command_or_error(&self.ui_state, self.session.drive_competition::<R>(scene))
    }

    pub(crate) fn record_acknowledged_human_jump(&self) -> bool {
        record_acknowledged_human_jump::<R>(&self.session, &self.ui_state, self.scene.as_ref())
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
        event: Event,
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
            &self.store,
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

    fn advance_results(
        &mut self,
        kind: R::ResultsKind,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        let scene = self.scene.as_ref()?;
        command_or_error(
            &self.ui_state,
            self.session.advance_results_and_drive::<R>(scene, kind),
        )
    }

    pub(crate) fn dismiss_results_and_advance(
        &mut self,
        kind: R::ResultsKind,
    ) -> Option<CompetitionFlowCommand<R::Context, R::ResultsKind>> {
        self.ui_state.dismiss_results();
        self.advance_results(kind)
    }

    fn ensure_scene(&mut self) {
        if self.scene.is_some() {
            return;
        }
        self.scene = Some(JumpScene::new(
            ResourcesRef::clone(&self.resources),
            StoreRef::clone(&self.store),
            0,
            15,
            JumpParticipant::trainee(),
            JumpPolicy::competition(),
        ));
    }
}

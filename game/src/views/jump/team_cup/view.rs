use super::setup::{SetupAction, TeamCupSetup};
use crate::competition::team_cup::types::{TeamCupJumpContext, TeamCupResultsKind, TeamCupRuntime};
use crate::components::screen;
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::flow::{
    command_or_error, handle_human_jump, handle_jump_scene_event, record_acknowledged_human_jump,
    render_jump_scene_with_overlay, route_error_back, CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::competition::overlay::CompetitionOverlay;
use crate::views::jump::competition::results::{
    self as competition_results, CompetitionResultsRequest,
};
use crate::views::jump::competition::session::CompetitionSession;
use crate::views::jump::competition::ui_state::{CompetitionUiState, RenderMode};
use crate::views::jump::scene::JumpScene;
use engine::ui::{Blinker, Element, Event, View};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewPhase {
    Setup,
    Jumping,
    Done,
}

pub struct TeamCupJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: Option<JumpScene>,
    ui_state: CompetitionUiState,
    overlay: CompetitionOverlay,
    phase: ViewPhase,
    blinker: Blinker,
    session: CompetitionSession,
    setup: TeamCupSetup,
    cursor_visible: bool,
    results_kind: TeamCupResultsKind,
}

impl TeamCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let setup = TeamCupSetup::new(&store);
        Self {
            resources: resources.clone(),
            store: store.clone(),
            scene: None,
            ui_state: CompetitionUiState::new(),
            overlay: CompetitionOverlay::new(resources.clone(), store.clone()),
            phase: ViewPhase::Setup,
            blinker: Blinker::new(),
            session: CompetitionSession::new(resources, store),
            setup,
            cursor_visible: true,
            results_kind: TeamCupResultsKind::LegResults,
        }
    }

    fn drive_until_visible(&mut self) {
        let scene = JumpScene::new(
            ResourcesRef::clone(&self.resources),
            StoreRef::clone(&self.store),
            0,
            15,
            JumpParticipant::trainee(),
            JumpPolicy::competition(),
        );
        if let Some(cmd) = command_or_error(
            &self.ui_state,
            self.session.drive_competition::<TeamCupRuntime>(&scene),
        ) {
            self.apply_command(cmd);
        } else if self.ui_state.render_mode() != RenderMode::Error {
            self.ui_state.enter_error("No competition running".into());
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<TeamCupJumpContext, TeamCupResultsKind>,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event: _,
            } => {
                let phase_label = if context.round_idx == 0 {
                    self.resources.langbase.lstr(54).to_string()
                } else {
                    self.resources.langbase.lstr(55).to_string()
                };
                handle_human_jump(
                    &mut self.scene,
                    &self.ui_state,
                    &self.resources,
                    &self.store,
                    participant,
                    hill_idx,
                    phase_label,
                    Some(context.team_name.clone()),
                );
                self.ui_state.enter_jump();
            }
            CompetitionFlowCommand::ShowResults(kind) => {
                self.results_kind = kind;
                self.ui_state.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.phase = ViewPhase::Done;
                self.ui_state.enter_done();
            }
        }
    }
}

impl View<RouteTarget> for TeamCupJumpView {
    fn update(&mut self) {
        self.cursor_visible = self.blinker.visible(10, 10);

        if self.phase == ViewPhase::Setup {
            return;
        }
        if self.phase != ViewPhase::Jumping {
            return;
        }

        record_acknowledged_human_jump::<TeamCupRuntime>(
            &self.session,
            &self.ui_state,
            self.scene.as_ref(),
        );

        // Drive competition only after human jump outcome is recorded,
        // not every frame during the jump (avoids recreating the scene).
        if self.ui_state.is_outcome_recorded() && self.ui_state.render_mode() != RenderMode::Results
        {
            if let Some(ref scene) = self.scene {
                if let Some(cmd) = command_or_error(
                    &self.ui_state,
                    self.session.drive_competition::<TeamCupRuntime>(scene),
                ) {
                    self.apply_command(cmd);
                }
            }
        }

        if let Some(ref mut scene) = self.scene {
            scene.update();
        }
    }

    fn elements(&self) -> Vec<Element> {
        if self.phase == ViewPhase::Setup {
            return self
                .setup
                .elements(&self.resources, &self.store, self.cursor_visible);
        }
        if self.phase == ViewPhase::Done {
            return vec![];
        }

        match self.ui_state.render_mode() {
            RenderMode::Jump => {
                if let Some(ref scene) = self.scene {
                    render_jump_scene_with_overlay(scene, &self.overlay, &self.ui_state)
                } else {
                    vec![]
                }
            }
            RenderMode::Results => competition_results::render(
                &self.resources,
                &self.store,
                &self.ui_state,
                CompetitionResultsRequest::TeamCup {
                    kind: self.results_kind,
                },
            ),
            RenderMode::Done | RenderMode::Error => {
                let msg = if self.ui_state.render_mode() == RenderMode::Error {
                    self.ui_state.error_message()
                } else {
                    String::new()
                };
                screen::message_screen(&msg, self.resources.langbase.lstr(15))
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if let Some(route) = route_error_back(&self.ui_state, event) {
            return Some(route);
        }
        if self.ui_state.render_mode() == RenderMode::Error {
            return None;
        }

        if self.phase == ViewPhase::Done {
            return Some(RouteTarget::Back);
        }

        if self.phase == ViewPhase::Setup {
            if self.setup.handle_event(&self.resources, &self.store, event)
                == SetupAction::StartJumping
            {
                self.phase = ViewPhase::Jumping;
                self.drive_until_visible();
            }
            return None;
        }

        if let Some(ref scene) = self.scene {
            // Let the shared input controller process events first
            if self.phase == ViewPhase::Jumping {
                match handle_jump_scene_event(scene, &self.ui_state, event, false, false, true) {
                    JumpInputResult::Route(route) => return Some(route),
                    JumpInputResult::Consumed => return None,
                    JumpInputResult::None => {}
                }
            }
        }

        if self.ui_state.render_mode() == RenderMode::Results {
            if matches!(event, Event::Keyboard(_)) {
                self.ui_state.dismiss_results();
                if let Some(ref scene) = self.scene {
                    if let Some(cmd) = command_or_error(
                        &self.ui_state,
                        self.session.advance_results_and_drive::<TeamCupRuntime>(
                            scene,
                            TeamCupResultsKind::LegResults,
                        ),
                    ) {
                        self.apply_command(cmd);
                    }
                }
            }
            return None;
        }

        None
    }
}

use super::setup::{SetupAction, TeamCupSetup};
use crate::competition::team_cup::types::{TeamCupJumpContext, TeamCupResultsKind, TeamCupRuntime};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{CompetitionFlowCommand, JumpInputResult};
use crate::views::jump::competition::ui_state::RenderMode;
use crate::views::jump::team_cup::results as team_cup_results;
use engine::oxide::Blinker;
use engine::oxide::{PaintCx, ScreenEventCx, UiEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewPhase {
    Setup,
    Jumping,
    Done,
}

pub struct TeamCupJumpView {
    controller: CompetitionJumpController<TeamCupRuntime>,
    phase: ViewPhase,
    blinker: Blinker,
    setup: TeamCupSetup,
    cursor_visible: bool,
    results_kind: TeamCupResultsKind,
}

impl TeamCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, save_manager: SaveRef) -> Self {
        Self {
            controller: CompetitionJumpController::new(resources, save_manager, false, None),
            phase: ViewPhase::Setup,
            blinker: Blinker::new(),
            setup: TeamCupSetup::new_with_dummy(),
            cursor_visible: true,
            results_kind: TeamCupResultsKind::LegResults,
        }
    }

    fn init_setup(&mut self, state: &GameState) {
        self.setup = TeamCupSetup::new(state);
    }

    fn drive_until_visible(&mut self, state: &mut GameState) {
        if let Some(cmd) = self.controller.drive(state) {
            self.apply_command(cmd, state);
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<TeamCupJumpContext, TeamCupResultsKind>,
        state: &mut GameState,
    ) {
        let lang = &self.controller.resources().langbase;
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event,
            } => {
                if is_new_event {
                    state.setup_jump_event();
                }
                let phase_label = if context.round_idx == 0 {
                    lang.tr(54).to_string()
                } else {
                    lang.tr(55).to_string()
                };
                self.controller.prepare_human_jump(
                    participant,
                    hill_idx,
                    phase_label,
                    Some(context.team_name),
                    state,
                );
            }
            CompetitionFlowCommand::ShowResults(kind) => {
                self.results_kind = kind;
                self.controller.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.phase = ViewPhase::Done;
                self.controller.enter_done();
            }
        }
    }

    fn paint_content(&mut self, cx: &mut PaintCx<'_>, state: &GameState) {
        if self.phase == ViewPhase::Setup {
            self.setup
                .paint(cx, self.controller.resources(), state, self.cursor_visible);
            return;
        }
        if self.phase == ViewPhase::Done {
            return;
        }

        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx, state);
            }
            RenderMode::Results => {
                team_cup_results::render(cx, self.controller.resources(), state, self.results_kind);
            }
            RenderMode::Done => {}
        }
    }

    fn handle_input(&mut self, event: UiEvent, state: &mut GameState) -> Option<RouteTarget> {
        if self.phase == ViewPhase::Done {
            return Some(RouteTarget::Back);
        }

        if self.phase == ViewPhase::Setup {
            if self
                .setup
                .handle_event(self.controller.resources(), state, event)
                == SetupAction::StartJumping
            {
                self.phase = ViewPhase::Jumping;
                self.drive_until_visible(state);
            }
            return None;
        }

        if self.phase == ViewPhase::Jumping {
            match self
                .controller
                .handle_jump_scene_event(event, false, false, true, state)
            {
                JumpInputResult::Route(route) => return Some(route),
                JumpInputResult::Consumed => return None,
                JumpInputResult::None => {}
            }
        }

        if self.controller.render_mode() == RenderMode::Results {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                if let Some(cmd) = self.controller.dismiss_results_and_advance(state) {
                    self.apply_command(cmd, state);
                }
            }
            return None;
        }

        None
    }
}

impl GameScreen for TeamCupJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        self.cursor_visible = self.blinker.visible(10, 10);

        if self.phase == ViewPhase::Setup {
            if !self.setup.has_teams() {
                self.init_setup(cx.state);
            }
            return;
        }
        if self.phase != ViewPhase::Jumping {
            return;
        }

        self.controller.record_acknowledged_human_jump(cx.state);

        if self.controller.ui_state().is_outcome_recorded()
            && self.controller.render_mode() != RenderMode::Results
        {
            if let Some(cmd) = self.controller.drive(cx.state) {
                self.apply_command(cmd, cx.state);
            }
        }

        self.controller.update_scene(cx.state);
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let Some(route) = self.handle_input(event, cx.state) {
            if route == RouteTarget::Back {
                nav.back();
            } else {
                nav.navigate(route);
            }
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(paint, cx.state);
    }
}

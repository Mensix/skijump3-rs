use super::setup::{SetupAction, TeamCupSetup};
use crate::competition::team_cup::types::{TeamCupJumpContext, TeamCupResultsKind, TeamCupRuntime};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{
    route_error_back, CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::team_cup::results as team_cup_results;
use crate::gfx::palette::{BLACK, FONT_DEFAULT, FONT_HELP};
use crate::views::jump::competition::ui_state::RenderMode;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::oxide::Blinker;

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
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let setup = TeamCupSetup::new(&store);
        Self {
            controller: CompetitionJumpController::new(resources, store, None),
            phase: ViewPhase::Setup,
            blinker: Blinker::new(),
            setup,
            cursor_visible: true,
            results_kind: TeamCupResultsKind::LegResults,
        }
    }

    fn drive_until_visible(&mut self) {
        if let Some(cmd) = self.controller.drive() {
            self.apply_command(cmd);
        } else if self.controller.render_mode() != RenderMode::Error {
            self.controller.enter_error("No competition running");
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
                is_new_event,
            } => {
                if is_new_event {
                    self.controller.store().setup_jump_event();
                }
                let phase_label = if context.round_idx == 0 {
                    self.controller.resources().langbase.lstr(54).to_string()
                } else {
                    self.controller.resources().langbase.lstr(55).to_string()
                };
                self.controller.prepare_human_jump(
                    participant,
                    hill_idx,
                    phase_label,
                    Some(context.team_name),
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

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        if self.phase == ViewPhase::Setup {
            self.setup.paint(
                cx,
                self.controller.resources(),
                self.controller.store(),
                self.cursor_visible,
            );
            return;
        }
        if self.phase == ViewPhase::Done {
            return;
        }

        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx);
            }
            RenderMode::Results => {
                team_cup_results::render(
                    cx,
                    self.controller.resources(),
                    self.controller.store(),
                    self.results_kind,
                );
            }
            RenderMode::Done | RenderMode::Error => {
                let msg = if self.controller.render_mode() == RenderMode::Error {
                    self.controller.ui_state().error_message()
                } else {
                    String::new()
                };
                cx.fill((0, 0, 320, 200), BLACK);
                cx.text((20, 80), FONT_DEFAULT, &msg);
                cx.text((20, 95), FONT_HELP, self.controller.resources().langbase.lstr(15));
            }
        }
    }

    fn handle_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        if let Some(route) = route_error_back(self.controller.ui_state(), event) {
            return Some(route);
        }
        if self.controller.render_mode() == RenderMode::Error {
            return None;
        }

        if self.phase == ViewPhase::Done {
            return Some(RouteTarget::Back);
        }

        if self.phase == ViewPhase::Setup {
            if self
                .setup
                .handle_event(self.controller.resources(), self.controller.store(), event)
                == SetupAction::StartJumping
            {
                self.phase = ViewPhase::Jumping;
                self.drive_until_visible();
            }
            return None;
        }

        // Let the shared input controller process events first
        if self.phase == ViewPhase::Jumping {
            match self
                .controller
                .handle_jump_scene_event(event, false, false, true)
            {
                JumpInputResult::Route(route) => return Some(route),
                JumpInputResult::Consumed => return None,
                JumpInputResult::None => {}
            }
        }

        if self.controller.render_mode() == RenderMode::Results {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                if let Some(cmd) = self
                    .controller
                    .dismiss_results_and_advance()
                {
                    self.apply_command(cmd);
                }
            }
            return None;
        }

        None
    }
}

impl Screen<RouteTarget> for TeamCupJumpView {
    fn update(&mut self) {
        self.cursor_visible = self.blinker.visible(10, 10);

        if self.phase == ViewPhase::Setup {
            return;
        }
        if self.phase != ViewPhase::Jumping {
            return;
        }

        self.controller.record_acknowledged_human_jump();

        // Drive competition only after human jump outcome is recorded,
        // not every frame during the jump (avoids recreating the scene).
        if self.controller.ui_state().is_outcome_recorded()
            && self.controller.render_mode() != RenderMode::Results
        {
            if let Some(cmd) = self.controller.drive() {
                self.apply_command(cmd);
            }
        }

        self.controller.update_scene();
    }

    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let Some(route) = self.handle_input(event) {
            if route == RouteTarget::Back {
                cx.back();
            } else {
                cx.navigate(route);
            }
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}



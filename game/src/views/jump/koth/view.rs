use crate::competition::koth::types::{KothJumpContext, KothResultsKind, KothRuntime};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{
    route_error_back, CompetitionFlowCommand, JumpInputResult,
};

use crate::views::jump::competition::ui_state::RenderMode;
use engine::oxide::{Key, PaintCx, Screen, ScreenEventCx, UiEvent};

pub struct KothJumpView {
    controller: CompetitionJumpController<KothRuntime>,
}

impl KothJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self {
            controller: CompetitionJumpController::new(resources, store, None),
        }
    }

    fn apply_command(&mut self, command: CompetitionFlowCommand<KothJumpContext, KothResultsKind>) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event: _,
            } => {
                let phase_label = format!(
                    "Round {} (elim {})",
                    context.jump_round + 1,
                    context.elimination_round + 1,
                );
                self.controller
                    .prepare_human_jump(participant, hill_idx, phase_label, None);
            }
            CompetitionFlowCommand::ShowResults(KothResultsKind::Results) => {
                self.controller.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.on_complete();
                self.controller.enter_done();
            }
        }
    }

    fn on_complete(&self) {
        self.controller.session().save_results();
        // TODO: KOTH records update (top[35+pack], profile.koth_level)
    }

    fn is_result_display_state(&self) -> bool {
        self.controller.render_mode() == RenderMode::Results
            || self.controller.ui_state().has_page()
    }

    fn dismiss_results_and_advance(&mut self) {
        self.controller
            .dismiss_results_and_advance(KothResultsKind::Results);
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx);
            }
            RenderMode::Results => {
                crate::views::jump::koth::results::render(
                    cx,
                    self.controller.resources(),
                    self.controller.store(),
                );
            }
            RenderMode::Done | RenderMode::Error => {
                let msg = if self.controller.render_mode() == RenderMode::Error {
                    self.controller.ui_state().error_message()
                } else {
                    String::new()
                };
                cx.fill((0, 0, 320, 200), crate::gfx::palette::BLACK);
                cx.text((20, 80), crate::gfx::palette::FONT_DEFAULT, &msg);
                cx.text((20, 95), crate::gfx::palette::FONT_HELP, self.controller.resources().langbase.lstr(15));
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

        if self.controller.render_mode() == RenderMode::Done {
            // KOTH complete, any key returns
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                return Some(RouteTarget::Back);
            }
            return None;
        }

        if self.is_result_display_state() {
            if matches!(
                event,
                UiEvent::KeyDown(Key::Right | Key::Enter | Key::Escape) | UiEvent::Text(' ')
            ) {
                self.dismiss_results_and_advance();
            }
            return None;
        }

        // Jump scene input
        match self
            .controller
            .handle_jump_scene_event(event, false, false, true)
        {
            JumpInputResult::Route(route) => return Some(route),
            JumpInputResult::Consumed => return None,
            JumpInputResult::None => {}
        }

        None
    }
}

impl Screen<RouteTarget> for KothJumpView {
    fn update(&mut self) {
        self.controller.record_acknowledged_human_jump();

        if let Some(command) = self.controller.drive() {
            self.apply_command(command);
        }

        if self.controller.render_mode() == RenderMode::Jump {
            self.controller.update_scene();
        }
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



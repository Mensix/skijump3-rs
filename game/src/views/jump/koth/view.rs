use crate::competition::koth::types::{KothJumpContext, KothResultsKind, KothRuntime};
use crate::components::screen;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{
    route_error_back, CompetitionFlowCommand, JumpInputResult,
};
use crate::views::jump::competition::results::{
    self as competition_results, CompetitionResultsRequest,
};
use crate::views::jump::competition::ui_state::RenderMode;
use engine::ui::{Blinker, Element, Event, Key, View};

pub struct KothJumpView {
    controller: CompetitionJumpController<KothRuntime>,
    blinker: Blinker,
}

impl KothJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        Self {
            controller: CompetitionJumpController::new(resources, store, None),
            blinker: Blinker::new(),
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<KothJumpContext, KothResultsKind>,
    ) {
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
        self.controller.dismiss_results_and_advance(KothResultsKind::Results);
    }
}

impl View<RouteTarget> for KothJumpView {
    fn update(&mut self) {
        self.controller.record_acknowledged_human_jump();

        if let Some(command) = self.controller.drive() {
            self.apply_command(command);
        }

        if self.controller.render_mode() == RenderMode::Jump {
            self.controller.update_scene();
        }
    }

    fn elements(&self) -> Vec<Element> {
        match self.controller.render_mode() {
            RenderMode::Jump => self.controller.render_jump_elements(),
            RenderMode::Results => competition_results::render(
                self.controller.resources(),
                self.controller.store(),
                self.controller.ui_state(),
                CompetitionResultsRequest::Koth,
            ),
            RenderMode::Done | RenderMode::Error => {
                let msg = if self.controller.render_mode() == RenderMode::Error {
                    self.controller.ui_state().error_message()
                } else {
                    String::new()
                };
                screen::message_screen(&msg, self.controller.resources().langbase.lstr(15))
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if let Some(route) = route_error_back(self.controller.ui_state(), event) {
            return Some(route);
        }
        if self.controller.render_mode() == RenderMode::Error {
            return None;
        }

        if self.controller.render_mode() == RenderMode::Done {
            // KOTH complete, any key returns
            if matches!(event, Event::Keyboard(_)) {
                return Some(RouteTarget::Back);
            }
            return None;
        }

        if self.is_result_display_state() {
            if matches!(event, Event::Keyboard(Key::Right | Key::Char(' ') | Key::Enter | Key::Escape)) {
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

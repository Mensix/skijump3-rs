use crate::competition::koth::types::{KothJumpContext, KothResultsKind, KothRuntime};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::current_timestamp;
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{CompetitionFlowCommand, JumpInputResult};
use crate::views::jump::competition::ui_state::RenderMode;
use crate::views::jump::koth::results;
use engine::oxide::{Key, PaintCx, ScreenEventCx, UiEvent};

pub struct KothJumpView {
    controller: CompetitionJumpController<KothRuntime>,
    completion_saved: bool,
}

impl KothJumpView {
    pub(crate) fn new(resources: ResourcesRef, save_manager: SaveRef) -> Self {
        Self {
            controller: CompetitionJumpController::new(resources, save_manager, false, None),
            completion_saved: false,
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<KothJumpContext, KothResultsKind>,
        state: &mut GameState,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
                is_new_event: _,
            } => {
                let phase_label = format!("Round {}", context.jump_round + 1,);
                self.controller
                    .prepare_human_jump(participant, hill_idx, phase_label, None, state);
            }
            CompetitionFlowCommand::ShowResults(KothResultsKind::Results) => {
                self.controller.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.on_complete(state);
                self.controller.enter_done();
            }
        }
    }

    fn on_complete(&mut self, state: &mut GameState) {
        self.update_koth_completion_records(state);
        self.controller.save_results(state);
    }

    fn update_koth_completion_records(&mut self, state: &mut GameState) {
        if self.completion_saved {
            return;
        }
        self.completion_saved = true;

        let pack = state.config.koth_pack;
        if !(1..=6).contains(&pack) {
            return;
        }

        let Some((winner_name, winner_profile_idx)) = state
            .active_competition
            .as_ref()
            .and_then(|comp| comp.koth_runtime())
            .and_then(koth_winner)
        else {
            return;
        };

        if let Some(record) = state.records.top.get_mut(34 + pack as usize) {
            record.score += 1.0;
            record.name = winner_name;
            record.time = current_timestamp();
            record.is_computer = false;
        }

        if let Some(profile) = state.profiles.profiles.get_mut(winner_profile_idx) {
            let pack = pack as usize;
            if profile.koth_level == 0 || pack < profile.koth_level {
                profile.koth_level = pack;
            }
        }
    }

    fn is_result_display_state(&self) -> bool {
        self.controller.render_mode() == RenderMode::Results
            || self.controller.ui_state().has_page()
    }

    fn dismiss_results_and_advance(&mut self, state: &mut GameState) -> Option<RouteTarget> {
        if let Some(command) = self.controller.dismiss_results_and_advance(state) {
            if matches!(command, CompetitionFlowCommand::Done) {
                self.on_complete(state);
                return Some(RouteTarget::Back);
            }
            self.apply_command(command, state);
        }
        None
    }

    fn paint_content(&mut self, cx: &mut PaintCx<'_>, state: &GameState) {
        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx, state);
            }
            RenderMode::Results => {
                results::render(cx, self.controller.resources(), state);
            }
            RenderMode::Done => {
                results::render(cx, self.controller.resources(), state);
            }
        }
    }

    fn handle_input(&mut self, event: UiEvent, state: &mut GameState) -> Option<RouteTarget> {
        if self.controller.render_mode() == RenderMode::Done {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                return Some(RouteTarget::Back);
            }
            return None;
        }

        if self.is_result_display_state() {
            if matches!(
                event,
                UiEvent::KeyDown(Key::Right | Key::Enter) | UiEvent::Text(' ')
            ) {
                if let Some(route) = self.dismiss_results_and_advance(state) {
                    return Some(route);
                }
            }
            return None;
        }

        match self
            .controller
            .handle_jump_scene_event(event, false, false, true, state)
        {
            JumpInputResult::Consumed => return None,
            JumpInputResult::None => {}
        }

        None
    }
}

fn koth_winner(runtime: &KothRuntime) -> Option<(String, usize)> {
    let mut alive = runtime
        .participants
        .iter()
        .filter(|participant| participant.is_alive());
    let winner = alive.next()?;
    if alive.next().is_some() {
        return None;
    }
    let profile_idx = winner.competitor.profile_idx?;
    Some((winner.competitor.name.clone(), profile_idx))
}

impl GameScreen for KothJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        self.controller.record_acknowledged_human_jump(cx.state);

        if !matches!(
            self.controller.render_mode(),
            RenderMode::Results | RenderMode::Done
        ) {
            if let Some(command) = self.controller.drive(cx.state) {
                self.apply_command(command, cx.state);
            }
        }

        if self.controller.render_mode() == RenderMode::Jump {
            self.controller.update_scene(cx.state);
        }
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

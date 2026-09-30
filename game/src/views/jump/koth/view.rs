use crate::competition::completion::CompletionSaveGuard;
use crate::competition::koth::types::{KothJumpContext, KothResultsKind, KothRuntime};
use crate::components::page_nav::{is_quit_event, page_event, PageDismissal, PageEventMap};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen, NavSignal, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::current_timestamp;
use crate::ui::UiCanvas;
use crate::ui::{Key, ScreenEventCx, UiEvent};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{CompetitionFlowCommand, JumpInputResult};
use crate::views::jump::competition::ui_state::CompetitionUiState;
use crate::views::jump::competition::ui_state::RenderMode;
use crate::views::jump::koth::results;
use crate::views::records::RecordNotification;

pub struct KothJumpView {
    controller: CompetitionJumpController<KothRuntime>,
    completion_saved: CompletionSaveGuard,
    record_notification: Option<RecordNotification>,
}

impl KothJumpView {
    pub(crate) fn new(resources: ResourcesRef) -> Self {
        Self {
            controller: CompetitionJumpController::new(resources, false, None),
            completion_saved: CompletionSaveGuard::default(),
            record_notification: None,
        }
    }

    fn apply_command(
        &mut self,
        command: CompetitionFlowCommand<KothJumpContext, KothResultsKind>,
        state: &mut GameState,
        cx: &Persistence,
    ) {
        match command {
            CompetitionFlowCommand::HumanJump {
                participant,
                hill_idx,
                context,
            } => {
                let phase_label = format!(
                    "Round {} (elim {})",
                    context.jump_round + 1,
                    context.elimination_round + 1
                );
                self.controller.prepare_human_jump(
                    participant,
                    hill_idx,
                    phase_label,
                    None,
                    false,
                    state,
                );
            }
            CompetitionFlowCommand::ShowResults(KothResultsKind::Results) => {
                self.controller.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.on_complete(state, cx);
                self.controller.enter_done();
            }
        }
    }

    fn on_complete(&mut self, state: &mut GameState, cx: &Persistence) {
        self.update_koth_completion_records(state);
        self.controller.save_results(state, cx);
    }

    fn update_koth_completion_records(&mut self, state: &mut GameState) {
        if !self.completion_saved.claim() {
            return;
        }

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

        let Some(old_record) = state.records.top.get_mut(34 + pack as usize).map(|record| {
            let old_record = (record.score > 0.0).then(|| record.clone());
            record.score += 1.0;
            record.name = winner_name;
            record.time = current_timestamp();
            record.is_computer = false;
            old_record
        }) else {
            return;
        };
        let entries = (1..=6)
            .filter_map(|record_pack| {
                state.records.top.get(34 + record_pack).map(|record| {
                    let mut display_record = record.clone();
                    display_record.pos = record_pack;
                    display_record
                })
            })
            .collect();
        self.record_notification = Some(RecordNotification::new(
            "NEW KING OF THE HILL RECORD!",
            entries,
            old_record,
        ));

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

    fn dismiss_results_and_advance(
        &mut self,
        state: &mut GameState,
        cx: &Persistence,
    ) -> NavSignal {
        if let Some(command) = self.controller.dismiss_results_and_advance(state) {
            if matches!(command, CompetitionFlowCommand::Done) {
                self.on_complete(state, cx);
                return if self.record_notification.is_some() {
                    NavSignal::None
                } else {
                    NavSignal::Back
                };
            }
            self.apply_command(command, state, cx);
        }
        NavSignal::None
    }

    fn paint_content(&mut self, cx: &mut dyn UiCanvas, state: &GameState) {
        if self.controller.paint_alerts(&self.record_notification, cx) {
            return;
        }
        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx, state);
            }
            RenderMode::Results | RenderMode::Done => {
                let hill_background = self.controller.render_results_background(cx, state);
                results::render(
                    cx,
                    self.controller.resources(),
                    state,
                    self.controller.ui_state().current_page(),
                    hill_background,
                );
            }
        }
    }

    fn handle_input(
        &mut self,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> NavSignal {
        if let Some(signal) = self
            .controller
            .handle_alert_input(&mut self.record_notification, event)
        {
            return signal;
        }
        if self.controller.render_mode() == RenderMode::Done {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                return NavSignal::Back;
            }
            return NavSignal::None;
        }

        if self.is_result_display_state() {
            if matches!(event, UiEvent::KeyDown(Key::Escape)) {
                return self.dismiss_results_and_advance(state, cx);
            }
            let total_pages = state
                .active_competition
                .as_ref()
                .and_then(|active| active.koth_runtime())
                .map(results::total_pages)
                .unwrap_or(1);
            if handle_paged_result_event(event, self.controller.ui_state_mut(), total_pages) {
                let signal = self.dismiss_results_and_advance(state, cx);
                if !matches!(signal, NavSignal::None) {
                    return signal;
                }
            }
            return NavSignal::None;
        }

        match self
            .controller
            .handle_jump_scene_event(event, false, false, true, state, cx)
        {
            JumpInputResult::Consumed => return NavSignal::None,
            JumpInputResult::OpenSetup => return NavSignal::Route(RouteTarget::OptionsMenu),
            JumpInputResult::None => {}
        }

        NavSignal::None
    }
}

fn handle_paged_result_event(
    event: UiEvent,
    ui_state: &mut CompetitionUiState,
    total_pages: usize,
) -> bool {
    page_event(event, PageEventMap::Koth)
        .is_some_and(|event| ui_state.handle_page_event(event, total_pages, PageDismissal::KOTH))
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
        if self.controller.has_profile_alert() {
            return;
        }
        let persistence = cx.persistence();
        self.controller
            .record_acknowledged_human_jump(cx.state, &persistence);

        if !matches!(
            self.controller.render_mode(),
            RenderMode::Results | RenderMode::Done
        ) {
            if let Some(command) = self.controller.drive(cx.state) {
                self.apply_command(command, cx.state, &persistence);
            }
        }

        if self.controller.render_mode() == RenderMode::Jump {
            self.controller.update_scene(cx.state);
        }
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if self.controller.has_profile_alert() {
            match event {
                UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::TextWithModifiers(_, _) => {
                    nav.back();
                }
                UiEvent::Quit | UiEvent::Tick => {}
            }
            nav.consume();
            return;
        }
        if self.controller.ui_state_mut().take_cup_exit_request()
            && matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_))
            && !self
                .controller
                .scene()
                .is_some_and(|scene| scene.is_save_dialog_active())
        {
            cx.state.abort_active_competition();
            nav.navigate(RouteTarget::MainMenu);
            return;
        }
        let event = if is_quit_event(event) {
            self.controller.ui_state_mut().request_cup_exit();
            UiEvent::KeyDown(Key::Escape)
        } else {
            event
        };
        let persistence = cx.persistence();
        self.handle_input(event, cx.state, &persistence)
            .dispatch(nav);
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint, cx.state);
    }

    fn has_modal(&self) -> bool {
        self.controller.has_profile_alert()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::page_nav::PageEvent;
    use crate::ui::Key;
    use crate::views::jump::competition::ui_state::CompetitionUiState;

    #[test]
    fn requested_result_keys_navigate_pages() {
        let forward = [
            UiEvent::KeyDown(Key::Right),
            UiEvent::KeyDown(Key::Down),
            UiEvent::KeyDown(Key::PageDown),
            UiEvent::Text(' '),
        ];
        for event in forward {
            let mut ui_state = CompetitionUiState::new();
            ui_state.enter_results();
            assert!(!handle_paged_result_event(event, &mut ui_state, 2));
            assert_eq!(ui_state.current_page(), 1);
        }

        let backward = [
            UiEvent::KeyDown(Key::Left),
            UiEvent::KeyDown(Key::Up),
            UiEvent::KeyDown(Key::PageUp),
        ];
        for event in backward {
            let mut ui_state = CompetitionUiState::new();
            ui_state.enter_results();
            assert!(!ui_state.handle_page_event(PageEvent::Next, 2, PageDismissal::TEAM));
            assert!(!handle_paged_result_event(event, &mut ui_state, 2));
            assert_eq!(ui_state.current_page(), 0);
        }
    }

    #[test]
    fn enter_dismisses_only_on_the_last_page() {
        let mut ui_state = CompetitionUiState::new();
        ui_state.enter_results();
        assert!(!handle_paged_result_event(
            UiEvent::KeyDown(Key::Enter),
            &mut ui_state,
            2,
        ));
        assert_eq!(ui_state.current_page(), 0);

        assert!(!ui_state.handle_page_event(PageEvent::Next, 2, PageDismissal::TEAM));
        assert!(handle_paged_result_event(
            UiEvent::KeyDown(Key::Enter),
            &mut ui_state,
            2,
        ));

        let mut single_page = CompetitionUiState::new();
        single_page.enter_results();
        assert!(handle_paged_result_event(
            UiEvent::KeyDown(Key::Enter),
            &mut single_page,
            1,
        ));
    }

    #[test]
    fn forward_navigation_dismisses_after_the_last_page() {
        let mut ui_state = CompetitionUiState::new();
        ui_state.enter_results();
        assert!(!handle_paged_result_event(
            UiEvent::KeyDown(Key::Down),
            &mut ui_state,
            2,
        ));
        assert_eq!(ui_state.current_page(), 1);
        assert!(handle_paged_result_event(
            UiEvent::KeyDown(Key::PageDown),
            &mut ui_state,
            2,
        ));
    }
}

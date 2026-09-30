use super::setup::{SetupAction, TeamCupSetup};
use crate::competition::completion::CompletionSaveGuard;
use crate::competition::team_cup::types::{
    TeamCupJumpContext, TeamCupPhase, TeamCupResultsKind, TeamCupRuntime,
};
use crate::components::page_nav::{
    is_quit_event, page_event, PageCursor, PageDismissal, PageEventMap,
};
use crate::data::records::{insert_ranked_hiscore, Hiscore};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen, NavSignal, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::current_timestamp;
use crate::ui::UiCanvas;
use crate::ui::{Blinker, Key, ScreenEventCx, UiEvent};
use crate::views::jump::competition::controller::CompetitionJumpController;
use crate::views::jump::competition::flow::{CompetitionFlowCommand, JumpInputResult};
use crate::views::jump::competition::ui_state::RenderMode;
use crate::views::jump::team_cup::results as team_cup_results;
use crate::views::records::RecordNotification;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewPhase {
    Setup,
    Jumping,
    Done,
}

#[derive(Debug, Default)]
struct TeamResultsUi {
    stats: bool,
    page: PageCursor,
}

impl TeamResultsUi {
    fn enter(&mut self, extra_statistics: bool) {
        self.stats = extra_statistics;
        self.page.reset();
    }

    fn toggle_stats(&mut self) {
        self.stats = !self.stats;
        self.page.reset();
    }

    fn show_stats(&mut self) {
        self.stats = true;
        self.page.reset();
    }
}

pub struct TeamCupJumpView {
    controller: CompetitionJumpController<TeamCupRuntime>,
    phase: ViewPhase,
    blinker: Blinker,
    setup: TeamCupSetup,
    cursor_visible: bool,
    results_kind: TeamCupResultsKind,
    results_ui: TeamResultsUi,
    completion_saved: CompletionSaveGuard,
    record_notification: Option<RecordNotification>,
}

impl TeamCupJumpView {
    pub(crate) fn new(resources: ResourcesRef, state: &GameState) -> Self {
        let runtime_phase = state
            .active_competition
            .as_ref()
            .and_then(|active| active.team_cup_runtime())
            .map_or(TeamCupPhase::Setup, |runtime| runtime.phase);
        let (phase, results_kind) = view_state_for_runtime(runtime_phase);
        let mut controller = CompetitionJumpController::new(resources, false, None);
        match runtime_phase {
            TeamCupPhase::LegResults | TeamCupPhase::TeamCupStandings => {
                controller.enter_results();
            }
            TeamCupPhase::Complete => controller.enter_done(),
            TeamCupPhase::Setup | TeamCupPhase::Jumping => {}
        }
        Self {
            controller,
            phase,
            blinker: Blinker::new(),
            setup: TeamCupSetup::new_with_dummy(),
            cursor_visible: true,
            results_kind,
            results_ui: TeamResultsUi::default(),
            completion_saved: CompletionSaveGuard::default(),
            record_notification: None,
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
            } => {
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
                    true,
                    state,
                );
            }
            CompetitionFlowCommand::ShowResults(kind) => {
                self.results_kind = kind;
                self.results_ui.enter(
                    state.config.extra_statistics != 0 && kind == TeamCupResultsKind::Standings,
                );
                self.controller.enter_results();
            }
            CompetitionFlowCommand::Done => {
                self.phase = ViewPhase::Done;
                self.controller.enter_done();
            }
        }
    }

    fn paint_content(&mut self, cx: &mut dyn UiCanvas, state: &GameState) {
        if self.controller.paint_alerts(&self.record_notification, cx) {
            return;
        }
        if self.phase == ViewPhase::Setup {
            self.setup
                .paint(cx, self.controller.resources(), state, self.cursor_visible);
            return;
        }
        if self.phase == ViewPhase::Done {
            team_cup_results::render(
                cx,
                self.controller.resources(),
                state,
                TeamCupResultsKind::Standings,
                false,
            );
            return;
        }

        match self.controller.render_mode() {
            RenderMode::Jump => {
                self.controller.render_jump(cx, state);
            }
            RenderMode::Results => {
                let hill_background = self.controller.render_results_background(cx, state);
                let runtime = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| active.team_cup_runtime());
                if self.results_ui.stats && state.config.extra_statistics != 0 {
                    if let Some(runtime) = runtime {
                        team_cup_results::render_stats(
                            cx,
                            self.controller.resources(),
                            runtime,
                            self.results_ui.page.current(),
                            hill_background,
                        );
                    }
                } else {
                    team_cup_results::render(
                        cx,
                        self.controller.resources(),
                        state,
                        self.results_kind,
                        hill_background,
                    );
                }
            }
            RenderMode::Done => {}
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
        if self.phase == ViewPhase::Done {
            if !matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                return NavSignal::None;
            }
            self.save_team_cup_results(state, cx);
            return if self.record_notification.is_some() {
                NavSignal::None
            } else {
                NavSignal::Back
            };
        }

        if self.phase == ViewPhase::Setup {
            if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) {
                return NavSignal::Back;
            }
            if self
                .setup
                .handle_event(self.controller.resources(), state, event)
                == SetupAction::StartJumping
            {
                self.phase = ViewPhase::Jumping;
                self.drive_until_visible(state);
            }
            return NavSignal::None;
        }

        if self.phase == ViewPhase::Jumping {
            match self
                .controller
                .handle_jump_scene_event(event, false, false, true, state, cx)
            {
                JumpInputResult::Consumed => return NavSignal::None,
                JumpInputResult::OpenSetup => return NavSignal::Route(RouteTarget::OptionsMenu),
                JumpInputResult::None => {}
            }
        }

        if self.controller.render_mode() == RenderMode::Results {
            if matches!(event, UiEvent::KeyDown(Key::Escape)) {
                return NavSignal::Back;
            }

            if state.config.extra_statistics == 0 {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    self.dismiss_results(state);
                }
                return NavSignal::None;
            }

            if self.results_ui.stats {
                let total = state
                    .active_competition
                    .as_ref()
                    .and_then(|active| active.team_cup_runtime())
                    .map(|runtime| team_cup_results::build_stats_pages(runtime).len())
                    .unwrap_or(0);
                if let Some(page_event) = page_event(event, PageEventMap::Team) {
                    if self
                        .results_ui
                        .page
                        .apply(page_event, total, PageDismissal::TEAM)
                    {
                        self.dismiss_results(state);
                    }
                    return NavSignal::None;
                }
            }

            match event {
                UiEvent::Text('s' | 'S') if self.skip_second_round(state) => {
                    self.dismiss_results(state);
                }
                UiEvent::Text('s' | 'S') if self.results_kind == TeamCupResultsKind::Standings => {
                    self.results_ui.show_stats();
                }
                UiEvent::Text('t' | 'T') => self.results_ui.toggle_stats(),
                UiEvent::KeyDown(Key::Enter) => self.dismiss_results(state),
                _ => {}
            }
            return NavSignal::None;
        }

        NavSignal::None
    }

    fn dismiss_results(&mut self, state: &mut GameState) {
        if let Some(cmd) = self.controller.dismiss_results_and_advance(state) {
            self.apply_command(cmd, state);
        }
    }

    fn skip_second_round(&mut self, state: &mut GameState) -> bool {
        state
            .active_competition
            .as_mut()
            .and_then(|active| active.team_cup_runtime_mut())
            .is_some_and(TeamCupRuntime::skip_second_round)
    }

    fn save_team_cup_results(&mut self, state: &mut GameState, cx: &Persistence) {
        let teams = {
            let Some(active) = state.active_competition.as_ref() else {
                return;
            };
            let Some(tc) = active.team_cup_runtime() else {
                return;
            };
            tc.overall_standings()
        };
        if !self.completion_saved.claim() {
            return;
        }

        let time = current_timestamp();
        let mut inserted = Vec::new();
        for team in teams.iter().filter(|team| team.is_human) {
            let candidate = Hiscore {
                name: team.name.clone(),
                pos: team.rank,
                score: team.primary_score,
                time: time.clone(),
                is_computer: false,
            };
            if insert_ranked_hiscore(&mut state.records.top, 20..30, candidate.clone()) {
                inserted.push(candidate);
            }
        }
        if !inserted.is_empty() {
            self.record_notification = Some(RecordNotification::new(
                "NEW TEAM CUP RECORD!",
                inserted,
                None,
            ));
        }

        self.controller.save_results(state, cx);
    }
}

impl GameScreen for TeamCupJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        self.cursor_visible = self.blinker.visible(10, 10);

        if self.controller.has_profile_alert() {
            return;
        }
        if self.phase == ViewPhase::Setup {
            if !self.setup.has_teams() {
                self.init_setup(cx.state);
            }
            return;
        }
        let persistence = cx.persistence();
        if self.phase == ViewPhase::Done {
            self.save_team_cup_results(cx.state, &persistence);
            return;
        }
        if self.phase != ViewPhase::Jumping {
            return;
        }

        self.controller
            .record_acknowledged_human_jump(cx.state, &persistence);

        if (self.controller.ui_state().is_outcome_recorded()
            && self.controller.render_mode() != RenderMode::Results)
            || (self.controller.render_mode() == RenderMode::Jump
                && self.controller.scene().is_none())
        {
            if let Some(cmd) = self.controller.drive(cx.state) {
                self.apply_command(cmd, cx.state);
            }
        }

        if self.phase == ViewPhase::Done {
            self.save_team_cup_results(cx.state, &persistence);
            return;
        }

        self.controller.update_scene(cx.state);
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

fn view_state_for_runtime(phase: TeamCupPhase) -> (ViewPhase, TeamCupResultsKind) {
    match phase {
        TeamCupPhase::Setup => (ViewPhase::Setup, TeamCupResultsKind::LegResults),
        TeamCupPhase::Jumping => (ViewPhase::Jumping, TeamCupResultsKind::LegResults),
        TeamCupPhase::LegResults => (ViewPhase::Jumping, TeamCupResultsKind::LegResults),
        TeamCupPhase::TeamCupStandings => (ViewPhase::Jumping, TeamCupResultsKind::Standings),
        TeamCupPhase::Complete => (ViewPhase::Done, TeamCupResultsKind::Standings),
    }
}

#[cfg(test)]
mod tests {
    use super::{view_state_for_runtime, TeamResultsUi, ViewPhase};
    use crate::competition::team_cup::types::{TeamCupPhase, TeamCupResultsKind};
    use crate::components::page_nav::{PageDismissal, PageEvent};

    #[test]
    fn loaded_runtime_phase_restores_view_mode() {
        assert_eq!(
            view_state_for_runtime(TeamCupPhase::Setup),
            (ViewPhase::Setup, TeamCupResultsKind::LegResults)
        );
        assert_eq!(
            view_state_for_runtime(TeamCupPhase::Jumping),
            (ViewPhase::Jumping, TeamCupResultsKind::LegResults)
        );
        assert_eq!(
            view_state_for_runtime(TeamCupPhase::LegResults),
            (ViewPhase::Jumping, TeamCupResultsKind::LegResults)
        );
        assert_eq!(
            view_state_for_runtime(TeamCupPhase::TeamCupStandings),
            (ViewPhase::Jumping, TeamCupResultsKind::Standings)
        );
        assert_eq!(
            view_state_for_runtime(TeamCupPhase::Complete),
            (ViewPhase::Done, TeamCupResultsKind::Standings)
        );
    }

    #[test]
    fn statistics_navigation_stays_within_one_team_page() {
        let mut ui = TeamResultsUi::default();
        ui.enter(true);
        ui.page.apply(PageEvent::Next, 1, PageDismissal::TEAM);
        assert!(ui.stats);
        assert_eq!(ui.page.current(), 0);
        ui.page.apply(PageEvent::Previous, 1, PageDismissal::TEAM);
        assert_eq!(ui.page.current(), 0);
    }

    #[test]
    fn statistics_navigation_moves_between_two_human_teams() {
        let mut ui = TeamResultsUi::default();
        ui.enter(true);
        ui.page.apply(PageEvent::Next, 2, PageDismissal::TEAM);
        assert_eq!(ui.page.current(), 1);
        ui.page.apply(PageEvent::Next, 2, PageDismissal::TEAM);
        assert_eq!(ui.page.current(), 1);
        ui.page.apply(PageEvent::Previous, 2, PageDismissal::TEAM);
        assert_eq!(ui.page.current(), 0);
        ui.toggle_stats();
        assert!(!ui.stats);
        assert_eq!(ui.page.current(), 0);
    }
}

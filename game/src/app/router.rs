use crate::components::layout::MainLayout;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::{Key, NavAction, ScreenBackground, ScreenEventCx, UiEvent};
use crate::views::{
    CompetitionJumpView, CustomCupSetupView, EditHillView, HallOfFameView, HillMakerView,
    HillRecordsView, JumpMenuView, KothHillPickerView, KothSetupView, MainMenuView, ProfilesView,
    ReplayBrowserView, ReplayView, SetupView, TrainingSetupView, WelcomeScreenView,
};
use engine::audio::Beep;
use std::rc::Rc;

fn make_screen(
    target: &RouteTarget,
    resources: &ResourcesRef,
    save_manager: &SaveRef,
    state: &mut GameState,
) -> Box<dyn GameScreen> {
    match target {
        RouteTarget::MainMenu => Box::new(MainMenuView::new()),
        RouteTarget::JumpMenu => Box::new(JumpMenuView::new(resources.clone())),
        RouteTarget::Practice => Box::new(TrainingSetupView::new(resources.clone(), state)),
        RouteTarget::CompetitionJump => {
            let Some(kind) = state
                .active_competition
                .as_ref()
                .map(|competition| competition.kind())
            else {
                return Box::new(MainMenuView::new());
            };
            Box::new(CompetitionJumpView::new(resources.clone(), state, kind))
        }
        RouteTarget::CustomCupSetup => Box::new(CustomCupSetupView::new(resources.clone())),
        RouteTarget::Replays => Box::new(ReplayBrowserView::new(resources.clone())),
        RouteTarget::ReplayPlayback { trace, return_to } => Box::new(ReplayView::new(
            resources.clone(),
            trace.as_ref().clone(),
            *return_to,
            state.config.graphics_detail == 1,
        )),
        RouteTarget::ProfilesList => Box::new(ProfilesView::new(resources.clone(), state)),
        RouteTarget::HallOfFame => Box::new(HallOfFameView::new(resources.clone())),
        RouteTarget::HillRecords => Box::new(HillRecordsView::new(resources.clone())),
        RouteTarget::OptionsMenu => Box::new(SetupView::new(
            resources.clone(),
            Persistence::from(save_manager.clone()),
        )),
        RouteTarget::KothHillPicker => Box::new(KothHillPickerView::new(resources.clone())),
        RouteTarget::KothSetup => Box::new(KothSetupView::new(resources.clone())),
        RouteTarget::HillMakerSetup => Box::new(HillMakerView::new(resources.clone())),
        RouteTarget::EditHill(filename) => {
            Box::new(EditHillView::new(resources.clone(), filename.clone()))
        }
        RouteTarget::Welcome => Box::new(WelcomeScreenView::new(resources.clone())),
    }
}

fn validate_route(target: RouteTarget, state: &GameState) -> RouteTarget {
    if target == RouteTarget::CompetitionJump && state.active_competition.is_none() {
        RouteTarget::MainMenu
    } else {
        target
    }
}

fn apply_history_transition<T>(
    history: &mut Vec<(RouteTarget, T)>,
    current_route: &mut Option<RouteTarget>,
    target: RouteTarget,
    previous: T,
) {
    if target == RouteTarget::MainMenu {
        history.clear();
    } else if let Some(route) = current_route.take() {
        history.push((route, previous));
    }
    *current_route = Some(target);
}

pub struct AppRouter {
    current: Box<dyn GameScreen>,
    current_route: Option<RouteTarget>,
    history: Vec<(RouteTarget, Box<dyn GameScreen>)>,
    resources: ResourcesRef,
    pub(crate) state: GameState,
    save_manager: SaveRef,
    layout: MainLayout,
    quit_requested: bool,
}

impl AppRouter {
    pub fn new(
        route: RouteTarget,
        resources: ResourcesRef,
        mut state: GameState,
        save_manager: SaveRef,
        layout: MainLayout,
    ) -> Self {
        let route = validate_route(route, &state);
        let current = make_screen(&route, &resources, &save_manager, &mut state);
        Self {
            current,
            current_route: Some(route),
            history: Vec::with_capacity(4),
            resources,
            state,
            save_manager,
            layout,
            quit_requested: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit_requested
    }

    pub fn drain_audio_cues(&mut self) -> impl Iterator<Item = Beep> + '_ {
        self.state.drain_audio_cues()
    }

    pub fn save_persistent_state(&self) {
        self.save_manager.save_persistent_state(
            &self.state.config,
            &self.state.profiles,
            &self.state.records,
        );
    }

    pub fn update(&mut self) {
        let mut cx = GameCx {
            state: &mut self.state,
            save_manager: self.save_manager.clone(),
            layout: &self.layout,
        };
        self.current.update(&mut cx);
    }

    pub fn handle_event(&mut self, event: UiEvent) {
        if matches!(event, UiEvent::KeyDown(Key::Escape)) && !self.current.has_modal() {
            let (action, consumed) = self.send_event(event);
            if action == NavAction::None && !consumed {
                self.back();
            } else {
                self.dispatch_nav(action);
            }
            return;
        }

        let action = self.send_event(event).0;
        self.dispatch_nav(action);
    }

    fn send_event(&mut self, event: UiEvent) -> (NavAction<RouteTarget>, bool) {
        let mut nav = ScreenEventCx::default();
        let mut cx = GameCx {
            state: &mut self.state,
            save_manager: self.save_manager.clone(),
            layout: &self.layout,
        };
        self.current.event(&mut cx, &mut nav, event);
        let action = nav.take_action();
        (action, nav.is_consumed())
    }

    fn dispatch_nav(&mut self, action: NavAction<RouteTarget>) {
        match action {
            NavAction::None => {}
            NavAction::Navigate(route) => self.navigate(route),
            NavAction::Back => self.back(),
            NavAction::Quit => self.quit_requested = true,
        }
    }

    pub fn paint(&mut self, paint: &mut dyn UiCanvas) {
        let mut cx = GameCx {
            state: &mut self.state,
            save_manager: self.save_manager.clone(),
            layout: &self.layout,
        };
        self.current.paint(&mut cx, paint);
    }

    pub fn screen_background(&self) -> ScreenBackground {
        self.current.background()
    }

    fn navigate(&mut self, target: RouteTarget) {
        let target = validate_route(target, &self.state);
        let next = make_screen(
            &target,
            &self.resources,
            &self.save_manager,
            &mut self.state,
        );
        let previous_screen = std::mem::replace(&mut self.current, next);
        apply_history_transition(
            &mut self.history,
            &mut self.current_route,
            target,
            previous_screen,
        );
    }

    fn back(&mut self) {
        if let Some((prev, screen)) = self.history.pop() {
            self.current_route = Some(prev);
            self.current = screen;
        }
    }
}

pub fn create_router(
    resources: ResourcesRef,
    state: GameState,
    start_route: RouteTarget,
    save_manager: SaveRef,
) -> AppRouter {
    let layout = MainLayout::new(
        Rc::clone(&resources.langbase),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    AppRouter::new(start_route, resources, state, save_manager, layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn competition_route_without_active_competition_is_invalid() {
        let state = GameState::default();
        assert_eq!(
            validate_route(RouteTarget::CompetitionJump, &state),
            RouteTarget::MainMenu
        );
    }

    #[test]
    fn normal_navigation_pushes_and_back_pops_history() {
        let mut history = Vec::new();
        let mut current = Some(RouteTarget::MainMenu);
        apply_history_transition(&mut history, &mut current, RouteTarget::Replays, "main");

        assert_eq!(current, Some(RouteTarget::Replays));
        assert_eq!(history.pop(), Some((RouteTarget::MainMenu, "main")));
    }

    #[test]
    fn main_menu_navigation_resets_history() {
        let mut history = vec![(RouteTarget::Welcome, "welcome")];
        let mut current = Some(RouteTarget::Welcome);
        apply_history_transition(&mut history, &mut current, RouteTarget::MainMenu, "intro");

        assert!(history.is_empty());
        assert_eq!(current, Some(RouteTarget::MainMenu));
    }
}

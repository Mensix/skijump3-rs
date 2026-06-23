use crate::competition::active::ActiveCompetitionKind;
use crate::components::layout::MainLayout;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::views::{
    CompetitionJumpView, CustomCupSetupView, EditHillView, HallOfFameView, HillMakerView,
    HillRecordsView, JumpMenuView, KothHillPickerView, KothSetupView, LoadCupView, MainMenuView,
    MultiplayerLobbyView, MultiplayerMenuView, ProfilesView, ReplayBrowserView, ReplayView,
    SetupView, TrainingSetupView, WelcomeScreenView,
};
use engine::oxide::{Key, NavAction, PaintCx, ScreenBackground, ScreenEventCx, UiEvent};
use std::rc::Rc;

const VERSION: &str = "3.14";

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
        RouteTarget::Jump | RouteTarget::CompetitionJump => {
            let kind = state
                .active_competition
                .as_ref()
                .map(|competition| competition.kind())
                .unwrap_or(ActiveCompetitionKind::Individual);
            Box::new(CompetitionJumpView::new(
                resources.clone(),
                save_manager.clone(),
                state,
                kind,
            ))
        }
        RouteTarget::CustomCupSetup => Box::new(CustomCupSetupView::new(
            resources.clone(),
            save_manager.clone(),
        )),
        RouteTarget::Replays => Box::new(ReplayBrowserView::new(resources.clone())),
        RouteTarget::ReplayPlayback => Box::new(ReplayView::new(
            resources.clone(),
            state.selected_replay.clone(),
        )),
        RouteTarget::ProfilesList => Box::new(ProfilesView::new(
            resources.clone(),
            save_manager.clone(),
            state,
        )),
        RouteTarget::LoadCup => Box::new(LoadCupView::new(save_manager.clone())),
        RouteTarget::HallOfFame => Box::new(HallOfFameView::new(resources.clone())),
        RouteTarget::HillRecords => Box::new(HillRecordsView::new(resources.clone())),
        RouteTarget::OptionsMenu => {
            Box::new(SetupView::new(resources.clone(), save_manager.clone()))
        }
        RouteTarget::KothHillPicker => Box::new(KothHillPickerView::new(
            resources.clone(),
            save_manager.clone(),
        )),
        RouteTarget::KothSetup => {
            Box::new(KothSetupView::new(resources.clone(), save_manager.clone()))
        }
        RouteTarget::HillMakerSetup => Box::new(HillMakerView::new(resources.clone())),
        RouteTarget::EditHill => {
            let filename = state.nav_edit_hill.clone();
            Box::new(EditHillView::new(resources.clone(), filename))
        }
        RouteTarget::Welcome => Box::new(WelcomeScreenView::new(resources.clone())),
        RouteTarget::MultiplayerMenu => Box::new(MultiplayerMenuView::new()),
        RouteTarget::MultiplayerLobby => {
            let lobby = state
                .pending_lobby
                .take()
                .expect("pending_lobby not set for MultiplayerLobby");
            Box::new(MultiplayerLobbyView::new(lobby))
        }
        RouteTarget::Quit => Box::new(MainMenuView::new()),
        _ => unreachable!(),
    }
}

pub struct AppRouter {
    current: Box<dyn GameScreen>,
    current_route: Option<RouteTarget>,
    history: Vec<RouteTarget>,
    resources: ResourcesRef,
    pub(crate) state: GameState,
    save_manager: SaveRef,
    layout: MainLayout,
}

impl AppRouter {
    pub fn new(
        route: RouteTarget,
        resources: ResourcesRef,
        mut state: GameState,
        save_manager: SaveRef,
        layout: MainLayout,
    ) -> Self {
        let current = make_screen(&route, &resources, &save_manager, &mut state);
        Self {
            current,
            current_route: Some(route),
            history: Vec::with_capacity(4),
            resources,
            state,
            save_manager,
            layout,
        }
    }

    pub fn current_route(&self) -> Option<&RouteTarget> {
        self.current_route.as_ref()
    }

    pub fn update(&mut self) {
        let mut cx = GameCx {
            state: &mut self.state,
            save_manager: &self.save_manager,
            layout: &self.layout,
        };
        self.current.update(&mut cx);
    }

    pub fn handle_event(&mut self, event: UiEvent) {
        if matches!(event, UiEvent::KeyDown(Key::Escape)) {
            if self.current.has_modal() {
                self.current.dismiss_modal();
                return;
            }
            let mut nav = ScreenEventCx::default();
            {
                let mut cx = GameCx {
                    state: &mut self.state,
                    save_manager: &self.save_manager,
                    layout: &self.layout,
                };
                self.current.event(&mut cx, &mut nav, event);
            }
            match nav.take_action() {
                NavAction::None if nav.is_consumed() => {}
                NavAction::None => self.back(),
                NavAction::Navigate(route) => self.navigate(route),
                NavAction::Back => self.back(),
                NavAction::Quit => self.navigate(RouteTarget::Quit),
            }
            return;
        }

        let mut nav = ScreenEventCx::default();
        {
            let mut cx = GameCx {
                state: &mut self.state,
                save_manager: &self.save_manager,
                layout: &self.layout,
            };
            self.current.event(&mut cx, &mut nav, event);
        }
        match nav.take_action() {
            NavAction::None => {}
            NavAction::Navigate(route) => self.navigate(route),
            NavAction::Back => self.back(),
            NavAction::Quit => self.navigate(RouteTarget::Quit),
        }
    }

    pub fn paint(&mut self, paint: &mut PaintCx<'_>) {
        let mut cx = GameCx {
            state: &mut self.state,
            save_manager: &self.save_manager,
            layout: &self.layout,
        };
        self.current.paint(&mut cx, paint);
    }

    pub fn screen_background(&self) -> ScreenBackground {
        self.current.background()
    }

    fn navigate(&mut self, target: RouteTarget) {
        if let Some(prev) = self.current_route {
            self.history.push(prev);
        }
        self.current_route = Some(target);
        self.current = make_screen(
            &target,
            &self.resources,
            &self.save_manager,
            &mut self.state,
        );
    }

    fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.current_route = Some(prev);
            self.current = make_screen(&prev, &self.resources, &self.save_manager, &mut self.state);
        }
    }
}

pub fn create_router(
    resources: ResourcesRef,
    state: GameState,
    start_route: RouteTarget,
    save_manager: SaveRef,
) -> AppRouter {
    let layout = MainLayout::new(Rc::clone(&resources.langbase), VERSION.to_string());
    AppRouter::new(start_route, resources, state, save_manager, layout)
}

use crate::components::layout::MainLayout;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{GameStateRef, ResourcesRef};
use crate::views::{
    CompetitionJumpView, CustomCupSetupView, EditHillView, HallOfFameView, HillMakerView,
    HillRecordsView, JumpMenuView, KothHillPickerView, KothSetupView, MainMenuView, ProfilesView,
    ReplayBrowserView, ReplayView, SetupView, TrainingSetupView, WelcomeScreenView,
};
use engine::oxide::{NavAction, PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};
use std::rc::Rc;

const VERSION: &str = "3.12";

fn make_screen(
    target: &RouteTarget,
    resources: &ResourcesRef,
    state: &GameStateRef,
    save_manager: &SaveRef,
    layout: &MainLayout,
) -> Box<dyn Screen<RouteTarget>> {
    match target {
        RouteTarget::MainMenu => Box::new(MainMenuView::new(layout.clone(), state.clone())),
        RouteTarget::JumpMenu => Box::new(JumpMenuView::new(
            layout.clone(),
            state.clone(),
            resources.clone(),
        )),
        RouteTarget::Practice => Box::new(TrainingSetupView::new(resources.clone(), state.clone())),
        RouteTarget::Jump => Box::new(CompetitionJumpView::new(
            resources.clone(),
            state.clone(),
            save_manager.clone(),
        )),
        RouteTarget::CompetitionJump => Box::new(CompetitionJumpView::new(
            resources.clone(),
            state.clone(),
            save_manager.clone(),
        )),
        RouteTarget::CustomCupSetup => {
            Box::new(CustomCupSetupView::new(resources.clone(), state.clone()))
        }
        RouteTarget::Replays => Box::new(ReplayBrowserView::new(
            resources.clone(),
            state.clone(),
            layout.clone(),
        )),
        RouteTarget::ReplayPlayback => Box::new(ReplayView::new(resources.clone(), state.clone())),
        RouteTarget::ProfilesList => Box::new(ProfilesView::new(
            resources.clone(),
            state.clone(),
            save_manager.clone(),
        )),
        RouteTarget::HallOfFame => Box::new(HallOfFameView::new(resources.clone(), state.clone())),
        RouteTarget::HillRecords => {
            Box::new(HillRecordsView::new(resources.clone(), state.clone()))
        }
        RouteTarget::OptionsMenu => Box::new(SetupView::new(
            resources.clone(),
            state.clone(),
            save_manager.clone(),
        )),
        RouteTarget::KothHillPicker => Box::new(KothHillPickerView::new(
            resources.clone(),
            state.clone(),
            save_manager.clone(),
        )),
        RouteTarget::KothSetup => Box::new(KothSetupView::new(
            resources.clone(),
            state.clone(),
            save_manager.clone(),
        )),
        RouteTarget::HillMakerSetup => Box::new(HillMakerView::new(resources.clone())),
        RouteTarget::EditHill => Box::new(EditHillView::new(resources.clone())),
        RouteTarget::Welcome => Box::new(WelcomeScreenView::new(
            resources.clone(),
            state.clone(),
            resources.langbase.languages.clone(),
            save_manager.clone(),
        )),
        RouteTarget::Quit => Box::new(MainMenuView::new(layout.clone(), state.clone())),
        _ => unreachable!(),
    }
}

pub struct AppRouter {
    current: Box<dyn Screen<RouteTarget>>,
    current_route: Option<RouteTarget>,
    history: Vec<RouteTarget>,
    resources: ResourcesRef,
    state: GameStateRef,
    save_manager: SaveRef,
    layout: MainLayout,
}

impl AppRouter {
    fn new(
        route: RouteTarget,
        resources: ResourcesRef,
        state: GameStateRef,
        save_manager: SaveRef,
        layout: MainLayout,
    ) -> Self {
        let current = make_screen(&route, &resources, &state, &save_manager, &layout);
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
        self.current.update();
    }

    pub fn handle_event(&mut self, event: UiEvent) {
        let mut cx = ScreenEventCx::default();
        self.current.event(&mut cx, event);
        match cx.take_action() {
            NavAction::None => {}
            NavAction::Navigate(route) => self.navigate(route),
            NavAction::Back => self.back(),
            NavAction::Quit => self.navigate(RouteTarget::Quit),
        }
    }

    pub fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.current.paint(cx);
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
            &self.state,
            &self.save_manager,
            &self.layout,
        );
    }

    fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.current_route = Some(prev);
            self.current = make_screen(
                &prev,
                &self.resources,
                &self.state,
                &self.save_manager,
                &self.layout,
            );
        }
    }
}

pub fn create_router(
    resources: ResourcesRef,
    state: GameStateRef,
    start_route: RouteTarget,
    save_manager: SaveRef,
) -> AppRouter {
    let layout = MainLayout::new(
        Rc::clone(&resources.langbase),
        VERSION.to_string(),
        state.clone(),
    );
    AppRouter::new(start_route, resources, state, save_manager, layout)
}

use crate::components::layout::MainLayout;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::{
    CompetitionJumpView, CustomCupSetupView, HallOfFameView, HillRecordsView, JumpMenuView,
    KothSetupView, MainMenuView, ProfilesView, ReplayBrowserView, ReplayView, SetupView,
    TrainingSetupView, WelcomeScreenView,
};
use engine::oxide::{
    NavAction, PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent, UpdateCx,
};
use std::rc::Rc;

const VERSION: &str = "3.12";

type ScreenFactory = Box<dyn Fn() -> Box<dyn AppScreen>>;
type RouteEntry = (RouteTarget, ScreenFactory);

pub trait AppScreen: Screen<RouteTarget> {
    fn screen_background(&self) -> ScreenBackground;
}

impl<T> AppScreen for T
where
    T: Screen<RouteTarget>,
{
    fn screen_background(&self) -> ScreenBackground {
        self.background()
    }
}

pub struct AppRouter {
    current: Box<dyn AppScreen>,
    current_route: Option<RouteTarget>,
    history: Vec<RouteTarget>,
    routes: Vec<RouteEntry>,
}

impl AppRouter {
    fn new(route: RouteTarget, initial: Box<dyn AppScreen>, routes: Vec<RouteEntry>) -> Self {
        Self {
            current: initial,
            current_route: Some(route),
            history: Vec::with_capacity(4),
            routes,
        }
    }

    pub fn current_route(&self) -> Option<&RouteTarget> {
        self.current_route.as_ref()
    }

    pub fn update(&mut self) {
        let mut cx = UpdateCx;
        self.current.update(&mut cx);
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

    pub fn paint(&self, cx: &mut PaintCx<'_>) {
        self.current.paint(cx);
    }

    pub fn screen_background(&self) -> ScreenBackground {
        self.current.screen_background()
    }

    fn navigate(&mut self, target: RouteTarget) {
        if let Some(idx) = self.routes.iter().position(|(route, _)| *route == target) {
            if let Some(prev) = self.current_route {
                self.history.push(prev);
            }
            self.current_route = Some(target);
            self.current = (self.routes[idx].1)();
        }
    }

    fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            if let Some(idx) = self.routes.iter().position(|(route, _)| *route == prev) {
                self.current_route = Some(prev);
                self.current = (self.routes[idx].1)();
            }
        }
    }
}

pub fn create_router(
    resources: ResourcesRef,
    store: StoreRef,
    start_route: RouteTarget,
    save_manager: SaveRef,
) -> AppRouter {
    let layout = MainLayout::new(
        Rc::clone(&resources.langbase),
        VERSION.to_string(),
        store.clone(),
    );

    let registry = RouteRegistry::new(resources, store, layout, save_manager);
    AppRouter::new(
        start_route,
        registry.initial_screen(&start_route),
        registry.routes(),
    )
}

struct RouteRegistry {
    resources: ResourcesRef,
    store: StoreRef,
    layout: MainLayout,
    save_manager: SaveRef,
}

impl RouteRegistry {
    fn new(
        resources: ResourcesRef,
        store: StoreRef,
        layout: MainLayout,
        save_manager: SaveRef,
    ) -> Self {
        Self {
            resources,
            store,
            layout,
            save_manager,
        }
    }

    fn initial_screen(&self, start_route: &RouteTarget) -> Box<dyn AppScreen> {
        match start_route {
            RouteTarget::Welcome => self.welcome_screen(),
            _ => self.main_menu_screen(),
        }
    }

    fn routes(self) -> Vec<RouteEntry> {
        vec![
            (
                RouteTarget::MainMenu,
                self.layout_store(|layout, store| Box::new(MainMenuView::new(layout, store))),
            ),
            (RouteTarget::JumpMenu, {
                let layout = self.layout.clone();
                let resources = self.resources.clone();
                let store = self.store.clone();
                Box::new(move || {
                    Box::new(JumpMenuView::new(
                        layout.clone(),
                        store.clone(),
                        resources.clone(),
                    ))
                })
            }),
            (
                RouteTarget::Practice,
                self.resources_store(|resources, store| {
                    Box::new(TrainingSetupView::new(resources, store))
                }),
            ),
            (
                RouteTarget::Jump,
                self.resources_store(|resources, store| {
                    Box::new(CompetitionJumpView::new(resources, store))
                }),
            ),
            (
                RouteTarget::CompetitionJump,
                self.resources_store(|resources, store| {
                    Box::new(CompetitionJumpView::new(resources, store))
                }),
            ),
            (
                RouteTarget::CustomCupSetup,
                self.resources_store(|resources, store| {
                    Box::new(CustomCupSetupView::new(resources, store))
                }),
            ),
            (RouteTarget::Replays, {
                let resources = self.resources.clone();
                let store = self.store.clone();
                let layout = self.layout.clone();
                Box::new(move || {
                    Box::new(ReplayBrowserView::new(
                        resources.clone(),
                        store.clone(),
                        layout.clone(),
                    ))
                })
            }),
            (
                RouteTarget::ReplayPlayback,
                self.resources_store(|resources, store| {
                    Box::new(ReplayView::new(resources, store))
                }),
            ),
            (RouteTarget::ProfilesList, {
                let resources = self.resources.clone();
                let store = self.store.clone();
                let save_manager = self.save_manager.clone();
                Box::new(move || {
                    Box::new(ProfilesView::new(
                        resources.clone(),
                        store.clone(),
                        save_manager.clone(),
                    ))
                })
            }),
            (
                RouteTarget::HallOfFame,
                self.resources_store(|resources, store| {
                    Box::new(HallOfFameView::new(resources, store))
                }),
            ),
            (
                RouteTarget::HillRecords,
                self.resources_store(|resources, store| {
                    Box::new(HillRecordsView::new(resources, store))
                }),
            ),
            (
                RouteTarget::OptionsMenu,
                self.resources_store(|resources, store| Box::new(SetupView::new(resources, store))),
            ),
            (
                RouteTarget::KothSetup,
                self.resources_store(|resources, store| {
                    Box::new(KothSetupView::new(resources, store))
                }),
            ),
            (
                RouteTarget::Quit,
                self.layout_store(|layout, store| Box::new(MainMenuView::new(layout, store))),
            ),
            (RouteTarget::Welcome, {
                let resources = self.resources;
                let save_manager = self.save_manager;
                Box::new(move || welcome_screen(&resources, save_manager.clone()))
            }),
        ]
    }

    fn main_menu_screen(&self) -> Box<dyn AppScreen> {
        Box::new(MainMenuView::new(self.layout.clone(), self.store.clone()))
    }

    fn welcome_screen(&self) -> Box<dyn AppScreen> {
        welcome_screen(&self.resources, self.save_manager.clone())
    }

    fn resources_store<F>(&self, ctor: F) -> ScreenFactory
    where
        F: Fn(ResourcesRef, StoreRef) -> Box<dyn AppScreen> + 'static,
    {
        let resources = self.resources.clone();
        let store = self.store.clone();
        Box::new(move || ctor(resources.clone(), store.clone()))
    }

    fn layout_store<F>(&self, ctor: F) -> ScreenFactory
    where
        F: Fn(MainLayout, StoreRef) -> Box<dyn AppScreen> + 'static,
    {
        let layout = self.layout.clone();
        let store = self.store.clone();
        Box::new(move || ctor(layout.clone(), store.clone()))
    }
}

fn welcome_screen(resources: &ResourcesRef, save_manager: SaveRef) -> Box<dyn AppScreen> {
    Box::new(WelcomeScreenView::new(
        resources.langbase.languages.clone(),
        &resources.langbase,
        save_manager,
    ))
}

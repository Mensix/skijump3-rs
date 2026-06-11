use crate::components::layout::MainLayout;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::{
    CompetitionJumpView, CustomCupSetupView, HallOfFameView, HillRecordsView, JumpMenuView,
    MainMenuView, ProfilesView, ReplayBrowserView, ReplayView, SetupView, TrainingSetupView,
    WelcomeScreenView,
};
use engine::ui::{RouteEntry, Router, View, ViewFactory};
use std::rc::Rc;

const VERSION: &str = "3.12";

pub fn create_router(
    resources: ResourcesRef,
    store: StoreRef,
    start_route: RouteTarget,
    save_manager: SaveRef,
) -> Router<RouteTarget> {
    let layout = MainLayout::new(
        Rc::clone(&resources.langbase),
        VERSION.to_string(),
        store.clone(),
    );

    let registry = RouteRegistry::new(resources, store, layout, save_manager);
    Router::new(
        start_route,
        registry.initial_view(&start_route),
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

    fn initial_view(&self, start_route: &RouteTarget) -> Box<dyn View<RouteTarget>> {
        match start_route {
            RouteTarget::Welcome => self.welcome_view(),
            _ => self.main_menu_view(),
        }
    }

    fn routes(self) -> Vec<RouteEntry<RouteTarget>> {
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
                RouteTarget::Quit,
                self.layout_store(|layout, store| Box::new(MainMenuView::new(layout, store))),
            ),
            (RouteTarget::Welcome, {
                let resources = self.resources;
                let save_manager = self.save_manager;
                Box::new(move || welcome_view(&resources, save_manager.clone()))
            }),
        ]
    }

    fn main_menu_view(&self) -> Box<dyn View<RouteTarget>> {
        Box::new(MainMenuView::new(self.layout.clone(), self.store.clone()))
    }

    fn welcome_view(&self) -> Box<dyn View<RouteTarget>> {
        welcome_view(&self.resources, self.save_manager.clone())
    }

    fn resources_store<F>(&self, ctor: F) -> ViewFactory<RouteTarget>
    where
        F: Fn(ResourcesRef, StoreRef) -> Box<dyn View<RouteTarget>> + 'static,
    {
        let resources = self.resources.clone();
        let store = self.store.clone();
        Box::new(move || ctor(resources.clone(), store.clone()))
    }

    fn layout_store<F>(&self, ctor: F) -> ViewFactory<RouteTarget>
    where
        F: Fn(MainLayout, StoreRef) -> Box<dyn View<RouteTarget>> + 'static,
    {
        let layout = self.layout.clone();
        let store = self.store.clone();
        Box::new(move || ctor(layout.clone(), store.clone()))
    }
}

fn welcome_view(resources: &ResourcesRef, save_manager: SaveRef) -> Box<dyn View<RouteTarget>> {
    Box::new(WelcomeScreenView::new(
        resources.langbase.languages.clone(),
        &resources.langbase,
        save_manager,
    ))
}

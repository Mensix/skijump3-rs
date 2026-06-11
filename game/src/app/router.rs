use crate::components::layout::MainLayout;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::{
    CustomCupSetupView, HallOfFameView, HillRecordsView, JumpMenuView, MainMenuView, ProfilesView,
    ReplayBrowserView, ReplayView, SetupView, CompetitionJumpView, TrainingSetupView,
    WelcomeScreenView,
};
use engine::ui::{Router, View};
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
    let initial_view: Box<dyn View<RouteTarget>> = match &start_route {
        RouteTarget::Welcome => Box::new(WelcomeScreenView::new(
            resources.langbase.languages.clone(),
            &resources.langbase,
            save_manager.clone(),
        )),
        _ => Box::new(MainMenuView::new(layout.clone(), store.clone())),
    };

    fn rs<F>(r: &ResourcesRef, s: &StoreRef, ctor: F) -> Box<dyn Fn() -> Box<dyn View<RouteTarget>>>
    where
        F: Fn(ResourcesRef, StoreRef) -> Box<dyn View<RouteTarget>> + 'static,
    {
        let r = r.clone();
        let s = s.clone();
        Box::new(move || ctor(r.clone(), s.clone()))
    }

    fn ls<F>(l: &MainLayout, s: &StoreRef, ctor: F) -> Box<dyn Fn() -> Box<dyn View<RouteTarget>>>
    where
        F: Fn(MainLayout, StoreRef) -> Box<dyn View<RouteTarget>> + 'static,
    {
        let l = l.clone();
        let s = s.clone();
        Box::new(move || ctor(l.clone(), s.clone()))
    }

    Router::new(
        start_route,
        initial_view,
        vec![
            (
                RouteTarget::MainMenu,
                ls(&layout, &store, |l, s| Box::new(MainMenuView::new(l, s))),
            ),
            (RouteTarget::JumpMenu, {
                let l = layout.clone();
                let r = resources.clone();
                let s = store.clone();
                Box::new(move || Box::new(JumpMenuView::new(l.clone(), s.clone(), r.clone())))
            }),
            (
                RouteTarget::Practice,
                rs(&resources, &store, |r, s| {
                    Box::new(TrainingSetupView::new(r, s))
                }),
            ),
            (
                RouteTarget::Jump,
                rs(&resources, &store, |r, s| {
                    Box::new(CompetitionJumpView::training(r, s))
                }),
            ),
            (
                RouteTarget::CompetitionJump,
                rs(&resources, &store, |r, s| {
                    Box::new(CompetitionJumpView::new(r, s))
                }),
            ),
            (
                RouteTarget::CustomCupSetup,
                rs(&resources, &store, |r, s| {
                    Box::new(CustomCupSetupView::new(r, s))
                }),
            ),
            (RouteTarget::Replays, {
                let r = resources.clone();
                let s = store.clone();
                let l = layout.clone();
                Box::new(move || Box::new(ReplayBrowserView::new(r.clone(), s.clone(), l.clone())))
            }),
            (
                RouteTarget::ReplayPlayback,
                rs(&resources, &store, |r, s| Box::new(ReplayView::new(r, s))),
            ),
            (RouteTarget::ProfilesList, {
                let r = resources.clone();
                let s = store.clone();
                let sm = save_manager.clone();
                Box::new(move || Box::new(ProfilesView::new(r.clone(), s.clone(), sm.clone())))
            }),
            (
                RouteTarget::HallOfFame,
                rs(&resources, &store, |r, s| {
                    Box::new(HallOfFameView::new(r, s))
                }),
            ),
            (
                RouteTarget::HillRecords,
                rs(&resources, &store, |r, s| {
                    Box::new(HillRecordsView::new(r, s))
                }),
            ),
            (
                RouteTarget::OptionsMenu,
                rs(&resources, &store, |r, s| Box::new(SetupView::new(r, s))),
            ),
            (
                RouteTarget::Quit,
                ls(&layout, &store, |l, s| Box::new(MainMenuView::new(l, s))),
            ),
            (RouteTarget::Welcome, {
                let r = resources;
                let sm = save_manager;
                Box::new(move || {
                    Box::new(WelcomeScreenView::new(
                        r.langbase.languages.clone(),
                        &r.langbase,
                        sm.clone(),
                    ))
                })
            }),
        ],
    )
}

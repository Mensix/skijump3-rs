use crate::competition::builder::build_competition;
use crate::competition::types::CupStyle;
use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{BG_ERASE, BG_LIST, FONT_DEFAULT, FONT_HEADER};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Component, Element, Event, View};

pub struct JumpMenuView {
    menu: Menu,
    layout: MainLayout,
    store: StoreRef,
    resources: ResourcesRef,
}

const JUMP_MENU_ACTIONS: &[Option<RouteTarget>] = &[
    None,                        // 1 - WorldCup (special, builds competition)
    None,                        // 2 - CustomCup (special, builds competition)
    None,                        // 3 - FourHills (special, builds competition)
    Some(RouteTarget::MainMenu), // 4 - TeamCup (not implemented)
    Some(RouteTarget::MainMenu), // 5 - SeasonComplete (not implemented)
    Some(RouteTarget::Practice), // 6 - Practice
    Some(RouteTarget::MainMenu), // 7 - MainMenu
];

impl JumpMenuView {
    #[must_use]
    pub fn new(layout: MainLayout, store: StoreRef, resources: ResourcesRef) -> Self {
        let items = vec![
            MenuItem::new(1, 27),
            MenuItem::new(2, 28),
            MenuItem::new(3, 29),
            MenuItem::new(4, 30),
            MenuItem::new(5, 31),
            MenuItem::new(6, 32),
            MenuItem::with_y(0, 33, 12),
        ];
        Self {
            menu: Menu::new(
                11,
                97,
                108,
                12,
                items,
                &layout.langbase,
                FONT_DEFAULT,
                FONT_DEFAULT,
            ),
            layout,
            store,
            resources,
        }
    }
}

impl View<RouteTarget> for JumpMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = self.layout.background();
        els.extend(self.layout.jumpers());
        els.extend(self.layout.registration());
        els.push(Element::fillbox(1, 94, 116, 106, BG_LIST));
        els.extend(layout::header_elements(
            self.layout.langbase.lstr(18),
            11,
            80,
            FONT_HEADER,
            BG_ERASE,
        ));
        els.extend(self.menu.elements());
        els.extend(self.layout.footer());
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(1) => Some(self.start_competition(CupStyle::WorldCup)),
            Some(2) => Some(RouteTarget::CustomCupSetup),
            Some(3) => Some(self.start_competition(CupStyle::FourHills)),
            Some(0) => Some(RouteTarget::MainMenu),
            Some(n) => JUMP_MENU_ACTIONS.get(n - 1).and_then(|&a| a),
            _ => None,
        }
    }

    fn gpu_background(&self) -> engine::ui::BackgroundMode {
        engine::ui::BackgroundMode::MainPng
    }
}

impl JumpMenuView {
    fn start_competition(&self, style: CupStyle) -> RouteTarget {
        let profiles = self.store.profiles.borrow();
        let trainrounds = self.resources.save_manager.config.borrow().trainrounds;
        let comp = build_competition(
            style,
            &profiles,
            self.resources.player_names(),
            self.resources.hills.len(),
            trainrounds as usize,
        );
        drop(profiles);
        self.store.competition.start(comp);
        RouteTarget::CompetitionJump
    }
}

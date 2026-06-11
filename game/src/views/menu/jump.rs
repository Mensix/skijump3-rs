use std::cell::Cell;

use crate::competition::factory;
use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::components::screen;
use crate::gfx::palette::{BG_ERASE, BG_LIST, FONT_DEFAULT, FONT_GOLD, FONT_HEADER};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Component, Element, Event, View};

pub struct JumpMenuView {
    menu: Menu,
    layout: MainLayout,
    store: StoreRef,
    resources: ResourcesRef,
    show_team_warning: Cell<bool>,
}

const JUMP_MENU_ACTIONS: &[Option<RouteTarget>] = &[
    None,                        // 1 - WorldCup (special)
    None,                        // 2 - CustomCup (special)
    None,                        // 3 - FourHills (special)
    None,                        // 4 - TeamCup (special)
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
            show_team_warning: Cell::new(false),
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

        if self.show_team_warning.get() {
            els.extend(Self::team_warning_elements(&self.layout));
        }

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.show_team_warning.get() {
            if matches!(event, Event::Keyboard(_)) {
                self.show_team_warning.set(false);
                self.menu.set_show_box(true);
            }
            return None;
        }

        match self.menu.handle_event(&event) {
            Some(1) => Some(self.start_world_cup()),
            Some(2) => Some(RouteTarget::CustomCupSetup),
            Some(3) => Some(self.start_four_hills()),
            Some(4) => {
                let num_players = self.store.profiles().active_order.len();
                if num_players == 4 || num_players == 8 {
                    Some(self.start_team_cup())
                } else {
                    self.menu.set_show_box(false);
                    self.show_team_warning.set(true);
                    None
                }
            }
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
    fn start_world_cup(&self) -> RouteTarget {
        let profiles = self.store.profiles();
        let trainrounds = self.resources.save_manager.config.borrow().trainrounds;
        let comp = factory::world_cup(
            &profiles,
            self.resources.player_names(),
            self.resources.hills.len(),
            trainrounds as usize,
        );
        drop(profiles);
        self.store.start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn start_four_hills(&self) -> RouteTarget {
        let profiles = self.store.profiles();
        let trainrounds = self.resources.save_manager.config.borrow().trainrounds;
        let comp = factory::four_hills(
            &profiles,
            self.resources.player_names(),
            self.resources.hills.len(),
            trainrounds as usize,
        );
        drop(profiles);
        self.store.start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn start_team_cup(&self) -> RouteTarget {
        let profiles = self.store.profiles();
        let names = self.resources.player_names().to_vec();
        let namenumber = self.resources.save_manager.config.borrow().namenumber;
        let teams_def = self.resources.namesets.teams_for_config(namenumber).to_vec();
        let num_players = profiles.active_order.len();
        let human_teams = num_players / 4;
        let hill_count = self.resources.hills.len();
        drop(profiles);

        let comp = self.store.with_jump_rng_wind_mut(|rng, _| {
            factory::team_cup(&names, &teams_def, &self.store.profiles(), human_teams, hill_count, rng)
        });
        self.store.start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn team_warning_elements(layout: &MainLayout) -> Vec<Element> {
        let mut els = screen::modal_background(59, 59, 203, 83);
        let lang = &layout.langbase;
        els.push(Element::text(lang.lstr(261), 80, 72, FONT_GOLD, false));
        els.push(Element::text(lang.lstr(262), 80, 82, FONT_GOLD, false));
        els.push(Element::text(lang.lstr(263), 80, 92, FONT_GOLD, false));
        els.push(Element::text(lang.lstr(264), 80, 112, FONT_GOLD, false));
        els.push(Element::text(lang.lstr(265), 80, 122, FONT_GOLD, false));
        els
    }
}

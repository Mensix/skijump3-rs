use std::cell::Cell;

use crate::competition::factory;
use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::components::screen;
use crate::gfx::palette::{BG_ERASE, BG_LIST, FONT_DEFAULT, FONT_GOLD, FONT_HEADER};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::oxide::legacy::{commands_to_elements, event_from_ui, paint_elements};
use engine::oxide::{CommandBuffer, NavAction, PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Component, Element, Event, View};

pub struct JumpMenuView {
    menu: Menu,
    layout: MainLayout,
    store: StoreRef,
    resources: ResourcesRef,
    show_team_warning: Cell<bool>,
}

const JUMP_MENU_ACTIONS: &[Option<RouteTarget>] = &[
    None,                         // 1 - WorldCup (starts shared competition shell)
    None,                         // 2 - CustomCup (opens setup)
    None,                         // 3 - FourHills (starts shared competition shell)
    None,                         // 4 - TeamCup (starts shared competition shell)
    Some(RouteTarget::KothSetup), // 5 - King of the Hill
    Some(RouteTarget::Practice),  // 6 - Practice
    Some(RouteTarget::MainMenu),  // 7 - MainMenu
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

    fn legacy_elements(&self) -> Vec<Element> {
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
}

impl View<RouteTarget> for JumpMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut commands = CommandBuffer::new();
        let mut cx = PaintCx::new(&mut commands);
        Screen::paint(self, &mut cx);
        commands_to_elements(&commands)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let mut cx = ScreenEventCx::default();
        Screen::event(self, &mut cx, event.into());
        match cx.take_action() {
            NavAction::Navigate(route) => Some(route),
            NavAction::Back => Some(RouteTarget::Back),
            NavAction::Quit => Some(RouteTarget::Quit),
            NavAction::None => None,
        }
    }

    fn gpu_background(&self) -> engine::ui::BackgroundMode {
        engine::ui::BackgroundMode::MainPng
    }
}

impl Screen<RouteTarget> for JumpMenuView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = event_from_ui(event) else {
            return;
        };

        if self.show_team_warning.get() {
            if matches!(event, Event::Keyboard(_)) {
                self.show_team_warning.set(false);
                self.menu.set_show_box(true);
                cx.consume();
            }
            return;
        }

        match self.menu.handle_event(&event) {
            Some(1) => cx.navigate(self.start_world_cup()),
            Some(2) => cx.navigate(RouteTarget::CustomCupSetup),
            Some(3) => cx.navigate(self.start_four_hills()),
            Some(4) => {
                let num_players = self.store.profiles().active_order.len();
                if num_players == 4 || num_players == 8 {
                    cx.navigate(self.start_team_cup());
                } else {
                    self.menu.set_show_box(false);
                    self.show_team_warning.set(true);
                    cx.consume();
                }
            }
            Some(0) => cx.navigate(RouteTarget::MainMenu),
            Some(n) => {
                if let Some(route) = JUMP_MENU_ACTIONS.get(n - 1).and_then(|&a| a) {
                    cx.navigate(route);
                }
            }
            _ => {}
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        paint_elements(cx, &self.legacy_elements());
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
        let teams_def = self
            .resources
            .namesets
            .teams_for_config(namenumber)
            .to_vec();
        let num_players = profiles.active_order.len();
        let human_teams = num_players / 4;
        let hill_count = self.resources.hills.len();
        drop(profiles);

        let comp = self.store.with_jump_rng_wind_mut(|rng, _| {
            factory::team_cup(
                &names,
                &teams_def,
                &self.store.profiles(),
                human_teams,
                hill_count,
                rng,
            )
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

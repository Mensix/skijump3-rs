use std::cell::Cell;

use crate::competition::factory;
use crate::components::layout::MainLayout;
use crate::gfx::theme::{BG_DARK, BG_RED, BLACK, FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::store::{GameStateRef, ResourcesRef};
use engine::oxide::widgets::menu::PixelMenu;
use engine::oxide::Widget;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};

pub struct JumpMenuView {
    menu: PixelMenu,
    layout: MainLayout,
    store: GameStateRef,
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
    pub fn new(layout: MainLayout, store: GameStateRef, resources: ResourcesRef) -> Self {
        use engine::oxide::widgets::menu::MenuItem as OxideMenuItem;

        let items = vec![
            OxideMenuItem::new(1, ""),
            OxideMenuItem::new(2, ""),
            OxideMenuItem::new(3, ""),
            OxideMenuItem::new(4, ""),
            OxideMenuItem::new(5, ""),
            OxideMenuItem::new(6, ""),
            OxideMenuItem::new(0, "").with_y(12),
        ];
        Self {
            menu: PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
                .with_labels(false)
                .with_box(false),
            layout,
            store,
            resources,
            show_team_warning: Cell::new(false),
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        self.layout.background(cx);
        self.layout.jumpers(cx);
        self.layout.registration(cx);
        cx.fill((1, 94, 116, 106), BG_DARK);
        cx.fill((11, 80, 100, 6), BG_DARK);
        cx.text((11, 80), FONT_GOLD, self.layout.langbase.lstr(18));
        paint_jump_menu(cx, &self.menu, &self.layout);
        self.layout.footer(cx);
        if self.show_team_warning.get() {
            Self::paint_team_warning(cx, &self.layout);
        }
    }
}

impl Screen<RouteTarget> for JumpMenuView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = engine::oxide::widget::EventCx::default();

        if self.show_team_warning.get() {
            if matches!(event, UiEvent::KeyDown(_)) {
                self.show_team_warning.set(false);
                self.menu.set_show_box(true);
                cx.consume();
            }
            return;
        }

        match self.menu.event(&mut ecx, event) {
            Some(1) => cx.navigate(self.start_world_cup()),
            Some(2) => cx.navigate(RouteTarget::CustomCupSetup),
            Some(3) => cx.navigate(self.start_four_hills()),
            Some(4) => {
                let num_players = self.store.borrow().profiles.active_order.len();
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
        if ecx.is_consumed() {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

impl JumpMenuView {
    fn start_world_cup(&self) -> RouteTarget {
        let profiles = self.store.borrow().profiles.clone();
        let trainrounds = self.resources.save_manager.config.borrow().trainrounds;
        let comp = factory::world_cup(
            &profiles,
            self.resources.player_names(),
            self.resources.hills.len(),
            trainrounds as usize,
        );
        self.store.borrow_mut().start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn start_four_hills(&self) -> RouteTarget {
        let profiles = self.store.borrow().profiles.clone();
        let trainrounds = self.resources.save_manager.config.borrow().trainrounds;
        let comp = factory::four_hills(
            &profiles,
            self.resources.player_names(),
            self.resources.hills.len(),
            trainrounds as usize,
        );
        self.store.borrow_mut().start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn start_team_cup(&self) -> RouteTarget {
        let profiles = self.store.borrow().profiles.clone();
        let num_players = profiles.active_order.len();
        let human_teams = num_players / 4;
        let names = self.resources.player_names().to_vec();
        let namenumber = self.resources.save_manager.config.borrow().namenumber;
        let teams_def = self
            .resources
            .namesets
            .teams_for_config(namenumber)
            .to_vec();
        let hill_count = self.resources.hills.len();

        let comp = {
            let mut s = self.store.borrow_mut();
            factory::team_cup(
                &names,
                &teams_def,
                &profiles,
                human_teams,
                hill_count,
                &mut s.rng,
            )
        };
        self.store.borrow_mut().start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn paint_team_warning(cx: &mut PaintCx<'_>, layout: &MainLayout) {
        let lang = &layout.langbase;
        cx.fill((59, 59, 203, 83), BLACK);
        cx.pattern_fill((60, 60, 201, 81), BG_RED);
        cx.text((80, 72), FONT_GOLD, lang.lstr(261));
        cx.text((80, 82), FONT_GOLD, lang.lstr(262));
        cx.text((80, 92), FONT_GOLD, lang.lstr(263));
        cx.text((80, 112), FONT_GOLD, lang.lstr(264));
        cx.text((80, 122), FONT_GOLD, lang.lstr(265));
    }
}

fn paint_jump_menu(cx: &mut PaintCx<'_>, menu: &PixelMenu, layout: &MainLayout) {
    let y_offsets = [0, 0, 0, 0, 0, 0, 12];
    for (i, label) in [27, 28, 29, 30, 31, 32, 33].iter().enumerate() {
        let num = if i == 6 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + y_offsets[i];
        cx.text(
            (11, y),
            FONT_BODY,
            format!("{} - {}", num, layout.langbase.lstr(*label)),
        );
    }
    let selected = menu.selected().min(y_offsets.len().saturating_sub(1));
    let y = 94 + (selected as i32) * 12 + y_offsets[selected];
    cx.stroke((5, y, 109, 13), FONT_BODY);
}

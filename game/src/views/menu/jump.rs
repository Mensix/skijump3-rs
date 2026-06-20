use std::cell::Cell;

use crate::competition::factory;
use crate::components::layout::MainLayout;
use crate::gfx::theme::{BG_DARK, BG_RED, BLACK, FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use engine::oxide::widgets::menu::PixelMenu;
use engine::oxide::Widget;
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx, UiEvent};

pub struct JumpMenuView {
    menu: PixelMenu,
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
    Some(RouteTarget::LoadCup),   // 7 - Load Cup
    Some(RouteTarget::MainMenu),  // 0 - Back to Main Menu
];

impl JumpMenuView {
    #[must_use]
    pub fn new(resources: ResourcesRef) -> Self {
        use engine::oxide::widgets::menu::MenuItem as OxideMenuItem;

        let items = vec![
            OxideMenuItem::new(0, ""),
            OxideMenuItem::new(1, ""),
            OxideMenuItem::new(2, ""),
            OxideMenuItem::new(3, ""),
            OxideMenuItem::new(4, ""),
            OxideMenuItem::new(5, ""),
            OxideMenuItem::new(6, ""),
            OxideMenuItem::new(7, "").with_y(12),
        ];
        Self {
            menu: PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
                .with_labels(false)
                .with_box(false),
            resources,
            show_team_warning: Cell::new(false),
        }
    }
}

impl GameScreen for JumpMenuView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = engine::oxide::widget::EventCx::default();

        if self.show_team_warning.get() {
            if matches!(event, UiEvent::KeyDown(_)) {
                self.show_team_warning.set(false);
                self.menu.set_show_box(true);
                nav.consume();
            }
            return;
        }

        match self.menu.event(&mut ecx, event) {
            Some(0) => nav.navigate(self.start_world_cup(cx.state)),
            Some(1) => nav.navigate(RouteTarget::CustomCupSetup),
            Some(2) => nav.navigate(self.start_four_hills(cx.state)),
            Some(3) => {
                let num_players = cx.state.profiles.active_order.len();
                if num_players == 4 || num_players == 8 {
                    nav.navigate(self.start_team_cup(cx.state));
                } else {
                    self.menu.set_show_box(false);
                    self.show_team_warning.set(true);
                    nav.consume();
                }
            }
            Some(6) => nav.navigate(RouteTarget::LoadCup),
            Some(7) => nav.navigate(RouteTarget::MainMenu),
            Some(n) => {
                if let Some(route) = JUMP_MENU_ACTIONS.get(n).and_then(|&a| a) {
                    nav.navigate(route);
                }
            }
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        cx.layout.background(paint);
        cx.layout.jumpers(paint, &cx.state.profiles);
        cx.layout.registration(paint);
        paint.fill((1, 94, 116, 106), BG_DARK);
        paint.text((11, 80), FONT_GOLD, cx.layout.langbase.lstr(18));
        paint_jump_menu(paint, &self.menu, cx.layout);
        cx.layout.footer(paint);
        if self.show_team_warning.get() {
            Self::paint_team_warning(paint, cx.layout);
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

impl JumpMenuView {
    fn start_world_cup(&self, state: &mut GameState) -> RouteTarget {
        let profiles = state.profiles.clone();
        let config = state.config.clone();
        let comp = factory::world_cup(
            &profiles,
            self.resources.player_names(config.name_set_index as usize),
            self.resources.hills.len(),
            config.training_rounds as usize,
            config.unique_computer_names != 0,
            config.ko_system != 0,
        );
        state.start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn start_four_hills(&self, state: &mut GameState) -> RouteTarget {
        let profiles = state.profiles.clone();
        let config = state.config.clone();
        let comp = factory::four_hills(
            &profiles,
            self.resources.player_names(config.name_set_index as usize),
            self.resources.hills.len(),
            config.training_rounds as usize,
            config.unique_computer_names != 0,
            config.ko_system != 0,
        );
        state.start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn start_team_cup(&self, state: &mut GameState) -> RouteTarget {
        let profiles = state.profiles.clone();
        let num_players = profiles.active_order.len();
        let human_teams = num_players / 4;
        let names = self
            .resources
            .player_names(state.config.name_set_index as usize)
            .to_vec();
        let name_set_index = state.config.name_set_index as usize;
        let teams_def = self
            .resources
            .namesets
            .teams_for_config(name_set_index as i32)
            .to_vec();
        let hill_count = self.resources.hills.len();

        let comp = {
            factory::team_cup(
                &names,
                &teams_def,
                &profiles,
                human_teams,
                hill_count,
                &mut state.rng,
            )
        };
        state.start_active(comp);
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
    let y_offsets = [0, 0, 0, 0, 0, 0, 0, 12];
    for (i, label) in [27, 28, 29, 30, 31, 32, 520, 33].iter().enumerate() {
        let num = if i == 7 { 0 } else { i + 1 };
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

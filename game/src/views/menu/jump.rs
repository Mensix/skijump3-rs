use super::paint_numbered_menu;
use crate::competition::factory;
use crate::competition::types::CupStyle;
use crate::components::layout::MainLayout;
use crate::gfx::theme::{BG_DARK, BG_RED, BLACK, FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::{
    EventCx, Key, MenuAction, MenuItem, PixelMenu, ScreenBackground, ScreenEventCx, UiEvent,
};
use engine::audio::Beep;

pub struct JumpMenuView {
    menu: PixelMenu,
    resources: ResourcesRef,
    show_team_warning: bool,
}

const JUMP_MENU_ACTIONS: &[Option<RouteTarget>] = &[
    None,
    None,
    None,
    None,
    Some(RouteTarget::KothSetup),
    Some(RouteTarget::Practice),
    Some(RouteTarget::MainMenu),
];

impl JumpMenuView {
    pub fn new(resources: ResourcesRef) -> Self {
        let items = vec![
            MenuItem::new(0, ""),
            MenuItem::new(1, ""),
            MenuItem::new(2, ""),
            MenuItem::new(3, ""),
            MenuItem::new(4, ""),
            MenuItem::new(5, ""),
            MenuItem::new(6, ""),
        ];
        Self {
            menu: PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
                .with_labels(false)
                .with_box(false),
            resources,
            show_team_warning: false,
        }
    }
}

impl GameScreen for JumpMenuView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = EventCx::default();

        if self.show_team_warning {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                self.show_team_warning = false;
                self.menu.set_show_box(true);
                nav.consume();
            }
            return;
        }

        if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) {
            nav.back();
            return;
        }
        if matches!(event, UiEvent::Text('0')) {
            nav.navigate(RouteTarget::MainMenu);
            nav.consume();
            return;
        }

        match self.menu.event_action(&mut ecx, event) {
            Some(MenuAction::Item(0)) => nav.navigate(self.start_world_cup(cx.state)),
            Some(MenuAction::Item(1)) => nav.navigate(RouteTarget::CustomCupSetup),
            Some(MenuAction::Item(2)) => nav.navigate(self.start_four_hills(cx.state)),
            Some(MenuAction::Item(3)) => {
                let num_players = cx.state.profiles.active_order.len();
                if num_players == 4 || num_players == 8 {
                    nav.navigate(self.start_team_cup(cx.state));
                } else {
                    request_invalid_team_roster_warning(cx.state);
                    self.menu.set_show_box(false);
                    self.show_team_warning = true;
                    nav.consume();
                }
            }
            Some(MenuAction::Item(n)) => {
                if let Some(route) = JUMP_MENU_ACTIONS.get(n).and_then(|a| a.clone()) {
                    nav.navigate(route);
                }
            }
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        let lang = &cx.layout.langbase;
        cx.layout.background(paint);
        cx.layout.jumpers(paint, &cx.state.profiles);
        cx.layout.registration(paint);
        paint.fill((1, 94, 116, 106), BG_DARK);
        paint.text((11, 80), FONT_GOLD, lang.tr(18));
        paint_jump_menu(paint, &self.menu, cx.layout);
        cx.layout.footer(paint);
        if self.show_team_warning {
            Self::paint_team_warning(paint, cx.layout);
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn request_invalid_team_roster_warning(state: &mut GameState) {
    state.request_beep(Beep::Type1);
}

impl JumpMenuView {
    fn start_cup(&self, state: &mut GameState, style: CupStyle) -> RouteTarget {
        let profiles = state.profiles.clone();
        let config = state.config.clone();
        let comp = factory::cup(
            style,
            &profiles,
            self.resources.player_names(config.name_set_index as usize),
            self.resources.hills.original_count(),
            config.training_rounds as usize,
            config.unique_computer_names != 0,
            config.ko_system != 0,
        );
        state.start_active(comp);
        RouteTarget::CompetitionJump
    }

    fn start_world_cup(&self, state: &mut GameState) -> RouteTarget {
        self.start_cup(state, CupStyle::WorldCup)
    }

    fn start_four_hills(&self, state: &mut GameState) -> RouteTarget {
        self.start_cup(state, CupStyle::FourHills)
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
        let hill_count = self.resources.hills.original_count();

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

    fn paint_team_warning(cx: &mut dyn UiCanvas, layout: &MainLayout) {
        let lang = &layout.langbase;
        cx.fill((59, 59, 203, 83), BLACK);
        cx.pattern_fill((60, 60, 201, 81), BG_RED);
        cx.text((80, 72), FONT_GOLD, lang.tr(261));
        cx.text((80, 82), FONT_GOLD, lang.tr(262));
        cx.text((80, 92), FONT_GOLD, lang.tr(263));
        cx.text((80, 112), FONT_GOLD, lang.tr(264));
        cx.text((80, 122), FONT_GOLD, lang.tr(265));
    }
}

fn paint_jump_menu(cx: &mut dyn UiCanvas, menu: &PixelMenu, layout: &MainLayout) {
    paint_numbered_menu(
        cx,
        &layout.langbase,
        [27, 28, 29, 30, 31, 32, 33],
        [0, 0, 0, 0, 0, 0, 0],
        Some(menu.selected()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_team_roster_warning_requests_type_one_beep() {
        let mut state = GameState::default();
        state.config.sound_effects = 1;

        request_invalid_team_roster_warning(&mut state);

        assert_eq!(state.drain_audio_cues().collect::<Vec<_>>(), [Beep::Type1]);
    }
}

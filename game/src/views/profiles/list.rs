use crate::components::modal::alert_prompt;
use crate::data::profile::Profile;
use crate::gfx::theme::FONT_BODY;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use engine::oxide::input::Key;
use engine::oxide::widget::EventCx;
use engine::oxide::widgets::menu::{MenuAction, PixelMenu};
use engine::oxide::{PaintCx, ScreenEventCx, UiEvent, Widget};

use super::actions::{
    commit_text_input, handle_edit_enter, handle_event_color_select, handle_event_replace_select,
    handle_event_text_input, handle_list_delete, handle_list_enter, save_players,
};
use super::render::{
    draw_color_select, draw_empty_edit, draw_help, draw_list, draw_profile, draw_replace_select,
    draw_screen_base,
};
use super::state::{ColorField, Mode, Pending, TextField};

pub struct ProfilesView {
    pub(super) resources: ResourcesRef,
    pub(super) save_manager: SaveRef,
    pub(super) selected: usize,
    pub(super) mode: Mode,
    pub(super) menu: PixelMenu,
    pub(super) edit_menu: PixelMenu,
    pub(super) confirm_delete: Option<usize>,
    pub(super) confirm_reset: Option<usize>,
}

impl ProfilesView {
    pub fn new(resources: ResourcesRef, save_manager: SaveRef, state: &GameState) -> Self {
        let lang = &resources.langbase;
        let np = state.profiles.num_profiles();
        let count = np + usize::from(state.profiles.has_slot());
        let menu = PixelMenu::new(34, 10, 123, 8, vec![], FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false)
            .with_return_index(true)
            .trailing(lang.tr(33), 0);
        let edit_menu = PixelMenu::new(162, 10, 155, 8, vec![], FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false)
            .with_return_index(true)
            .trailing(lang.tr(33), 0);
        let mut view = Self {
            menu,
            edit_menu,
            resources,
            save_manager,
            selected: 0,
            mode: Mode::List,
            confirm_delete: None,
            confirm_reset: None,
        };
        view.menu.set_index_items(count);
        view.edit_menu.set_index_items(8);
        view
    }

    fn rebuild_list_menu(&mut self, state: &GameState) {
        let np = state.profiles.num_profiles();
        let count = np + usize::from(state.profiles.has_slot());
        self.menu.set_index_items(count);
        self.selected = self.menu.selected();
    }

    pub(super) fn enter_edit_mode(&mut self, _state: &GameState, profile: usize, initial_selection: usize) {
        self.edit_menu.set_index_items(8);
        self.edit_menu.set_selected(initial_selection);
        self.mode = Mode::Edit { profile };
    }

    pub(super) const fn row_y(row: usize) -> i32 {
        (row * 8 + 4) as i32
    }

    pub(super) const fn col_y(row: usize) -> i32 {
        match row {
            0..=9 => Self::row_y(row),
            10..=15 => (row * 8 + 10) as i32,
            _ => (row * 16 - 118) as i32,
        }
    }

    pub(super) fn entries(&self, state: &GameState) -> usize {
        let np = state.profiles.num_profiles();
        if state.profiles.has_slot() {
            np + 1
        } else {
            np
        }
    }

    pub(super) fn unique_default_profile(&self, state: &GameState) -> Profile {
        let mut profile = Profile::default();
        let mut counter = 2;
        while state
            .profiles
            .profiles
            .iter()
            .any(|p| p.name == profile.name)
        {
            profile.name = format!("SKI JUMPER {counter}");
            counter += 1;
        }
        profile
    }

    pub(super) fn menu_selected(&self) -> Option<usize> {
        match self.mode {
            Mode::Edit { .. } => Some(self.edit_menu.selected()),
            _ => None,
        }
    }

    pub(super) fn active_profile(&self, state: &GameState) -> Option<usize> {
        match self.mode {
            Mode::Edit { profile }
            | Mode::TextInput { profile, .. }
            | Mode::ColorSelect { profile, .. }
            | Mode::ReplaceSelect { profile, .. } => Some(profile),
            _ => (self.selected < state.profiles.num_profiles()).then_some(self.selected),
        }
    }

    fn paint_content(&self, state: &GameState, cx: &mut PaintCx<'_>) {
        let lang = &self.resources.langbase;
        draw_screen_base(self, cx);

        if let Some(profile) = self.active_profile(state) {
            let edit_phase = !matches!(self.mode, Mode::List);
            draw_profile(self, state, cx, profile, edit_phase);
            if matches!(self.mode, Mode::List) {
                draw_help(self, state, cx, Some(profile));
            }
        } else {
            draw_empty_edit(cx);
            draw_help(self, state, cx, None);
        }

        draw_list(self, state, cx);

        match &self.mode {
            Mode::TextInput { input, .. } => input.paint(cx),
            Mode::ColorSelect {
                selector,
                color_x,
                color_y,
                color_max,
                color_suit,
                ..
            } => draw_color_select(cx, selector, *color_x, *color_y, *color_max, *color_suit),
            Mode::ReplaceSelect { selector, .. } => {
                draw_replace_select(self, state, cx, selector)
            }
            _ => {}
        }

        if let Some(profile) = self.confirm_delete {
            let name = &state.profiles.profiles[profile].name;
            alert_prompt(cx, format!("{} {}", lang.tr(328), name), lang.tr(193), true);
        } else if self.confirm_reset.is_some() {
            alert_prompt(cx, lang.tr(329), lang.tr(193), true);
        }
    }

    fn handle_input(
        &mut self,
        ecx: &mut EventCx,
        state: &mut GameState,
        event: UiEvent,
    ) -> Option<RouteTarget> {
        if self.confirm_delete.is_some() {
            match event {
                UiEvent::Text('y' | 'Y') => {
                    let profile = self.confirm_delete.take().unwrap();
                    state.profiles.remove_profile(profile);
                    let np = state.profiles.num_profiles();
                    if self.selected >= np {
                        self.selected = np.saturating_sub(1);
                    }
                    self.mode = Mode::List;
                    self.rebuild_list_menu(state);
                    save_players(self, state);
                    ecx.consume();
                }
                UiEvent::Text('n' | 'N') => {
                    self.confirm_delete = None;
                    ecx.consume();
                }
                _ => {}
            }
            return None;
        }
        if self.confirm_reset.is_some() {
            match event {
                UiEvent::Text('y' | 'Y') => {
                    let profile = self.confirm_reset.take().unwrap();
                    let name = state.profiles.profiles[profile].name.clone();
                    let reset = Profile {
                        name,
                        ..Default::default()
                    };
                    state.profiles.profiles[profile] = reset;
                    self.enter_edit_mode(state, profile, 7);
                    save_players(self, state);
                    ecx.consume();
                }
                UiEvent::Text('n' | 'N') => {
                    let profile = self.confirm_reset.take().unwrap();
                    self.enter_edit_mode(state, profile, 7);
                    ecx.consume();
                }
                _ => {}
            }
            return None;
        }

        if matches!(self.mode, Mode::List) {
            match event {
                UiEvent::KeyDown(Key::Escape) => return Some(RouteTarget::Back),
                UiEvent::KeyDown(Key::Delete | Key::Backspace) => {
                    self.selected = self.menu.selected();
                    handle_list_delete(self, state);
                    ecx.consume();
                }
                _ => {
                    let action = self.menu.event_action(ecx, event);
                    self.selected = self.menu.selected();
                    if action.is_some() {
                        if let Some(route) = handle_list_enter(self, state) {
                            return Some(route);
                        }
                    }
                }
            }
            return None;
        }

        let mut pending = None;
        match &mut self.mode {
            Mode::Edit { profile } => match event {
                UiEvent::KeyDown(Key::Escape) => {
                    self.mode = Mode::List;
                    self.rebuild_list_menu(state);
                    ecx.consume();
                }
                _ => {
                    match self.edit_menu.event_action(ecx, event) {
                        Some(MenuAction::Item(action)) => {
                            pending = Some(Pending::EditEnter(*profile, action));
                        }
                        Some(MenuAction::Trailing) => {
                            self.mode = Mode::List;
                            self.rebuild_list_menu(state);
                        }
                        None => {}
                    }
                }
            },
            Mode::TextInput { .. } => {
                pending = handle_event_text_input(&mut self.mode, event);
            }
            Mode::ColorSelect { .. } => {
                pending = handle_event_color_select(&mut self.mode, state, event);
            }
            Mode::ReplaceSelect { .. } => {
                pending = handle_event_replace_select(&mut self.mode, state, event);
            }
            Mode::List => {}
        }

        match pending {
            Some(Pending::EditEnter(profile, selected)) => {
                handle_edit_enter(self, state, profile, selected);
            }
            Some(Pending::TextCommit(profile, field, value)) => {
                commit_text_input(self, state, profile, field, &value);
                save_players(self, state);
            }
            Some(Pending::TextCancel(profile, field)) => {
                self.enter_edit_mode(state, profile, match field {
                    TextField::Name => 0,
                    TextField::RealName => 1,
                });
            }
            Some(Pending::ColorCommit(profile, field)) => {
                save_players(self, state);
                self.enter_edit_mode(state, profile, match field {
                    ColorField::Suit => 2,
                    ColorField::Ski => 3,
                });
            }
            Some(Pending::ColorCancel(profile, field)) => {
                self.enter_edit_mode(state, profile, match field {
                    ColorField::Suit => 2,
                    ColorField::Ski => 3,
                });
            }
            Some(Pending::ReplaceCommit(profile)) => {
                save_players(self, state);
                self.enter_edit_mode(state, profile, 4);
            }
            Some(Pending::ReplaceCancel(profile)) => {
                self.enter_edit_mode(state, profile, 4);
            }
            None => {}
        }
        None
    }
}

impl GameScreen for ProfilesView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        let mut ecx = EventCx::default();
        if let Some(route) = self.handle_input(&mut ecx, cx.state, event) {
            if route == RouteTarget::Back {
                nav.back();
            } else {
                nav.navigate(route);
            }
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(cx.state, paint);
    }
}

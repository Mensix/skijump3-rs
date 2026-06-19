use crate::components::page_nav::cycle_index;
use crate::data::profile::Profile;
use crate::gfx::jumper_colors::{ski_color, suit_color_shade};
use crate::gfx::theme::{BG_RED, BLACK, FILL_GRAY, FONT_BODY};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{GameStateRef, ResourcesRef};
use crate::text::layout::{lstr, replace_display_name};
use engine::oxide::input::Key;
use engine::oxide::widget::EventCx;
use engine::oxide::widgets::confirm::{ConfirmDialog as OxideConfirmDialog, ConfirmMessage};
use engine::oxide::widgets::selector::{NumericSelector, SelectorMessage};
use engine::oxide::widgets::text_input::{TextInput as OxideTextInput, TextInputMessage};
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent, Widget};

use super::actions::{
    apply_question, commit_text_input, handle_edit_enter, handle_list_delete, handle_list_enter,
    save_players,
};
use super::render::{draw_empty_edit, draw_help, draw_list, draw_profile, draw_screen_base};

const EDIT_MENU_ITEMS: usize = 9;
pub(super) const REPLACE_MAX: usize = 65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextField {
    Name,
    RealName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ColorField {
    Suit,
    Ski,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QuestionAction {
    DeleteProfile(usize),
    ResetProfile(usize),
}

#[derive(Debug)]
pub(super) enum Mode {
    List,
    Edit {
        profile: usize,
        selected: usize,
    },
    TextInput {
        profile: usize,
        field: TextField,
        input: OxideTextInput,
    },
    ColorSelect {
        profile: usize,
        field: ColorField,
        selector: NumericSelector,
        color_x: i32,
        color_y: i32,
        color_max: usize,
        color_suit: bool,
    },
    ReplaceSelect {
        profile: usize,
        selector: NumericSelector,
    },
    Question {
        action: QuestionAction,
        dialog: OxideConfirmDialog,
    },
}

pub struct ProfilesView {
    pub(super) resources: ResourcesRef,
    pub(super) store: GameStateRef,
    pub(super) save_manager: SaveRef,
    pub(super) selected: usize,
    pub(super) mode: Mode,
}

pub(super) enum Pending {
    EditEnter(usize, usize),
    TextCommit(usize, TextField, String),
    TextCancel(usize, TextField),
    ColorCommit(usize, ColorField),
    ColorCancel(usize, ColorField),
    ReplaceCommit(usize),
    ReplaceCancel(usize),
    QuestionYes(QuestionAction),
    QuestionNo(QuestionAction),
}

impl ProfilesView {
    pub const fn new(resources: ResourcesRef, store: GameStateRef, save_manager: SaveRef) -> Self {
        Self {
            resources,
            store,
            save_manager,
            selected: 0,
            mode: Mode::List,
        }
    }

    pub(super) const fn y_for(row: usize) -> i32 {
        (row * 8 + 4) as i32
    }

    pub(super) const fn col_y(row: usize) -> i32 {
        match row {
            0..=9 => Self::y_for(row),
            10..=15 => (row * 8 + 10) as i32,
            _ => (row * 16 - 118) as i32,
        }
    }

    pub(super) fn entries(&self) -> usize {
        let store = &self.store.borrow().profiles;
        let np = store.num_profiles();
        if store.has_slot() {
            np + 1
        } else {
            np
        }
    }

    pub(super) fn unique_default_profile(&self) -> Profile {
        let store = &self.store.borrow().profiles;
        let mut profile = Profile::default();
        let mut counter = 2;
        while store.profiles.iter().any(|p| p.name == profile.name) {
            profile.name = format!("SKI JUMPER {counter}");
            counter += 1;
        }
        profile
    }

    pub(super) const fn menu_selected(&self) -> Option<usize> {
        match self.mode {
            Mode::Edit { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub(super) fn active_profile(&self) -> Option<usize> {
        match self.mode {
            Mode::Edit { profile, .. }
            | Mode::TextInput { profile, .. }
            | Mode::ColorSelect { profile, .. }
            | Mode::ReplaceSelect { profile, .. } => Some(profile),
            _ => (self.selected < self.store.borrow().profiles.num_profiles())
                .then_some(self.selected),
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        draw_screen_base(self, cx);

        if let Some(profile) = self.active_profile() {
            let edit_phase = !matches!(self.mode, Mode::List | Mode::Question { .. });
            draw_profile(self, cx, profile, edit_phase);
            if matches!(self.mode, Mode::List) {
                draw_help(self, cx, Some(profile));
            }
        } else {
            draw_empty_edit(cx);
            draw_help(self, cx, None);
        }

        draw_list(self, cx);

        match &self.mode {
            Mode::TextInput { input, .. } => input.paint(cx),
            Mode::ColorSelect {
                selector,
                color_x,
                color_y,
                color_max,
                color_suit,
                ..
            } => {
                let value = selector.value();
                let width = 31i32;
                cx.fill(
                    (*color_x, *color_y, width, 5 + ((*color_max + 1) as i32 * 8)),
                    BLACK,
                );
                cx.stroke(
                    (*color_x, *color_y, width, 5 + ((*color_max + 1) as i32 * 8)),
                    BG_RED,
                );
                for v in 0..=*color_max {
                    let by = *color_y + 4 + v as i32 * 8;
                    let fill = if *color_suit {
                        suit_color_shade(v, 0)
                    } else {
                        ski_color(v)
                    };
                    cx.fill((*color_x + 6, by, 19, 5), fill);
                    if *color_suit {
                        cx.stroke((*color_x + 6, by, 19, 5), suit_color_shade(v, 2));
                    }
                }
                cx.stroke(
                    (*color_x + 3, *color_y + 2 + value as i32 * 8, 25, 9),
                    FONT_BODY,
                );
            }
            Mode::ReplaceSelect { selector, .. } => {
                let value = selector.value();
                let x = self.resources.font.string_width("Replace:") as i32 + 170;
                cx.fill((x - 2, 43, 320 - x, 8), FILL_GRAY);
                if value > 0 {
                    if value
                        <= self
                            .resources
                            .player_names(self.store.borrow().config.name_set_index as usize)
                            .len()
                    {
                        let n = replace_display_name(
                            value,
                            self.resources
                                .player_names(self.store.borrow().config.name_set_index as usize),
                            &self.resources.font,
                            x,
                        );
                        cx.text((x, 44), FONT_BODY, n);
                        cx.right_text((316, 44), FONT_BODY, format!("#{value}"));
                    } else {
                        cx.text((x, 44), FONT_BODY, format!("#{value}"));
                    }
                } else {
                    cx.text(
                        (x, 44),
                        FONT_BODY,
                        lstr(&self.resources.langbase, 9, "None"),
                    );
                }
            }
            Mode::Question { dialog, .. } => dialog.paint(cx),
            _ => {}
        }
    }

    fn handle_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        if matches!(self.mode, Mode::List) {
            match event {
                UiEvent::KeyDown(Key::Up) => {
                    let total = self.entries() + 1;
                    self.selected = cycle_index(self.selected, total, -1);
                }
                UiEvent::KeyDown(Key::Down) => {
                    let total = self.entries() + 1;
                    self.selected = cycle_index(self.selected, total, 1);
                }
                UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                    if let Some(route) = handle_list_enter(self) {
                        return Some(route);
                    }
                }
                UiEvent::KeyDown(Key::Delete | Key::Backspace) => {
                    handle_list_delete(self);
                }
                UiEvent::KeyDown(Key::Escape) => return Some(RouteTarget::Back),
                UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::Quit | UiEvent::Tick => {}
            }
            return None;
        }

        let mut pending = None;
        match &mut self.mode {
            Mode::Edit { profile, selected } => match event {
                UiEvent::KeyDown(Key::Up) => {
                    *selected = if *selected == 0 {
                        EDIT_MENU_ITEMS - 1
                    } else {
                        *selected - 1
                    }
                }
                UiEvent::KeyDown(Key::Down) => *selected = (*selected + 1) % EDIT_MENU_ITEMS,
                UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                    if *selected < 8 {
                        pending = Some(Pending::EditEnter(*profile, *selected));
                    } else {
                        self.mode = Mode::List;
                    }
                }
                UiEvent::KeyDown(Key::Escape) => *selected = EDIT_MENU_ITEMS - 1,
                UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::Quit | UiEvent::Tick => {}
            },
            Mode::TextInput {
                profile,
                field,
                input,
            } => {
                let mut ecx = EventCx::default();
                if let Some(action) = input.event(&mut ecx, event) {
                    pending = Some(match action {
                        TextInputMessage::Commit(value) => {
                            Pending::TextCommit(*profile, *field, value)
                        }
                        TextInputMessage::Cancel => Pending::TextCancel(*profile, *field),
                    });
                }
            }
            Mode::ColorSelect {
                profile,
                field,
                selector,
                ..
            } => {
                let mut ecx = EventCx::default();
                if let Some(action) = selector.event(&mut ecx, event) {
                    pending = Some(match action {
                        SelectorMessage::Commit(value) => {
                            let mut state = self.store.borrow_mut();
                            match field {
                                ColorField::Suit => {
                                    state.profiles.profiles[*profile].suit_color = value
                                }
                                ColorField::Ski => {
                                    state.profiles.profiles[*profile].ski_color = value
                                }
                            }
                            Pending::ColorCommit(*profile, *field)
                        }
                        SelectorMessage::Cancel => Pending::ColorCancel(*profile, *field),
                    });
                }
            }
            Mode::ReplaceSelect { profile, selector } => {
                let mut ecx = EventCx::default();
                if let Some(action) = selector.event(&mut ecx, event) {
                    match action {
                        SelectorMessage::Commit(value) => {
                            self.store.borrow_mut().profiles.profiles[*profile].replace = value;
                            pending = Some(Pending::ReplaceCommit(*profile));
                        }
                        SelectorMessage::Cancel => {
                            pending = Some(Pending::ReplaceCancel(*profile));
                        }
                    }
                }
            }
            Mode::Question { action, dialog } => {
                let mut ecx = EventCx::default();
                if let Some(result) = dialog.event(&mut ecx, event) {
                    pending = Some(match result {
                        ConfirmMessage::Yes => Pending::QuestionYes(*action),
                        ConfirmMessage::No => Pending::QuestionNo(*action),
                    });
                }
            }
            Mode::List => {}
        }

        match pending {
            Some(Pending::EditEnter(profile, selected)) => {
                handle_edit_enter(self, profile, selected);
            }
            Some(Pending::TextCommit(profile, field, value)) => {
                commit_text_input(self, profile, field, &value);
                save_players(self);
            }
            Some(Pending::TextCancel(profile, field)) => {
                self.mode = Mode::Edit {
                    profile,
                    selected: match field {
                        TextField::Name => 0,
                        TextField::RealName => 1,
                    },
                };
            }
            Some(Pending::ColorCommit(profile, field)) => {
                save_players(self);
                self.mode = Mode::Edit {
                    profile,
                    selected: match field {
                        ColorField::Suit => 2,
                        ColorField::Ski => 3,
                    },
                };
            }
            Some(Pending::ColorCancel(profile, field)) => {
                self.mode = Mode::Edit {
                    profile,
                    selected: match field {
                        ColorField::Suit => 2,
                        ColorField::Ski => 3,
                    },
                };
            }
            Some(Pending::ReplaceCommit(profile)) => {
                save_players(self);
                self.mode = Mode::Edit {
                    profile,
                    selected: 4,
                };
            }
            Some(Pending::ReplaceCancel(profile)) => {
                self.mode = Mode::Edit {
                    profile,
                    selected: 4,
                }
            }
            Some(Pending::QuestionYes(action)) => {
                apply_question(self, action);
                save_players(self);
            }
            Some(Pending::QuestionNo(action)) => {
                self.mode = match action {
                    QuestionAction::DeleteProfile(_) => Mode::List,
                    QuestionAction::ResetProfile(profile) => Mode::Edit {
                        profile,
                        selected: 7,
                    },
                };
            }
            None => {}
        }
        None
    }
}

impl Screen<RouteTarget> for ProfilesView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if matches!(event, UiEvent::Quit | UiEvent::Tick) {
            return;
        }
        if let Some(route) = self.handle_input(event) {
            if route == RouteTarget::Back {
                cx.back();
            } else {
                cx.navigate(route);
            }
        } else {
            cx.consume();
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

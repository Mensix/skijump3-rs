use crate::components::confirm_dialog::{ConfirmAction, ConfirmDialog};
use crate::components::text_input::{TextInput, TextInputAction};
use crate::components::value_selector::{ValueSelector, ValueSelectorAction};
use crate::data::profile::{Profile, NUM_SKIS, NUM_SUITS};
use crate::gfx::palette::{
    apply_ski_palette, apply_ski_palette_at, apply_suit_palette, apply_suit_palette_at,
    FONT_DEFAULT,
};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::layout::{lstr, replace_display_name};
use engine::palette::Palette;
use engine::ui::{Component, Element, Event, Key, View};

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
        input: TextInput,
    },
    ColorSelect {
        profile: usize,
        field: ColorField,
        selector: ValueSelector,
    },
    ReplaceSelect {
        profile: usize,
        selector: ValueSelector,
    },
    Question {
        action: QuestionAction,
        dialog: ConfirmDialog,
    },
}

pub struct ProfilesView {
    pub(super) resources: ResourcesRef,
    pub(super) store: StoreRef,
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
    pub const fn new(resources: ResourcesRef, store: StoreRef, save_manager: SaveRef) -> Self {
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
        let store = self.store.profiles.borrow();
        let np = store.num_profiles();
        if store.has_slot() {
            np + 1
        } else {
            np
        }
    }

    pub(super) fn unique_default_profile(&self) -> Profile {
        let store = self.store.profiles.borrow();
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

    fn active_profile(&self) -> Option<usize> {
        match self.mode {
            Mode::Edit { profile, .. }
            | Mode::TextInput { profile, .. }
            | Mode::ColorSelect { profile, .. }
            | Mode::ReplaceSelect { profile, .. } => Some(profile),
            _ => (self.selected < self.store.profiles.borrow().num_profiles())
                .then_some(self.selected),
        }
    }
}

impl View<RouteTarget> for ProfilesView {
    fn elements(&self) -> Vec<Element> {
        let mut els = Vec::new();
        draw_screen_base(self, &mut els);

        if let Some(profile) = self.active_profile() {
            let edit_phase = !matches!(self.mode, Mode::List | Mode::Question { .. });
            draw_profile(self, &mut els, profile, edit_phase);
            if matches!(self.mode, Mode::List) {
                draw_help(self, &mut els, Some(profile));
            }
        } else {
            draw_empty_edit(&mut els);
            draw_help(self, &mut els, None);
        }

        draw_list(self, &mut els);

        match &self.mode {
            Mode::TextInput { input, .. } => els.extend(input.elements()),
            Mode::ColorSelect { selector, .. } => els.extend(selector.elements()),
            Mode::ReplaceSelect { selector, .. } => {
                let value = selector.value();
                let x = self.resources.font.string_width("Replace:") as i32 + 170;
                els.push(Element::fillbox(x - 2, 43, 320 - x, 8, 245));
                if value > 0 {
                    if value <= self.resources.player_names.len() {
                        let n = replace_display_name(
                            value,
                            &self.resources.player_names,
                            &self.resources.font,
                            x,
                        );
                        els.push(Element::text(n, x, 44, FONT_DEFAULT, false));
                        els.push(Element::text(
                            format!("#{value}"),
                            316,
                            44,
                            FONT_DEFAULT,
                            true,
                        ));
                    } else {
                        els.push(Element::text(
                            format!("#{value}"),
                            x,
                            44,
                            FONT_DEFAULT,
                            false,
                        ));
                    }
                } else {
                    els.push(Element::text(
                        lstr(&self.resources.langbase, 9, "None"),
                        x,
                        44,
                        FONT_DEFAULT,
                        false,
                    ));
                }
            }
            Mode::Question { dialog, .. } => els.extend(dialog.elements()),
            _ => {}
        }

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if matches!(self.mode, Mode::List) {
            return match event {
                Event::Keyboard(Key::Up) => {
                    let total = self.entries() + 1;
                    self.selected = if self.selected == 0 {
                        total - 1
                    } else {
                        self.selected - 1
                    };
                    None
                }
                Event::Keyboard(Key::Down) => {
                    let total = self.entries() + 1;
                    self.selected = (self.selected + 1) % total;
                    None
                }
                Event::Keyboard(Key::Enter | Key::Char(' ')) => handle_list_enter(self),
                Event::Keyboard(Key::Delete | Key::Backspace) => {
                    handle_list_delete(self);
                    None
                }
                Event::Keyboard(Key::Escape) => Some(RouteTarget::Back),
                Event::Keyboard(_) => None,
            };
        }

        let mut pending = None;
        match &mut self.mode {
            Mode::Edit { profile, selected } => match event {
                Event::Keyboard(Key::Up) => {
                    *selected = if *selected == 0 {
                        EDIT_MENU_ITEMS - 1
                    } else {
                        *selected - 1
                    }
                }
                Event::Keyboard(Key::Down) => *selected = (*selected + 1) % EDIT_MENU_ITEMS,
                Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                    if *selected < 8 {
                        pending = Some(Pending::EditEnter(*profile, *selected));
                    } else {
                        self.mode = Mode::List;
                    }
                }
                Event::Keyboard(Key::Escape) => *selected = EDIT_MENU_ITEMS - 1,
                Event::Keyboard(_) => {}
            },
            Mode::TextInput {
                profile,
                field,
                input,
            } => {
                if let Some(action) = input.handle_event(&event) {
                    pending = Some(match action {
                        TextInputAction::Commit(value) => {
                            Pending::TextCommit(*profile, *field, value)
                        }
                        TextInputAction::Cancel => Pending::TextCancel(*profile, *field),
                    });
                }
            }
            Mode::ColorSelect {
                profile,
                field,
                selector,
            } => {
                if let Some(action) = selector.handle_event(&event) {
                    pending = Some(match action {
                        ValueSelectorAction::Commit(value) => {
                            let mut store = self.store.profiles.borrow_mut();
                            match field {
                                ColorField::Suit => store.profiles[*profile].suit_color = value,
                                ColorField::Ski => store.profiles[*profile].ski_color = value,
                            }
                            drop(store);
                            Pending::ColorCommit(*profile, *field)
                        }
                        ValueSelectorAction::Cancel => Pending::ColorCancel(*profile, *field),
                    });
                }
            }
            Mode::ReplaceSelect { profile, selector } => {
                if let Some(action) = selector.handle_event(&event) {
                    match action {
                        ValueSelectorAction::Commit(value) => {
                            self.store.profiles.borrow_mut().profiles[*profile].replace = value;
                            pending = Some(Pending::ReplaceCommit(*profile));
                        }
                        ValueSelectorAction::Cancel => {
                            pending = Some(Pending::ReplaceCancel(*profile));
                        }
                    }
                }
            }
            Mode::Question { action, dialog } => {
                if let Some(result) = dialog.handle_event(&event) {
                    pending = Some(match result {
                        ConfirmAction::Yes => Pending::QuestionYes(*action),
                        ConfirmAction::No => Pending::QuestionNo(*action),
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

    fn apply_palette(&self, palette: &mut Palette) {
        if let Some(profile) = self.active_profile() {
            let store = self.store.profiles.borrow();
            if let Some(p) = store.profiles.get(profile) {
                apply_suit_palette(palette, p.suit_color);
                apply_ski_palette(palette, p.ski_color);
            }
        }
        if let Mode::ColorSelect { field, .. } = &self.mode {
            match field {
                ColorField::Suit => {
                    for i in 0..NUM_SUITS {
                        apply_suit_palette_at(palette, i, (i + 1) * 5);
                    }
                }
                ColorField::Ski => {
                    for i in 0..NUM_SKIS {
                        apply_ski_palette_at(palette, i, (i + 1) * 5);
                    }
                }
            }
        }
    }
}

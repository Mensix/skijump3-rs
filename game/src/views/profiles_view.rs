use crate::components::confirm_dialog::{ConfirmAction, ConfirmDialog};
use crate::components::text_input::{TextInput, TextInputAction};
use crate::components::value_selector::{ValueSelector, ValueSelectorAction};
use crate::data::profile::{Profile, NUM_SKIS, NUM_SUITS};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::StoreRef;
use crate::utils::{format_profile_value, replace_display_name};
use engine::palette::Palette;
use engine::ui::{Component, Element, Event, Key, View};
use std::rc::Rc;

const EDIT_MENU_ITEMS: usize = 9;
const REPLACE_MAX: usize = 65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextField {
    Name,
    RealName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorField {
    Suit,
    Ski,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuestionAction {
    DeleteProfile(usize),
    ResetProfile(usize),
}

enum Mode {
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
    store: StoreRef,
    selected: usize,
    mode: Mode,
    langbase: Rc<crate::parsers::langbase::LangBase>,
}

impl ProfilesView {
    pub fn new(store: StoreRef) -> Self {
        let langbase = store.borrow().langbase.clone();
        Self {
            store,
            selected: 0,
            mode: Mode::List,
            langbase,
        }
    }

    fn y_for(temp: usize) -> i32 {
        (temp * 8 + 4) as i32
    }

    fn col_y(temp: usize) -> i32 {
        match temp {
            0..=9 => (temp * 8 + 4) as i32,
            10..=15 => (temp * 8 + 10) as i32,
            _ => (temp * 16 - 118) as i32,
        }
    }

    fn entries(&self) -> usize {
        let store = self.store.borrow();
        let np = store.profiles.num_profiles();
        if store.profiles.has_slot() {
            np + 1
        } else {
            np
        }
    }

    fn lstr(&self, index: usize, fallback: &str) -> String {
        let v = self.langbase.lstr(index);
        if v == "?" {
            fallback.to_string()
        } else {
            v.to_string()
        }
    }

    fn unique_default_profile(&self) -> Profile {
        let store = self.store.borrow();
        let mut profile = Profile::default();
        let mut counter = 2;
        while store
            .profiles
            .profiles
            .iter()
            .any(|p| p.name == profile.name)
        {
            profile.name = format!("SKI JUMPER {}", counter);
            profile.real_name = profile.name.clone();
            counter += 1;
        }
        profile
    }

    fn menu_selected(&self) -> Option<usize> {
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
            _ => (self.selected < self.store.borrow().profiles.num_profiles())
                .then_some(self.selected),
        }
    }

    fn draw_screen_base(&self, els: &mut Vec<Element>) {
        els.push(Element::fillbox(0, 0, 320, 200, 0));
        els.push(Element::fillbox(0, 0, 159, 200, BG_LEFT));
        els.push(Element::fillbox(160, 0, 160, 200, BG_RIGHT));
        els.push(Element::FillArea { thing: 63 });
        els.push(Element::text_color(
            &self.lstr(34, "Jumpers:"),
            40,
            3,
            FONT_HELP,
        ));
    }

    fn draw_list(&self, els: &mut Vec<Element>) {
        let store = self.store.borrow();
        let np = store.profiles.num_profiles();

        for (i, profile) in store.profiles.profiles.iter().enumerate() {
            let y = Self::y_for(i + 1);
            els.push(Element::fillbox(10, y - 1, 21, 8, BG_ORDER));
            if let Some(order_pos) = store.profiles.order_pos(i) {
                els.push(Element::text_color(
                    format!("{}.", order_pos + 1),
                    18,
                    y,
                    FONT_NEW,
                ));
            }
            els.push(Element::text_color(&profile.name, 40, y, FONT_NAME));
        }

        if store.profiles.has_slot() {
            els.push(Element::text_color(
                &self.lstr(302, "*Create New Jumper*"),
                40,
                Self::y_for(np + 2),
                FONT_NEW,
            ));
        }

        let back_temp = if store.profiles.has_slot() {
            np + 4
        } else {
            np + 3
        };
        els.push(Element::text_color(
            &self.lstr(33, "Back to Main Menu"),
            40,
            Self::y_for(back_temp),
            FONT_BACK,
        ));

        let entries = if store.profiles.has_slot() {
            np + 1
        } else {
            np
        };
        let box_y = if self.selected < entries {
            10 + (self.selected as i32) * 8
        } else {
            10 + (entries as i32 + 1) * 8
        };
        els.push(Element::box_(34, box_y, 123, 9, FONT_DEFAULT));
    }

    fn draw_help(&self, els: &mut Vec<Element>, profile: Option<usize>) {
        let store = self.store.borrow();
        if store.profiles.num_profiles() >= 16 {
            return;
        }

        els.push(Element::fillbox(1, 175, 158, 25, BG_LEFT));
        els.push(Element::FillArea { thing: 63 });

        if let Some(profile) = profile {
            let in_order = store.profiles.order_pos(profile).is_some();
            els.push(Element::text_color(
                &self.lstr(322, "(Use arrows,"),
                8,
                175,
                FONT_HELP,
            ));
            if in_order {
                els.push(Element::text_color(
                    &self.lstr(323, "ENTER edits jumper,"),
                    11,
                    183,
                    FONT_HELP,
                ));
                els.push(Element::text_color(
                    &self.lstr(324, "DEL removes from order)"),
                    11,
                    191,
                    FONT_HELP,
                ));
            } else {
                els.push(Element::text_color(
                    &self.lstr(325, "ENTER adds jumper,"),
                    11,
                    183,
                    FONT_HELP,
                ));
                els.push(Element::text_color(
                    &self.lstr(326, "DEL deletes jumper)"),
                    11,
                    191,
                    FONT_HELP,
                ));
            }
        }
    }

    fn draw_empty_edit(&self, els: &mut Vec<Element>) {
        els.push(Element::fillbox(166, 4, 154, 195, BG_RIGHT));
        els.push(Element::FillArea { thing: 63 });
    }

    fn draw_suit_ski(&self, els: &mut Vec<Element>, _profile: &Profile) {
        let labels = self.profile_labels();
        let suit_w = self.store.borrow().font.string_width(labels[2]) as i32;
        let ski_w = self.store.borrow().font.string_width(labels[3]) as i32;
        let x = 178 + suit_w.max(ski_w);
        let xl = (x + 18).min(318);

        els.push(Element::fillbox(x, 28, xl - x + 1, 5, 216));
        els.push(Element::box_(x, 28, xl - x + 1, 5, 218));
        els.push(Element::fillbox(x + 1, 37, xl - x - 1, 3, 231));
    }

    fn profile_labels(&self) -> [&'static str; 18] {
        [
            "Name:",
            "Real name:",
            "Suit Color:",
            "Ski Color:",
            "Replace:",
            "Coach:",
            "Skip Quali:",
            "Reset Jumper",
            "Exit",
            "Total Jumps:",
            "World Cup Completed:",
            "Legs Won:",
            "World Cups Won:",
            "Best:",
            "Best 4H:",
            "Longest WC:",
            "Longest:",
            "KOTH:",
        ]
    }

    fn draw_profile(&self, els: &mut Vec<Element>, profile_index: usize, edit_phase: bool) {
        self.draw_empty_edit(els);

        let store = self.store.borrow();
        let Some(profile) = store.profiles.profiles.get(profile_index).cloned() else {
            return;
        };
        let labels = self.profile_labels();
        let label_color = if edit_phase { FONT_DEFAULT } else { FONT_HELP };
        let value_color = FONT_NEW;

        if edit_phase {
            els.push(Element::fillbox(175, 85, 131, 1, FONT_HELP));
        }

        drop(store);
        self.draw_suit_ski(els, &profile);
        let store = self.store.borrow();
        let profile = &store.profiles.profiles[profile_index];

        for (i, label) in labels.iter().enumerate() {
            let temp = i + 1;
            if !edit_phase && temp > 7 && temp < 10 {
                continue;
            }
            if !label.is_empty() {
                let lc = if edit_phase && temp >= 10 {
                    FONT_HELP
                } else {
                    label_color
                };
                els.push(Element::text_color(*label, 166, Self::col_y(temp), lc));
            }
        }

        for (i, label) in labels.iter().enumerate() {
            let temp = i + 1;
            let y = Self::col_y(temp);
            let x = if temp > 15 {
                170
            } else {
                170 + store.font.string_width(label) as i32
            };
            let y = if temp > 15 { y + 8 } else { y };
            let value = format_profile_value(
                profile,
                temp,
                &store.font,
                &store.player_names,
                &self.langbase,
            );
            if !value.is_empty() {
                els.push(Element::text_color(value, x, y, value_color));
            }
        }

        if let Some(selected) = self.menu_selected() {
            els.push(Element::box_(
                162,
                10 + (selected as i32 * 8),
                155,
                9,
                FONT_DEFAULT,
            ));
        }
    }

    fn handle_list_enter(&mut self) -> Option<RouteTarget> {
        let entries = self.entries();
        let np = self.store.borrow().profiles.num_profiles();
        if self.selected >= entries {
            return Some(RouteTarget::MainMenu);
        }

        if self.selected >= np {
            let profile = self.unique_default_profile();
            let mut store = self.store.borrow_mut();
            store.profiles.profiles.push(profile);
            let profile_index = store.profiles.num_profiles() - 1;
            store.profiles.edit_index = profile_index;
            self.selected = profile_index;
            self.mode = Mode::Edit {
                profile: profile_index,
                selected: 0,
            };
            return None;
        }

        let in_order = self
            .store
            .borrow()
            .profiles
            .order_pos(self.selected)
            .is_some();
        if in_order {
            self.mode = Mode::Edit {
                profile: self.selected,
                selected: 0,
            };
        } else {
            self.store.borrow_mut().profiles.add_to_order(self.selected);
        }
        None
    }

    fn handle_list_delete(&mut self) {
        let np = self.store.borrow().profiles.num_profiles();
        if self.selected >= np {
            return;
        }

        if self
            .store
            .borrow()
            .profiles
            .order_pos(self.selected)
            .is_some()
        {
            self.store
                .borrow_mut()
                .profiles
                .remove_from_order(self.selected);
        } else {
            let name = self.store.borrow().profiles.profiles[self.selected]
                .name
                .clone();
            self.mode = Mode::Question {
                action: QuestionAction::DeleteProfile(self.selected),
                dialog: ConfirmDialog::new(
                    format!("{}: {}", self.lstr(328, "Delete"), name),
                    Rc::clone(&self.langbase),
                    self.store.borrow().font.clone(),
                ),
            };
        }
    }

    fn handle_edit_enter(&mut self, profile: usize, selected: usize) {
        match selected {
            0 => self.start_text_input(profile, TextField::Name),
            1 => self.start_text_input(profile, TextField::RealName),
            2 => {
                let value = self.store.borrow().profiles.profiles[profile].suit_color;
                let labels = self.profile_labels();
                let font = &self.store.borrow().font;
                let x = (172
                    + font
                        .string_width(labels[2])
                        .max(font.string_width(labels[3])) as i32)
                    .min(288);
                self.mode = Mode::ColorSelect {
                    profile,
                    field: ColorField::Suit,
                    selector: ValueSelector::color_bars(
                        x,
                        24,
                        NUM_SUITS - 1,
                        value,
                        242,
                        BG_RIGHT,
                        true,
                    ),
                };
            }
            3 => {
                let value = self.store.borrow().profiles.profiles[profile].ski_color;
                let labels = self.profile_labels();
                let font = &self.store.borrow().font;
                let x = (172
                    + font
                        .string_width(labels[2])
                        .max(font.string_width(labels[3])) as i32)
                    .min(288);
                self.mode = Mode::ColorSelect {
                    profile,
                    field: ColorField::Ski,
                    selector: ValueSelector::color_bars(
                        x,
                        32,
                        NUM_SKIS - 1,
                        value,
                        242,
                        BG_RIGHT,
                        false,
                    ),
                };
            }
            4 => {
                let value = self.store.borrow().profiles.profiles[profile]
                    .replace
                    .min(REPLACE_MAX);
                let x = self.store.borrow().font.string_width("Replace:") as i32 + 170;
                let display = if value > 0 {
                    replace_display_name(
                        value,
                        &self.store.borrow().player_names,
                        &self.store.borrow().font,
                        x,
                    )
                } else {
                    String::new()
                };
                let mut selector = ValueSelector::numeric(
                    x,
                    44,
                    320 - x,
                    REPLACE_MAX,
                    value,
                    245,
                    FONT_DEFAULT,
                    display,
                );
                if value > 0 {
                    selector.set_right_text(&format!("#{}", value));
                }
                selector.set_wrap(false);
                self.mode = Mode::ReplaceSelect { profile, selector };
            }
            5 => {
                let langbase = self.langbase.clone();
                let style = {
                    let mut store = self.store.borrow_mut();
                    let p = &mut store.profiles.profiles[profile];
                    p.coach_style += 1;
                    p.coach_style
                };
                let check = langbase.lstr(361 + style * 40);
                if check == "?" {
                    let mut store = self.store.borrow_mut();
                    store.profiles.profiles[profile].coach_style = 0;
                }
            }
            6 => {
                let mut store = self.store.borrow_mut();
                let profile = &mut store.profiles.profiles[profile];
                profile.skip_quali = (profile.skip_quali + 1) % 3;
            }
            7 => {
                self.mode = Mode::Question {
                    action: QuestionAction::ResetProfile(profile),
                    dialog: ConfirmDialog::new(
                        self.lstr(329, "Reset jumper?"),
                        Rc::clone(&self.langbase),
                        self.store.borrow().font.clone(),
                    ),
                };
            }
            _ => {}
        }
    }

    fn start_text_input(&mut self, profile: usize, field: TextField) {
        let store = self.store.borrow();
        let profile_data = &store.profiles.profiles[profile];
        let labels = self.profile_labels();
        let (old, label) = match field {
            TextField::Name => (profile_data.name.clone(), labels[0]),
            TextField::RealName => (profile_data.real_name.clone(), labels[1]),
        };
        let x = 170 + store.font.string_width(label) as i32;
        let y = match field {
            TextField::Name => 12,
            TextField::RealName => 20,
        };
        let max_width = 314 - 170 - store.font.string_width(label) as i32;
        let font = store.font.clone();
        drop(store);
        self.mode = Mode::TextInput {
            profile,
            field,
            input: TextInput::new(x, y, max_width, old, 245, FONT_DEFAULT, font),
        };
    }

    fn commit_text_input(&mut self, profile: usize, field: TextField, buf: String) {
        let value = buf.trim().to_ascii_uppercase();
        if value.is_empty() {
            self.mode = Mode::Edit {
                profile,
                selected: match field {
                    TextField::Name => 0,
                    TextField::RealName => 1,
                },
            };
            return;
        }

        if field == TextField::Name {
            let duplicate = self
                .store
                .borrow()
                .profiles
                .profiles
                .iter()
                .enumerate()
                .any(|(i, p)| i != profile && p.name == value);
            if duplicate {
                self.start_text_input(profile, field);
                return;
            }
        }

        let mut store = self.store.borrow_mut();
        match field {
            TextField::Name => store.profiles.profiles[profile].name = value,
            TextField::RealName => store.profiles.profiles[profile].real_name = value,
        }
        drop(store);
        self.mode = Mode::Edit {
            profile,
            selected: match field {
                TextField::Name => 0,
                TextField::RealName => 1,
            },
        };
    }

    fn apply_question(&mut self, action: QuestionAction) {
        match action {
            QuestionAction::DeleteProfile(profile) => {
                self.store.borrow_mut().profiles.remove_profile(profile);
                let np = self.store.borrow().profiles.num_profiles();
                if self.selected >= np {
                    self.selected = np.saturating_sub(1);
                }
                self.mode = Mode::List;
            }
            QuestionAction::ResetProfile(profile) => {
                let name = self.store.borrow().profiles.profiles[profile].name.clone();
                let mut reset = Profile::default();
                reset.name = name.clone();
                reset.real_name = name;
                self.store.borrow_mut().profiles.profiles[profile] = reset;
                self.mode = Mode::Edit {
                    profile,
                    selected: 7,
                };
            }
        }
    }
}

impl View<RouteTarget> for ProfilesView {
    fn elements(&self) -> Vec<Element> {
        let mut els = Vec::new();
        self.draw_screen_base(&mut els);
        self.draw_list(&mut els);

        if let Some(profile) = self.active_profile() {
            let edit_phase = !matches!(self.mode, Mode::List | Mode::Question { .. });
            self.draw_profile(&mut els, profile, edit_phase);
            if matches!(self.mode, Mode::List) {
                self.draw_help(&mut els, Some(profile));
            }
        } else {
            self.draw_empty_edit(&mut els);
            self.draw_help(&mut els, None);
        }

        match &self.mode {
            Mode::TextInput { input, .. } => els.extend(input.elements()),
            Mode::ColorSelect { selector, .. } => els.extend(selector.elements()),
            Mode::ReplaceSelect { selector, .. } => {
                let value = selector.value();
                let x = self.store.borrow().font.string_width("Replace:") as i32 + 170;
                els.push(Element::fillbox(x - 2, 43, 320 - x, 8, 245));
                if value > 0 {
                    let store = self.store.borrow();
                    if value <= store.player_names.len() {
                        let n = replace_display_name(value, &store.player_names, &store.font, x);
                        els.push(Element::text_color(n, x, 44, FONT_DEFAULT));
                        els.push(Element::text_color_right(
                            format!("#{}", value),
                            316,
                            44,
                            FONT_DEFAULT,
                        ));
                    } else {
                        els.push(Element::text_color(
                            format!("#{}", value),
                            x,
                            44,
                            FONT_DEFAULT,
                        ));
                    }
                } else {
                    els.push(Element::text_color(
                        self.lstr(9, "None"),
                        x,
                        44,
                        FONT_DEFAULT,
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
                Event::Keyboard(Key::Enter | Key::Char(' ')) => self.handle_list_enter(),
                Event::Keyboard(Key::Delete | Key::Backspace) => {
                    self.handle_list_delete();
                    None
                }
                Event::Keyboard(Key::Escape) => Some(RouteTarget::MainMenu),
                _ => None,
            };
        }

        enum Pending {
            EditEnter(usize, usize),
            TextCommit(usize, TextField, String),
            TextCancel(usize, TextField),
            ColorCommit(usize, ColorField),
            ColorCancel(usize, ColorField),
            ReplaceCommit(usize, usize),
            ReplaceCancel(usize),
            QuestionYes(QuestionAction),
            QuestionNo(QuestionAction),
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
                        pending = Some(Pending::EditEnter(*profile, *selected))
                    } else {
                        self.mode = Mode::List
                    }
                }
                Event::Keyboard(Key::Escape) => *selected = EDIT_MENU_ITEMS - 1,
                _ => {}
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
                            let mut store = self.store.borrow_mut();
                            match field {
                                ColorField::Suit => {
                                    store.profiles.profiles[*profile].suit_color = value
                                }
                                ColorField::Ski => {
                                    store.profiles.profiles[*profile].ski_color = value
                                }
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
                            self.store.borrow_mut().profiles.profiles[*profile].replace = value;
                            pending = Some(Pending::ReplaceCommit(*profile, value));
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
                self.handle_edit_enter(profile, selected)
            }
            Some(Pending::TextCommit(profile, field, value)) => {
                self.commit_text_input(profile, field, value)
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
            Some(Pending::ReplaceCommit(profile, _value)) => {
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
            Some(Pending::QuestionYes(action)) => self.apply_question(action),
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
            let store = self.store.borrow();
            if let Some(p) = store.profiles.profiles.get(profile) {
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

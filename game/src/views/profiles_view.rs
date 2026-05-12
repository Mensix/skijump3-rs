use crate::components::confirm_dialog::{ConfirmAction, ConfirmDialog};
use crate::components::text_input::{TextInput, TextInputAction};
use crate::components::value_selector::{ValueSelector, ValueSelectorAction};
use crate::data::profile::{Profile, NUM_SKIS, NUM_SUITS};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::StoreRef;
use engine::ui::{Component, Element, Event, Key, View};

const EDIT_MENU_ITEMS: usize = 8;
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
}

impl ProfilesView {
    pub fn new(store: StoreRef) -> Self {
        Self {
            store,
            selected: 0,
            mode: Mode::List,
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
        let store = self.store.borrow();
        let value = store.langbase.lstr(index);
        if value == "?" {
            fallback.to_string()
        } else {
            value.to_string()
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
                Self::y_for(np + 1),
                FONT_NEW,
            ));
        }

        let back_temp = if store.profiles.has_slot() {
            np + 3
        } else {
            np + 2
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
            els.push(Element::text_color("(Use arrows,", 8, 175, FONT_HELP));
            if in_order {
                els.push(Element::text_color(
                    "ENTER edits jumper,",
                    11,
                    183,
                    FONT_HELP,
                ));
                els.push(Element::text_color(
                    "DEL removes from order)",
                    11,
                    191,
                    FONT_HELP,
                ));
            } else {
                els.push(Element::text_color(
                    "ENTER adds jumper,",
                    11,
                    183,
                    FONT_HELP,
                ));
                els.push(Element::text_color(
                    "DEL deletes jumper)",
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

    fn draw_suit_ski(&self, els: &mut Vec<Element>, profile: &Profile) {
        let labels = self.profile_labels();
        let suit_w = self.store.borrow().font.string_width(labels[2]) as i32;
        let ski_w = self.store.borrow().font.string_width(labels[3]) as i32;
        let x = 178 + suit_w.max(ski_w);
        let xl = (x + 18).min(318);

        els.push(Element::fillbox(
            x,
            28,
            xl - x + 1,
            5,
            216 + (profile.suit_color as u8 * 5),
        ));
        els.push(Element::box_(
            x,
            28,
            xl - x + 1,
            5,
            218 + (profile.suit_color as u8 * 5),
        ));
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
            "",
            "",
            "Total Jumps:",
            "WC:",
            "Legs Won:",
            "WC Won:",
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
                els.push(Element::text_color(
                    *label,
                    166,
                    Self::col_y(temp),
                    label_color,
                ));
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
            let value = self.profile_value(profile, temp);
            if !value.is_empty() {
                els.push(Element::text_color(value, x, y, value_color));
            }
        }

        if let Some(selected) = self.menu_selected() {
            els.push(Element::box_(
                168,
                10 + (selected as i32 * 8),
                154,
                9,
                FONT_DEFAULT,
            ));
        }
    }

    fn profile_value(&self, profile: &Profile, temp: usize) -> String {
        match temp {
            1 => profile.name.clone(),
            2 => profile.real_name.clone(),
            5 => {
                if profile.replace == 0 {
                    "-".to_string()
                } else {
                    format!("#{}", profile.replace)
                }
            }
            6 => {
                if profile.coach_style == 0 {
                    self.lstr(9, "None")
                } else {
                    format!("Style {}", profile.coach_style)
                }
            }
            7 => match profile.skip_quali {
                0 => "Never".to_string(),
                1 => "If possible".to_string(),
                _ => "Always".to_string(),
            },
            10 => profile.total_jumps.to_string(),
            11 => profile.world_cups.to_string(),
            12 => profile.legs_won.to_string(),
            13 => profile.world_cups_won.to_string(),
            14 => profile.best_result.clone(),
            15 => profile.best_4h_result.clone(),
            16 => {
                if profile.best_wc_jump == 0 {
                    "-".to_string()
                } else {
                    format!("{} {}", profile.best_wc_jump, profile.best_wc_hill)
                }
            }
            17 => {
                if profile.best_jump == 0 {
                    "-".to_string()
                } else {
                    format!("{} {}", profile.best_jump, profile.best_hill)
                }
            }
            18 => {
                if profile.koth_level == 0 {
                    "-".to_string()
                } else {
                    format!("Level {}", profile.koth_level)
                }
            }
            _ => String::new(),
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
                dialog: ConfirmDialog::new(format!("Delete: {}", name)),
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
                self.mode = Mode::ReplaceSelect {
                    profile,
                    selector: ValueSelector::numeric(
                        x,
                        44,
                        320 - x,
                        REPLACE_MAX,
                        value,
                        245,
                        FONT_DEFAULT,
                    ),
                };
            }
            5 => {
                let mut store = self.store.borrow_mut();
                let profile = &mut store.profiles.profiles[profile];
                profile.coach_style = (profile.coach_style + 1) % 4;
            }
            6 => {
                let mut store = self.store.borrow_mut();
                let profile = &mut store.profiles.profiles[profile];
                profile.skip_quali = (profile.skip_quali + 1) % 3;
            }
            7 => {
                self.mode = Mode::Question {
                    action: QuestionAction::ResetProfile(profile),
                    dialog: ConfirmDialog::new(self.lstr(329, "Reset jumper?")),
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
            Mode::ReplaceSelect { selector, .. } => els.extend(selector.elements()),
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
            ColorCommit(usize, ColorField, usize),
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
                    pending = Some(Pending::EditEnter(*profile, *selected))
                }
                Event::Keyboard(Key::Escape) => self.mode = Mode::List,
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
                            Pending::ColorCommit(*profile, *field, value)
                        }
                        ValueSelectorAction::Cancel => Pending::ColorCancel(*profile, *field),
                    });
                }
            }
            Mode::ReplaceSelect { profile, selector } => {
                if let Some(action) = selector.handle_event(&event) {
                    pending = Some(match action {
                        ValueSelectorAction::Commit(value) => {
                            Pending::ReplaceCommit(*profile, value)
                        }
                        ValueSelectorAction::Cancel => Pending::ReplaceCancel(*profile),
                    });
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
            Some(Pending::ColorCommit(profile, field, value)) => {
                let mut store = self.store.borrow_mut();
                match field {
                    ColorField::Suit => store.profiles.profiles[profile].suit_color = value,
                    ColorField::Ski => store.profiles.profiles[profile].ski_color = value,
                }
                drop(store);
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
            Some(Pending::ReplaceCommit(profile, value)) => {
                self.store.borrow_mut().profiles.profiles[profile].replace = value;
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
}

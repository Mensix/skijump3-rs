use crate::data::profile::{Profile, NUM_SKIS, NUM_SUITS};
use crate::gfx::palette::{BG_RIGHT, BLACK, FILL_DIM, FONT_DEFAULT, FONT_HEADER, FONT_NEW};
use crate::route::RouteTarget;
use crate::text::layout::{lstr, replace_display_name};
use engine::oxide::widgets::confirm::ConfirmDialog as OxideConfirmDialog;
use engine::oxide::widgets::selector::NumericSelector;
use engine::oxide::widgets::text_input::TextInput as OxideTextInput;

use super::list::{ColorField, Mode, ProfilesView, QuestionAction, TextField, REPLACE_MAX};
use super::render::profile_label;

pub(super) fn save_players(view: &ProfilesView) {
    let result = view
        .store
        .with_profiles(|profiles| view.save_manager.save_players(profiles));
    if let Err(e) = result {
        eprintln!("Warning: failed to save players: {e}");
    }
}

pub(super) fn handle_list_enter(view: &mut ProfilesView) -> Option<RouteTarget> {
    let entries = view.entries();
    let np = view.store.profiles().num_profiles();
    if view.selected >= entries {
        return Some(RouteTarget::MainMenu);
    }

    if view.selected >= np {
        let profile = view.unique_default_profile();
        let profile_index = view.store.with_profiles_mut(|store| {
            store.profiles.push(profile);
            store.num_profiles() - 1
        });
        save_players(view);
        view.selected = profile_index;
        view.mode = Mode::Edit {
            profile: profile_index,
            selected: 0,
        };
        return None;
    }

    let in_order = view.store.profiles().order_pos(view.selected).is_some();
    if in_order {
        view.mode = Mode::Edit {
            profile: view.selected,
            selected: 0,
        };
    } else {
        view.store.profiles_mut().add_to_order(view.selected);
        save_players(view);
    }
    None
}

pub(super) fn handle_list_delete(view: &mut ProfilesView) {
    let np = view.store.profiles().num_profiles();
    if view.selected >= np {
        return;
    }

    if view.store.profiles().order_pos(view.selected).is_some() {
        view.store.profiles_mut().remove_from_order(view.selected);
        save_players(view);
    } else {
        let name = view.store.profiles().profiles[view.selected].name.clone();
        view.mode = Mode::Question {
            action: QuestionAction::DeleteProfile(view.selected),
            dialog: OxideConfirmDialog::new(
                (59, 79, 203, 53),
                BG_RIGHT,
                BLACK,
                FONT_HEADER,
                format!(
                    "{}: {}",
                    lstr(&view.resources.langbase, 328, "Delete"),
                    name
                ),
                "Y",
                "N",
            )
            .with_subtitle(lstr(&view.resources.langbase, 193, "Are you sure?")),
        };
    }
}

pub(super) fn handle_edit_enter(view: &mut ProfilesView, profile: usize, selected: usize) {
    match selected {
        0 => start_text_input(view, profile, TextField::Name),
        1 => start_text_input(view, profile, TextField::RealName),
        2 => {
            let value = view.store.profiles().profiles[profile].suit_color;
            let x = (172
                + view
                    .resources
                    .font
                    .string_width(&profile_label(view, 3))
                    .max(view.resources.font.string_width(&profile_label(view, 4)))
                    as i32)
                .min(288);
            view.mode = Mode::ColorSelect {
                profile,
                field: ColorField::Suit,
                selector: NumericSelector::new(
                    x,
                    24,
                    31,
                    NUM_SUITS - 1,
                    value,
                    BLACK,
                    FONT_DEFAULT,
                    "",
                ),
                color_x: x,
                color_y: 24,
                color_max: NUM_SUITS - 1,
                color_suit: true,
            };
        }
        3 => {
            let value = view.store.profiles().profiles[profile].ski_color;
            let x = (172
                + view
                    .resources
                    .font
                    .string_width(&profile_label(view, 3))
                    .max(view.resources.font.string_width(&profile_label(view, 4)))
                    as i32)
                .min(288);
            view.mode = Mode::ColorSelect {
                profile,
                field: ColorField::Ski,
                selector: NumericSelector::new(
                    x,
                    32,
                    31,
                    NUM_SKIS - 1,
                    value,
                    BLACK,
                    FONT_DEFAULT,
                    "",
                ),
                color_x: x,
                color_y: 32,
                color_max: NUM_SKIS - 1,
                color_suit: false,
            };
        }
        4 => {
            let value = view.store.profiles().profiles[profile]
                .replace
                .min(REPLACE_MAX);
            let x = view.resources.font.string_width("Replace:") as i32 + 170;
            let display = if value > 0 {
                replace_display_name(
                    value,
                    view.resources.player_names(),
                    &view.resources.font,
                    x,
                )
            } else {
                String::new()
            };
            let mut selector = NumericSelector::new(
                x,
                44,
                320 - x,
                REPLACE_MAX,
                value,
                FILL_DIM,
                FONT_DEFAULT,
                display,
            );
            selector.set_wrap(false);
            view.mode = Mode::ReplaceSelect { profile, selector };
        }
        5 => {
            let style = {
                let mut store = view.store.profiles_mut();
                let p = &mut store.profiles[profile];
                p.coach_style += 1;
                p.coach_style
            };
            let check = view.resources.langbase.lstr(361 + style * 40);
            if check == "?" {
                let mut store = view.store.profiles_mut();
                store.profiles[profile].coach_style = 0;
            }
            save_players(view);
        }
        6 => {
            let mut store = view.store.profiles_mut();
            let profile_ref = &mut store.profiles[profile];
            profile_ref.skip_quali = (profile_ref.skip_quali + 1) % 3;
            drop(store);
            save_players(view);
        }
        7 => {
            view.mode = Mode::Question {
                action: QuestionAction::ResetProfile(profile),
                dialog: OxideConfirmDialog::new(
                    (59, 79, 203, 53),
                    BG_RIGHT,
                    BLACK,
                    FONT_HEADER,
                    lstr(&view.resources.langbase, 329, "Reset jumper?"),
                    "Y",
                    "N",
                )
                .with_subtitle(lstr(
                    &view.resources.langbase,
                    193,
                    "Are you sure?",
                )),
            };
        }
        _ => {}
    }
}

pub(super) fn start_text_input(view: &mut ProfilesView, profile: usize, field: TextField) {
    let store = view.store.profiles();
    let profile_data = &store.profiles[profile];
    let old = match field {
        TextField::Name => profile_data.name.clone(),
        TextField::RealName => profile_data.real_name.clone(),
    };
    drop(store);
    let label = match field {
        TextField::Name => profile_label(view, 1),
        TextField::RealName => profile_label(view, 2),
    };
    let x = 170 + view.resources.font.string_width(&label) as i32;
    let y = match field {
        TextField::Name => 12,
        TextField::RealName => 20,
    };
    let max_width = 314 - 170 - view.resources.font.string_width(&label) as i32;
    view.mode = Mode::TextInput {
        profile,
        field,
        input: OxideTextInput::new(
            x,
            y,
            max_width,
            old,
            130,
            FILL_DIM,
            FONT_NEW,
            FONT_DEFAULT,
            view.resources.font.clone(),
        ),
    };
}

pub(super) fn commit_text_input(
    view: &mut ProfilesView,
    profile: usize,
    field: TextField,
    buf: &str,
) {
    let value = buf.trim().to_ascii_uppercase();
    if value.is_empty() {
        view.mode = Mode::Edit {
            profile,
            selected: match field {
                TextField::Name => 0,
                TextField::RealName => 1,
            },
        };
        return;
    }

    if field == TextField::Name {
        let duplicate = view
            .store
            .profiles()
            .profiles
            .iter()
            .enumerate()
            .any(|(i, p)| i != profile && p.name == value);
        if duplicate {
            start_text_input(view, profile, field);
            return;
        }
    }

    let mut store = view.store.profiles_mut();
    match field {
        TextField::Name => store.profiles[profile].name = value,
        TextField::RealName => store.profiles[profile].real_name = value,
    }
    drop(store);
    view.mode = Mode::Edit {
        profile,
        selected: match field {
            TextField::Name => 0,
            TextField::RealName => 1,
        },
    };
}

pub(super) fn apply_question(view: &mut ProfilesView, action: QuestionAction) {
    match action {
        QuestionAction::DeleteProfile(profile) => {
            view.store.profiles_mut().remove_profile(profile);
            let np = view.store.profiles().num_profiles();
            if view.selected >= np {
                view.selected = np.saturating_sub(1);
            }
            view.mode = Mode::List;
        }
        QuestionAction::ResetProfile(profile) => {
            let name = view.store.profiles().profiles[profile].name.clone();
            let reset = Profile {
                name,
                ..Default::default()
            };
            view.store.profiles_mut().profiles[profile] = reset;
            view.mode = Mode::Edit {
                profile,
                selected: 7,
            };
        }
    }
}

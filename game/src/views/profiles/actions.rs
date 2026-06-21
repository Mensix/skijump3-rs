use crate::data::profile::{Profile, NUM_SKIS, NUM_SUITS};
use crate::gfx::theme::{BG_RED, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::store::GameState;
use crate::text::layout::replace_display_name;
use engine::oxide::widgets::selector::NumericSelector;
use engine::oxide::widgets::text_input::TextInput as OxideTextInput;

use super::list::{ColorField, Mode, ProfilesView, TextField, REPLACE_MAX};
use super::render::profile_label;

pub(super) fn save_players(view: &ProfilesView, state: &GameState) {
    view.save_manager.save_players(&state.profiles);
}

pub(super) fn handle_list_enter(
    view: &mut ProfilesView,
    state: &mut GameState,
) -> Option<RouteTarget> {
    let entries = view.entries(state);
    let np = state.profiles.num_profiles();
    if view.selected >= entries {
        return Some(RouteTarget::MainMenu);
    }

    if view.selected >= np {
        let profile = view.unique_default_profile(state);
        state.profiles.profiles.push(profile);
        let profile_index = state.profiles.num_profiles() - 1;
        save_players(view, state);
        view.selected = profile_index;
        view.mode = Mode::Edit {
            profile: profile_index,
            selected: 0,
        };
        return None;
    }

    let in_order = state.profiles.order_pos(view.selected).is_some();
    if in_order {
        view.mode = Mode::Edit {
            profile: view.selected,
            selected: 0,
        };
    } else {
        state.profiles.add_to_order(view.selected);
        save_players(view, state);
    }
    None
}

pub(super) fn handle_list_delete(view: &mut ProfilesView, state: &mut GameState) {
    let np = state.profiles.num_profiles();
    if view.selected >= np {
        return;
    }

    if state.profiles.order_pos(view.selected).is_some() {
        state.profiles.remove_from_order(view.selected);
        save_players(view, state);
    } else {
        view.confirm_delete = Some(view.selected);
    }
}

pub(super) fn handle_edit_enter(
    view: &mut ProfilesView,
    state: &mut GameState,
    profile: usize,
    selected: usize,
) {
    let lang = &view.resources.langbase;
    match selected {
        0 => start_text_input(view, state, profile, TextField::Name),
        1 => start_text_input(view, state, profile, TextField::RealName),
        2 => {
            let value = state.profiles.profiles[profile].suit_color;
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
                    FONT_BODY,
                    "",
                ),
                color_x: x,
                color_y: 24,
                color_max: NUM_SUITS - 1,
                color_suit: true,
            };
        }
        3 => {
            let value = state.profiles.profiles[profile].ski_color;
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
                    FONT_BODY,
                    "",
                ),
                color_x: x,
                color_y: 32,
                color_max: NUM_SKIS - 1,
                color_suit: false,
            };
        }
        4 => {
            let value = state.profiles.profiles[profile].replace.min(REPLACE_MAX);
            let x = view.resources.font.string_width("Replace:") as i32 + 170;
            let display = if value > 0 {
                replace_display_name(
                    value,
                    view.resources
                        .player_names(state.config.name_set_index as usize),
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
                FILL_GRAY,
                FONT_BODY,
                display,
            );
            selector.set_wrap(false);
            view.mode = Mode::ReplaceSelect { profile, selector };
        }
        5 => {
            let p = &mut state.profiles.profiles[profile];
            p.coach_style += 1;
            let style = p.coach_style;
            let check = lang.tr(361 + style * 40);
            if check == "?" {
                state.profiles.profiles[profile].coach_style = 0;
            }
            save_players(view, state);
        }
        6 => {
            state.profiles.profiles[profile].skip_qualification =
                (state.profiles.profiles[profile].skip_qualification + 1) % 3;
            save_players(view, state);
        }
        7 => {
            view.confirm_reset = Some(profile);
        }
        _ => {}
    }
}

pub(super) fn start_text_input(
    view: &mut ProfilesView,
    state: &GameState,
    profile: usize,
    field: TextField,
) {
    let profile_data = &state.profiles.profiles[profile];
    let old = match field {
        TextField::Name => profile_data.name.clone(),
        TextField::RealName => profile_data.real_name.clone(),
    };
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
            FILL_GRAY,
            FONT_GOLD,
            FONT_BODY,
            view.resources.font.clone(),
        ),
    };
}

pub(super) fn commit_text_input(
    view: &mut ProfilesView,
    state: &mut GameState,
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
        let duplicate = state
            .profiles
            .profiles
            .iter()
            .enumerate()
            .any(|(i, p)| i != profile && p.name == value);
        if duplicate {
            start_text_input(view, state, profile, field);
            return;
        }
    }

    match field {
        TextField::Name => state.profiles.profiles[profile].name = value,
        TextField::RealName => state.profiles.profiles[profile].real_name = value,
    }
    view.mode = Mode::Edit {
        profile,
        selected: match field {
            TextField::Name => 0,
            TextField::RealName => 1,
        },
    };
}

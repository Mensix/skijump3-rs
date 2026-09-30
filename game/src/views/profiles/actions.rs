use crate::data::profile::{NUM_SKIS, NUM_SUITS};
use crate::gfx::jumper_colors::{SkiIdx, SuitIdx};
use crate::gfx::theme::{FILL_GRAY, FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::screen::{NavSignal, Persistence};
use crate::store::GameState;
use crate::ui::{
    EventCx, NumericSelector, NumericSelectorParams, SelectorMessage, TextInput, TextInputMessage,
    TextInputParams,
};
use crate::ui::{UiEvent, Widget};

use super::list::ProfilesView;
use super::render::profile_label;
use super::state::{ColorField, Mode, Pending, TextField, REPLACE_MAX};

pub(super) fn save_players(cx: &Persistence, state: &GameState) {
    cx.save_players(&state.profiles);
}

pub(super) fn handle_list_enter(
    view: &mut ProfilesView,
    state: &mut GameState,
    cx: &Persistence,
) -> NavSignal {
    let entries = view.entries(state);
    let np = state.profiles.num_profiles();
    if view.selected >= entries {
        return NavSignal::Route(RouteTarget::MainMenu);
    }

    if view.selected >= np {
        let profile = view.unique_default_profile(state);
        state.profiles.profiles.push(profile);
        let profile_index = state.profiles.num_profiles() - 1;
        save_players(cx, state);
        view.selected = profile_index;
        view.enter_edit_mode(profile_index, 0);
        return NavSignal::None;
    }

    let in_order = state.profiles.order_pos(view.selected).is_some();
    if in_order {
        view.enter_edit_mode(view.selected, 0);
    } else {
        state.profiles.add_to_order(view.selected);
        save_players(cx, state);
    }
    NavSignal::None
}

pub(super) fn handle_list_delete(view: &mut ProfilesView, state: &mut GameState, cx: &Persistence) {
    let np = state.profiles.num_profiles();
    if view.selected >= np {
        return;
    }

    if state.profiles.order_pos(view.selected).is_some() {
        state.profiles.remove_from_order(view.selected);
        save_players(cx, state);
    } else {
        view.confirm_delete = Some(view.selected);
    }
}

fn start_color_select(
    view: &mut ProfilesView,
    profile: usize,
    field: ColorField,
    value: usize,
    max: usize,
    color_y: i32,
    color_suit: bool,
) {
    let x = (172
        + view
            .resources
            .font
            .string_width(&profile_label(view, 3))
            .max(view.resources.font.string_width(&profile_label(view, 4))) as i32)
        .min(288);
    view.mode = Mode::ColorSelect {
        profile,
        field,
        selector: NumericSelector::new(NumericSelectorParams { max, value }),
        color_x: x,
        color_y,
        color_max: max,
        color_suit,
    };
}

pub(super) fn handle_edit_enter(
    view: &mut ProfilesView,
    state: &mut GameState,
    profile: usize,
    selected: usize,
    cx: &Persistence,
) {
    let lang = &view.resources.langbase;
    match selected {
        0 => start_text_input(view, state, profile, TextField::Name),
        1 => start_text_input(view, state, profile, TextField::RealName),
        2 => {
            let current = state.profiles.profiles[profile].suit_color;
            start_color_select(
                view,
                profile,
                ColorField::Suit,
                SuitIdx::from_rgb(current).0 as usize,
                NUM_SUITS - 1,
                24,
                true,
            );
        }
        3 => {
            let current = state.profiles.profiles[profile].ski_color;
            start_color_select(
                view,
                profile,
                ColorField::Ski,
                SkiIdx::from_rgb(current).0 as usize,
                NUM_SKIS - 1,
                32,
                false,
            );
        }
        4 => {
            let slider_val = state.profiles.profiles[profile]
                .replace
                .map_or(0, |v| v + 1)
                .min(REPLACE_MAX);
            let mut selector = NumericSelector::new(NumericSelectorParams {
                max: REPLACE_MAX,
                value: slider_val,
            });
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
            save_players(cx, state);
        }
        6 => {
            state.profiles.profiles[profile].skip_qualification =
                (state.profiles.profiles[profile].skip_qualification + 1) % 3;
            save_players(cx, state);
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
        input: TextInput::new(TextInputParams {
            x,
            y,
            max_width,
            initial: old,
            max_chars: usize::MAX,
            bg: FILL_GRAY,
            fg: FONT_GOLD,
            cursor: FONT_BODY,
            font: view.resources.font.clone(),
        }),
    };
}

pub(super) fn commit_text_input(
    view: &mut ProfilesView,
    state: &mut GameState,
    profile: usize,
    field: TextField,
    buf: &str,
) {
    let value = buf.trim().to_uppercase();
    if value.is_empty() {
        view.enter_edit_mode(
            profile,
            match field {
                TextField::Name => 0,
                TextField::RealName => 1,
            },
        );
        return;
    }

    if field == TextField::Name {
        let duplicate = state
            .profiles
            .profiles
            .iter()
            .enumerate()
            .any(|(i, p)| i != profile && p.name.eq_ignore_ascii_case(&value));
        if duplicate {
            start_text_input(view, state, profile, field);
            return;
        }
    }

    match field {
        TextField::Name => state.profiles.profiles[profile].name = value,
        TextField::RealName => state.profiles.profiles[profile].real_name = value,
    }
    view.enter_edit_mode(
        profile,
        match field {
            TextField::Name => 0,
            TextField::RealName => 1,
        },
    );
}

pub(super) fn handle_event_text_input(mode: &mut Mode, event: UiEvent) -> Option<Pending> {
    let Mode::TextInput {
        profile,
        field,
        input,
    } = mode
    else {
        return None;
    };
    let mut ecx = EventCx::default();
    input.event(&mut ecx, event).map(|action| match action {
        TextInputMessage::Commit(value) => Pending::TextCommit(*profile, *field, value),
        TextInputMessage::Cancel => Pending::TextCancel(*profile, *field),
    })
}

pub(super) fn handle_event_color_select(
    mode: &mut Mode,
    state: &mut GameState,
    event: UiEvent,
) -> Option<Pending> {
    let Mode::ColorSelect {
        profile,
        field,
        selector,
        ..
    } = mode
    else {
        return None;
    };
    let mut ecx = EventCx::default();
    selector.event(&mut ecx, event).map(|action| match action {
        SelectorMessage::Commit(value) => {
            match field {
                ColorField::Suit => {
                    state.profiles.profiles[*profile].suit_color = SuitIdx(value as u8).rgb()
                }
                ColorField::Ski => {
                    state.profiles.profiles[*profile].ski_color = SkiIdx(value as u8).rgb()
                }
            }
            Pending::ColorCommit(*profile, *field)
        }
        SelectorMessage::Cancel => Pending::ColorCancel(*profile, *field),
    })
}

pub(super) fn handle_event_replace_select(
    mode: &mut Mode,
    state: &mut GameState,
    event: UiEvent,
) -> Option<Pending> {
    let Mode::ReplaceSelect {
        profile, selector, ..
    } = mode
    else {
        return None;
    };
    let mut ecx = EventCx::default();
    selector.event(&mut ecx, event).map(|action| match action {
        SelectorMessage::Commit(value) => {
            state.profiles.profiles[*profile].replace =
                if value == 0 { None } else { Some(value - 1) };
            Pending::ReplaceCommit(*profile)
        }
        SelectorMessage::Cancel => Pending::ReplaceCancel(*profile),
    })
}

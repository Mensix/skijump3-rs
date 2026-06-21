use crate::gfx::jumper_colors::{ski_color, suit_color_shade};
use crate::gfx::theme::{BG_PURPLE, BG_RED, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::store::GameState;
use crate::views::profiles::format::format_profile_value;
use engine::oxide::PaintCx;

use super::list::{Mode, ProfilesView};

pub(super) fn draw_screen_base(view: &ProfilesView, cx: &mut PaintCx<'_>) {
    let lang = &view.resources.langbase;
    cx.fill((0, 0, 320, 200), BLACK);
    cx.pattern_fill((0, 0, 159, 200), BG_PURPLE);
    cx.pattern_fill((160, 0, 160, 200), BG_RED);

    cx.text((40, 3), FONT_GRAY, lang.tr(34));
}

pub(super) fn draw_list(view: &ProfilesView, state: &GameState, cx: &mut PaintCx<'_>) {
    let lang = &view.resources.langbase;
    let store = &state.profiles;
    let np = store.num_profiles();

    for (i, profile) in store.profiles.iter().enumerate() {
        let y = ProfilesView::row_y(i + 1);
        cx.fill((10, y - 1, 21, 8), BG_PURPLE);
        if let Some(order_pos) = store.order_pos(i) {
            cx.text((18, y), FONT_GOLD, format!("{}.", order_pos + 1));
        }
        cx.text((40, y), FONT_BODY, &profile.name);
    }

    if store.has_slot() {
        cx.text((40, ProfilesView::row_y(np + 1)), FONT_GOLD, lang.tr(302));
    }

    let back_row = if store.has_slot() { np + 3 } else { np + 2 };
    cx.text((40, ProfilesView::row_y(back_row)), FONT_BODY, lang.tr(33));

    if matches!(view.mode, Mode::List) {
        let entries = if store.has_slot() { np + 1 } else { np };
        let box_y = if view.selected < entries {
            10 + (view.selected as i32) * 8
        } else {
            10 + (entries as i32 + 1) * 8
        };
        cx.stroke((34, box_y, 123, 9), FONT_BODY);
    }
}

pub(super) fn draw_help(
    view: &ProfilesView,
    state: &GameState,
    cx: &mut PaintCx<'_>,
    profile: Option<usize>,
) {
    let lang = &view.resources.langbase;
    let store = &state.profiles;
    if store.num_profiles() >= 16 {
        return;
    }

    cx.pattern_fill((1, 175, 158, 25), BG_PURPLE);

    if let Some(profile) = profile {
        let in_order = store.order_pos(profile).is_some();
        cx.text((8, 175), FONT_GRAY, lang.tr(322));
        if in_order {
            cx.text((11, 183), FONT_GRAY, lang.tr(323));
            cx.text((11, 191), FONT_GRAY, lang.tr(324));
        } else {
            cx.text((11, 183), FONT_GRAY, lang.tr(325));
            cx.text((11, 191), FONT_GRAY, lang.tr(326));
        }
    }
}

pub(super) fn draw_empty_edit(cx: &mut PaintCx<'_>) {
    cx.pattern_fill((166, 4, 154, 195), BG_RED);
}

pub(super) fn draw_suit_ski(
    view: &ProfilesView,
    cx: &mut PaintCx<'_>,
    suit_idx: usize,
    ski_idx: usize,
) {
    let suit_label = profile_label(view, 3);
    let ski_label = profile_label(view, 4);
    let suit_w = view.resources.font.string_width(&suit_label) as i32;
    let ski_w = view.resources.font.string_width(&ski_label) as i32;
    let x = 178 + suit_w.max(ski_w);
    let xl = (x + 18).min(318);

    cx.fill((x, 28, xl - x + 1, 5), suit_color_shade(suit_idx, 1));
    cx.stroke((x, 28, xl - x + 1, 5), suit_color_shade(suit_idx, 3));
    cx.fill((x + 1, 37, xl - x - 1, 3), ski_color(ski_idx));
}

pub(super) fn profile_label(view: &ProfilesView, field: usize) -> String {
    let lang = &view.resources.langbase;
    match field {
        1..=18 => lang.tr(303 + field).to_string(),
        _ => String::new(),
    }
}

pub(super) fn draw_profile(
    view: &ProfilesView,
    state: &GameState,
    cx: &mut PaintCx<'_>,
    profile_index: usize,
    edit_phase: bool,
) {
    draw_empty_edit(cx);

    let store = &state.profiles;
    let Some(profile) = store.profiles.get(profile_index) else {
        return;
    };
    let label_color = if edit_phase { FONT_BODY } else { FONT_GRAY };
    let value_color = FONT_GOLD;

    if edit_phase {
        cx.fill((175, 85, 131, 1), FONT_GRAY);
    }

    draw_suit_ski(view, cx, profile.suit_color, profile.ski_color);

    for field in 0..18 {
        if !edit_phase && field > 7 && field < 10 {
            continue;
        }
        let label = profile_label(view, field);
        if !label.is_empty() {
            let lc = if edit_phase && field >= 10 {
                FONT_GRAY
            } else {
                label_color
            };
            cx.text((166, ProfilesView::col_y(field)), lc, label);
        }
    }

    for field in 0..18 {
        let y = ProfilesView::col_y(field);
        let x = if field > 15 {
            170
        } else {
            170 + view
                .resources
                .font
                .string_width(&profile_label(view, field)) as i32
        };
        let y = if field > 15 { y + 8 } else { y };
        let value = format_profile_value(
            profile,
            field,
            &view.resources.font,
            view.resources
                .player_names(state.config.name_set_index as usize),
            &view.resources.langbase,
        );
        if !value.is_empty() {
            cx.text((x, y), value_color, value);
        }
    }

    if let Some(selected) = view.menu_selected() {
        cx.stroke((162, 10 + (selected as i32 * 8), 155, 9), FONT_BODY);
    }
}

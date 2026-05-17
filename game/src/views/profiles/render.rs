use engine::ui::Element;

use crate::gfx::palette::{
    BG_LEFT, BG_ORDER, BG_RIGHT, FONT_BACK, FONT_DEFAULT, FONT_HELP, FONT_NAME, FONT_NEW,
};
use crate::text::layout::lstr;
use crate::views::profiles::format::format_profile_value;

use super::list::{Mode, ProfilesView};

pub(super) fn draw_screen_base(view: &ProfilesView, els: &mut Vec<Element>) {
    els.push(Element::fillbox(0, 0, 320, 200, 0));
    els.push(Element::fillbox(0, 0, 159, 200, BG_LEFT));
    els.push(Element::fillbox(160, 0, 160, 200, BG_RIGHT));
    els.push(Element::FillArea { thing: 63 });
    els.push(Element::text(
        lstr(&view.resources.langbase, 34, "Jumpers:"),
        40,
        3,
        FONT_HELP,
        false,
    ));
}

pub(super) fn draw_list(view: &ProfilesView, els: &mut Vec<Element>) {
    let store = view.store.profiles.borrow();
    let np = store.num_profiles();

    for (i, profile) in store.profiles.iter().enumerate() {
        let y = ProfilesView::y_for(i + 1);
        els.push(Element::fillbox(10, y - 1, 21, 8, BG_ORDER));
        if let Some(order_pos) = store.order_pos(i) {
            els.push(Element::text(
                format!("{}.", order_pos + 1),
                18,
                y,
                FONT_NEW,
                false,
            ));
        }
        els.push(Element::text(&profile.name, 40, y, FONT_NAME, false));
    }

    if store.has_slot() {
        els.push(Element::text(
            lstr(&view.resources.langbase, 302, "*Create New Jumper*"),
            40,
            ProfilesView::y_for(np + 1),
            FONT_NEW,
            false,
        ));
    }

    let back_temp = if store.has_slot() { np + 3 } else { np + 2 };
    els.push(Element::text(
        lstr(&view.resources.langbase, 33, "Back to Main Menu"),
        40,
        ProfilesView::y_for(back_temp),
        FONT_BACK,
        false,
    ));

    if matches!(view.mode, Mode::List) {
        let entries = if store.has_slot() { np + 1 } else { np };
        let box_y = if view.selected < entries {
            10 + (view.selected as i32) * 8
        } else {
            10 + (entries as i32 + 1) * 8
        };
        els.push(Element::box_(34, box_y, 123, 9, FONT_DEFAULT));
    }
}

pub(super) fn draw_help(view: &ProfilesView, els: &mut Vec<Element>, profile: Option<usize>) {
    let store = view.store.profiles.borrow();
    if store.num_profiles() >= 16 {
        return;
    }

    els.push(Element::fillbox(1, 175, 158, 25, BG_LEFT));
    els.push(Element::FillArea { thing: 63 });

    if let Some(profile) = profile {
        let in_order = store.order_pos(profile).is_some();
        els.push(Element::text(
            lstr(&view.resources.langbase, 322, "(Use arrows,"),
            8,
            175,
            FONT_HELP,
            false,
        ));
        if in_order {
            els.push(Element::text(
                lstr(&view.resources.langbase, 323, "ENTER edits jumper,"),
                11,
                183,
                FONT_HELP,
                false,
            ));
            els.push(Element::text(
                lstr(&view.resources.langbase, 324, "DEL removes from order)"),
                11,
                191,
                FONT_HELP,
                false,
            ));
        } else {
            els.push(Element::text(
                lstr(&view.resources.langbase, 325, "ENTER adds jumper,"),
                11,
                183,
                FONT_HELP,
                false,
            ));
            els.push(Element::text(
                lstr(&view.resources.langbase, 326, "DEL deletes jumper)"),
                11,
                191,
                FONT_HELP,
                false,
            ));
        }
    }
}

pub(super) fn draw_empty_edit(els: &mut Vec<Element>) {
    els.push(Element::fillbox(166, 4, 154, 195, BG_RIGHT));
    els.push(Element::FillArea { thing: 63 });
}

pub(super) fn draw_suit_ski(view: &ProfilesView, els: &mut Vec<Element>) {
    let suit_label = profile_label(view, 3);
    let ski_label = profile_label(view, 4);
    let suit_w = view.resources.font.string_width(&suit_label) as i32;
    let ski_w = view.resources.font.string_width(&ski_label) as i32;
    let x = 178 + suit_w.max(ski_w);
    let xl = (x + 18).min(318);

    els.push(Element::fillbox(x, 28, xl - x + 1, 5, 216));
    els.push(Element::box_(x, 28, xl - x + 1, 5, 218));
    els.push(Element::fillbox(x + 1, 37, xl - x - 1, 3, 231));
}

pub(super) fn profile_label(view: &ProfilesView, field: usize) -> String {
    lstr(
        &view.resources.langbase,
        303 + field,
        match field {
            1 => "Name:",
            2 => "Real name:",
            3 => "Suit Color:",
            4 => "Ski Color:",
            5 => "Replace:",
            6 => "Coach:",
            7 => "Skip Quali:",
            8 => "Reset Jumper",
            9 => "Exit",
            10 => "Total Jumps:",
            11 => "World Cup Completed:",
            12 => "Legs Won:",
            13 => "World Cups Won:",
            14 => "Best:",
            15 => "Best 4H:",
            16 => "Longest WC:",
            17 => "Longest:",
            18 => "KOTH:",
            _ => "",
        },
    )
}

pub(super) fn draw_profile(
    view: &ProfilesView,
    els: &mut Vec<Element>,
    profile_index: usize,
    edit_phase: bool,
) {
    draw_empty_edit(els);

    let store = view.store.profiles.borrow();
    let Some(profile) = store.profiles.get(profile_index) else {
        return;
    };
    let label_color = if edit_phase { FONT_DEFAULT } else { FONT_HELP };
    let value_color = FONT_NEW;

    if edit_phase {
        els.push(Element::fillbox(175, 85, 131, 1, FONT_HELP));
    }

    draw_suit_ski(view, els);

    for field in 1..=18 {
        if !edit_phase && field > 7 && field < 10 {
            continue;
        }
        let label = profile_label(view, field);
        if !label.is_empty() {
            let lc = if edit_phase && field >= 10 {
                FONT_HELP
            } else {
                label_color
            };
            els.push(Element::text(
                label,
                166,
                ProfilesView::col_y(field),
                lc,
                false,
            ));
        }
    }

    for field in 1..=18 {
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
            &view.resources.player_names,
            &view.resources.langbase,
        );
        if !value.is_empty() {
            els.push(Element::text(value, x, y, value_color, false));
        }
    }

    if let Some(selected) = view.menu_selected() {
        els.push(Element::box_(
            162,
            10 + (selected as i32 * 8),
            155,
            9,
            FONT_DEFAULT,
        ));
    }
}

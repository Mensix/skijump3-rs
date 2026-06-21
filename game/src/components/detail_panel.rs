use crate::gfx::theme::{BG_PURPLE, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY};
use engine::oxide::PaintCx;

const LABEL_X: i32 = 150;
const VALUE_X: i32 = 170;
const TITLE_Y: i32 = 51;
const FILENAME_LABEL_Y: i32 = 71;
const FILENAME_BOX_Y: i32 = 78;
const FILENAME_BOX_H: i32 = 21;
const FILENAME_TEXT_Y: i32 = 85;
const COUNTER_X: i32 = 272;
const FIELDS_START_Y: i32 = 106;
const FIELD_SPACING: i32 = 20;
const FIELD_VALUE_OFFSET: i32 = 9;
const EXTRA_Y: i32 = 163;
const EMPTY_Y: i32 = 80;
const NAV_HINT_Y: i32 = 185;

pub fn paint_detail_panel(
    cx: &mut PaintCx<'_>,
    title: &str,
    filename: &str,
    fields: &[(String, String)],
    extra_value: Option<&str>,
    counter: Option<(usize, usize)>,
    nav_hint: &str,
    empty_text: &str,
    is_empty: bool,
) {
    cx.text((VALUE_X, TITLE_Y), FONT_GRAY, title);
    cx.text((LABEL_X, NAV_HINT_Y), FONT_GRAY, nav_hint);

    if is_empty {
        cx.text((VALUE_X, EMPTY_Y), FONT_GOLD, empty_text);
        return;
    }

    cx.text((LABEL_X, FILENAME_LABEL_Y), FONT_GRAY, "Filename:");
    let fw = cx.string_width(filename) as i32;
    let bx = LABEL_X + 13;
    let bw = (fw + 14).clamp(95, 155);
    cx.fill((bx, FILENAME_BOX_Y, bw, FILENAME_BOX_H), FILL_PURPLE);
    cx.fill(
        (bx + 1, FILENAME_BOX_Y + 1, bw - 2, FILENAME_BOX_H - 2),
        BG_PURPLE,
    );
    cx.text((bx + 7, FILENAME_TEXT_Y), FONT_GOLD, filename);

    if let Some((cur, total)) = counter {
        cx.text(
            (COUNTER_X, FILENAME_TEXT_Y),
            FONT_GRAY,
            format!("{cur}/{total}"),
        );
    }

    for (i, (label, value)) in fields.iter().enumerate() {
        let y = FIELDS_START_Y + (i as i32) * FIELD_SPACING;
        cx.text((LABEL_X, y), FONT_GRAY, label);
        cx.text((VALUE_X, y + FIELD_VALUE_OFFSET), FONT_BODY, value);
    }

    if let Some(val) = extra_value {
        cx.text((VALUE_X, EXTRA_Y), FONT_GRAY, val);
    }
}

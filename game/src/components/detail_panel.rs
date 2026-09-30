use crate::gfx::theme::{BG_PURPLE, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::text::lang::LangBase;
use crate::ui::UiCanvas;
use engine::color::Rgba;

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
const FILENAME_BOX_W: i32 = 95;

pub struct DetailPanel<'a> {
    pub lang: &'a LangBase,
    pub title: &'a str,
    pub filename: &'a str,
    pub filename_color: Rgba,
    pub fields: &'a [(String, String)],
    pub extra_value: Option<&'a str>,
    pub counter: Option<(usize, usize)>,
    pub nav_hint: &'a str,
    pub empty_text: &'a str,
    pub is_empty: bool,
}

pub fn paint_detail_panel(cx: &mut dyn UiCanvas, panel: DetailPanel<'_>) {
    let DetailPanel {
        lang,
        title,
        filename,
        filename_color,
        fields,
        extra_value,
        counter,
        nav_hint,
        empty_text,
        is_empty,
    } = panel;
    cx.text((VALUE_X, TITLE_Y), FONT_GRAY, title);
    cx.text((LABEL_X, NAV_HINT_Y), FONT_GRAY, nav_hint);

    if is_empty {
        cx.text((VALUE_X, EMPTY_Y), FONT_GOLD, empty_text);
        return;
    }

    cx.text((LABEL_X, FILENAME_LABEL_Y), FONT_GRAY, lang.tr(293));
    let bx = LABEL_X + 13;
    let max_text_w = FILENAME_BOX_W - 14;
    let mut display_filename = filename.to_string();
    while cx.string_width(&display_filename) as i32 > max_text_w {
        display_filename.pop();
    }
    cx.fill(
        (bx, FILENAME_BOX_Y, FILENAME_BOX_W, FILENAME_BOX_H),
        FILL_PURPLE,
    );
    cx.fill(
        (
            bx + 1,
            FILENAME_BOX_Y + 1,
            FILENAME_BOX_W - 2,
            FILENAME_BOX_H - 2,
        ),
        BG_PURPLE,
    );
    cx.text((bx + 7, FILENAME_TEXT_Y), filename_color, &display_filename);

    if let Some((cur, total)) = counter {
        cx.text(
            (COUNTER_X, FILENAME_TEXT_Y),
            FONT_GRAY,
            &format!("{cur}/{total}"),
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

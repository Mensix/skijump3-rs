use crate::gfx::theme::{BG_PURPLE, BLACK, FONT_BODY, FONT_GOLD};
use engine::color::Rgba;
use engine::oxide::PaintCx;

pub const ALERT_RECT: (i32, i32, i32, i32) = (60, 80, 201, 51);

pub fn alert_box(cx: &mut PaintCx<'_>, rect: (i32, i32, i32, i32), bg: Rgba) {
    cx.fill((rect.0 - 1, rect.1 - 1, rect.2 + 2, rect.3 + 2), BLACK);
    cx.pattern_fill(rect, bg);
}

pub fn alert_prompt(
    cx: &mut PaintCx<'_>,
    bg: Rgba,
    line1: impl AsRef<str>,
    line2: impl AsRef<str>,
    cursor_on: bool,
) {
    let line2 = line2.as_ref();
    let cursor_x = 80 + cx.string_width(line2) as i32 + 8;

    alert_box(cx, ALERT_RECT, bg);
    cx.text((80, 90), FONT_GOLD, line1.as_ref());
    cx.text((80, 110), FONT_GOLD, line2);
    cx.fill((cursor_x - 2, 108, 9, 11), BG_PURPLE);
    if cursor_on {
        cx.fill((cursor_x, 116, 5, 1), FONT_BODY);
    }
}

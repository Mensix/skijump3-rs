use crate::gfx::theme::{BG_PURPLE, BG_RED, BLACK, FONT_BODY, FONT_GOLD};
use engine::color::Rgba;
use engine::oxide::PaintCx;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub const ALERT_RECT: (i32, i32, i32, i32) = (60, 80, 201, 51);

static BLINKER: AtomicU32 = AtomicU32::new(0);

fn blinker_visible(on: u32, off: u32) -> bool {
    let c = BLINKER.fetch_add(1, Ordering::SeqCst);
    c % (on + off) < on
}

pub fn alert_box(cx: &mut PaintCx<'_>, rect: (i32, i32, i32, i32), bg: Rgba) {
    cx.fill((rect.0 - 1, rect.1 - 1, rect.2 + 2, rect.3 + 2), BLACK);
    cx.pattern_fill(rect, bg);
}

pub fn alert_prompt(cx: &mut PaintCx<'_>, line1: impl AsRef<str>, line2: impl AsRef<str>) {
    let line2 = line2.as_ref();
    let cursor_x = 80 + cx.string_width(line2) as i32 + 8;

    alert_box(cx, ALERT_RECT, BG_RED);
    cx.text((80, 90), FONT_GOLD, line1.as_ref());
    cx.text((80, 110), FONT_GOLD, line2);
    cx.fill((cursor_x - 2, 108, 9, 11), BG_PURPLE);
    if blinker_visible(11, 10) {
        cx.fill((cursor_x, 116, 5, 1), FONT_BODY);
    }
}

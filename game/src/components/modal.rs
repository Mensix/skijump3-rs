use crate::gfx::theme::{BG_PURPLE, BG_RED, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY};
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

pub fn alert_prompt(cx: &mut PaintCx<'_>, line1: impl AsRef<str>, line2: impl AsRef<str>, has_yn: bool) {
    let line2 = line2.as_ref();

    alert_box(cx, ALERT_RECT, BG_RED);
    cx.text((80, 90), FONT_GOLD, line1.as_ref());

    let msg_w = cx.string_width(line2) as i32;
    cx.text((80, 110), FONT_GOLD, line2);
    if has_yn {
        cx.text((80 + msg_w, 110), FONT_GRAY, " (Y/N):");
    }

    let cursor_x = if has_yn {
        80 + msg_w + cx.string_width(" (Y/N):") as i32 + 8
    } else {
        80 + msg_w + 8
    };
    cx.fill((cursor_x - 2, 108, 9, 11), BG_PURPLE);
    if blinker_visible(11, 10) {
        cx.fill((cursor_x, 116, 5, 1), FONT_BODY);
    }
}

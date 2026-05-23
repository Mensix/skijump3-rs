use crate::gfx::palette::{BG_LEFT, FONT_HELP};
use crate::gfx::sprites;
use engine::ui::Element;

#[must_use]
pub fn new_screen(style: u8) -> Vec<Element> {
    let mut els = vec![Element::fillbox(0, 0, 320, 200, 0)];

    match style {
        1 => {
            els.push(Element::fillbox(0, 0, 320, 19, 245));
            els.push(Element::fillbox(0, 20, 320, 180, 243));
        }
        2 => {
            els.push(Element::fillbox(0, 0, 11, 200, 245));
            els.push(Element::fillbox(12, 0, 296, 200, BG_LEFT));
            els.push(Element::fillbox(309, 0, 11, 200, 245));
        }
        4 => {
            els.push(Element::fillbox(0, 0, 320, 19, 245));
            els.push(Element::fillbox(0, 20, 320, 99, 243));
            els.push(Element::fillbox(0, 120, 320, 19, 245));
            els.push(Element::fillbox(0, 140, 320, 60, 243));
        }
        5 => {
            els.push(Element::fillbox(0, 0, 320, 200, 243));
        }
        _ => {}
    }

    els.push(Element::fill_area(63));

    match style {
        1 => els.push(Element::sprite(sprites::Sprite::Logo as u16, 5, 2)),
        2 => els.push(Element::sprite(sprites::Sprite::Logo as u16, 30, 8)),
        4 => {
            els.push(Element::sprite(sprites::Sprite::Logo as u16, 5, 2));
            els.push(Element::sprite(sprites::Sprite::Logo as u16, 5, 122));
        }
        _ => {}
    }

    els
}

/// Two-tone modal box overlay with dither.
#[must_use]
pub fn modal_background(x: i32, y: i32, w: i32, h: i32) -> Vec<Element> {
    vec![
        Element::fillbox(x, y, w, h, 242),
        Element::fillbox(x + 1, y + 1, w - 2, h - 2, 244),
        Element::fill_area(63),
    ]
}

#[must_use]
pub fn page_hints(page: usize, pages: usize, prev: &str, next: &str, end: &str) -> Vec<Element> {
    let mut els = Vec::with_capacity(2);
    if page > 0 {
        els.push(Element::right_text(format!("(-{prev}"), 319, 5, FONT_HELP));
    }
    let text = if page + 1 == pages { end } else { next };
    els.push(Element::right_text(format!("{text}-)"), 319, 13, FONT_HELP));
    els
}

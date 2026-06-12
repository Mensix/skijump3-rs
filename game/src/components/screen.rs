use crate::competition::types::{CompetitionPhase, CupStyle};
use crate::gfx::palette::{
    BG_4HILLS, BG_LEFT, BG_RIGHT, BG_WC, BLACK, FILL_DIM, FONT_DEFAULT, FONT_HELP,
};
use crate::gfx::sprites;
use engine::color::Rgba;
use engine::ui::Element;

#[must_use]
pub fn new_screen(style: u8) -> Vec<Element> {
    new_screen_with_bg(style, BG_LEFT)
}

#[must_use]
pub fn new_screen_with_bg(style: u8, bg: Rgba) -> Vec<Element> {
    let mut els = vec![Element::fillbox(0, 0, 320, 200, BLACK)];

    match style {
        1 => {
            els.push(Element::fillbox(0, 0, 320, 19, FILL_DIM));
            els.push(Element::fillbox(0, 20, 320, 180, bg));
        }
        2 => {
            els.push(Element::fillbox(0, 0, 11, 200, FILL_DIM));
            els.push(Element::fillbox(12, 0, 296, 200, bg));
            els.push(Element::fillbox(309, 0, 11, 200, FILL_DIM));
        }
        3 => {
            els.push(Element::fillbox(0, 0, 168, 98, FILL_DIM));
            els.push(Element::fillbox(0, 100, 168, 199, BG_RIGHT));
            els.push(Element::fillbox(170, 0, 319, 199, bg));
        }
        4 => {
            els.push(Element::fillbox(0, 0, 320, 19, FILL_DIM));
            els.push(Element::fillbox(0, 20, 320, 99, bg));
            els.push(Element::fillbox(0, 120, 320, 19, FILL_DIM));
            els.push(Element::fillbox(0, 140, 320, 60, bg));
        }
        5 => {
            els.push(Element::fillbox(0, 0, 320, 200, bg));
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

#[must_use]
pub fn black_screen() -> Vec<Element> {
    vec![Element::fillbox(0, 0, 320, 200, BLACK)]
}

#[must_use]
pub fn message_screen(message: &str, hint: &str) -> Vec<Element> {
    vec![
        Element::fillbox(0, 0, 320, 200, BLACK),
        Element::text(message, 20, 80, FONT_DEFAULT, false),
        Element::text(hint, 20, 95, FONT_HELP, false),
    ]
}

/// Two-tone modal box overlay with dither.
#[must_use]
pub fn modal_background(x: i32, y: i32, w: i32, h: i32) -> Vec<Element> {
    vec![
        Element::fillbox(x, y, w, h, BLACK),
        Element::fillbox(x + 1, y + 1, w - 2, h - 2, BG_RIGHT),
        Element::fill_area(63),
    ]
}

#[must_use]
pub fn panel_background(x: i32, y: i32, w: i32, h: i32, border: Rgba, bg: Rgba) -> Vec<Element> {
    vec![
        Element::fillbox(x, y, w, h, border),
        Element::fillbox(x + 1, y + 1, w - 2, h - 2, bg),
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

/// Background colour for list/standings screens, matching Pascal MuutaMenu tints.
#[must_use]
pub fn list_background(phase: CompetitionPhase, style: CupStyle) -> Rgba {
    match phase {
        CompetitionPhase::FourHillsStandings => BG_4HILLS,
        CompetitionPhase::WorldCupStandings => BG_WC,
        CompetitionPhase::SeasonComplete => {
            if matches!(style, CupStyle::FourHills | CupStyle::CustomCup) {
                BG_4HILLS
            } else {
                BG_WC
            }
        }
        _ => BG_LEFT,
    }
}

use crate::palette_consts::FONT_HELP;
use engine::ui::Element;

const MENU_LOGO_SPRITE: u16 = 61;

#[must_use] 
pub fn new_screen(style: u8) -> Vec<Element> {
    let mut els = vec![Element::fillbox(0, 0, 320, 200, 0)];

    match style {
        1 => {
            els.push(Element::fillbox(0, 0, 320, 19, 245));
            els.push(Element::fillbox(0, 20, 320, 180, 243));
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

    els.push(Element::FillArea { thing: 63 });

    match style {
        1 => els.push(Element::sprite(MENU_LOGO_SPRITE, 5, 2)),
        4 => {
            els.push(Element::sprite(MENU_LOGO_SPRITE, 5, 2));
            els.push(Element::sprite(MENU_LOGO_SPRITE, 5, 122));
        }
        _ => {}
    }

    els
}

#[must_use] 
pub fn page_hints(page: usize, pages: usize, prev: &str, next: &str, end: &str) -> Vec<Element> {
    let mut els = Vec::with_capacity(2);
    if page > 0 {
        els.push(Element::text_color_right(
            format!("(-{prev}"),
            319,
            5,
            FONT_HELP,
        ));
    }
    let text = if page + 1 == pages { end } else { next };
    els.push(Element::text_color_right(
        format!("{text}-)"),
        319,
        13,
        FONT_HELP,
    ));
    els
}

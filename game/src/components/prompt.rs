use crate::gfx::palette::FONT_DEFAULT;
use crate::text::lang::LangBase;
use engine::color::Rgba;
use engine::ui::Element;

pub(crate) fn push_cursor_box(
    els: &mut Vec<Element>,
    x: i32,
    y: i32,
    bg: Rgba,
    cursor_color: Rgba,
    cursor_visible: bool,
) {
    els.push(Element::fillbox(x - 2, y - 2, 9, 11, bg));
    if cursor_visible {
        els.push(Element::fillbox(x, y + 6, 5, 1, cursor_color));
    }
}

pub(crate) fn push_wait_for_key(
    els: &mut Vec<Element>,
    langbase: &LangBase,
    x: i32,
    y: i32,
    bg: Rgba,
    text_color: Rgba,
    cursor_color: Rgba,
    cursor_visible: bool,
) {
    els.push(Element::right_text(
        langbase.lstr(15).to_string(),
        x,
        y,
        text_color,
    ));
    push_cursor_box(els, x + 1, y, bg, cursor_color, cursor_visible);
}

pub(crate) fn push_yes_no_cursor(
    els: &mut Vec<Element>,
    x: i32,
    y: i32,
    bg: Rgba,
    cursor_visible: bool,
) {
    push_cursor_box(els, x, y, bg, FONT_DEFAULT, cursor_visible);
}

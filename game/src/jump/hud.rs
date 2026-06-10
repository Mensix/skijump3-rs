use crate::components::prompt;
use crate::data::records::HillRecord;
use crate::gfx::palette::{FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP};
use crate::gfx::sprites;
use crate::text::lang::LangBase;
use engine::color::Rgba;
use engine::ui::{Element, Font};

const KEY_NAMES: [&str; 5] = ["ARROW UP", "ARROW RIGHT", "ARROW LEFT", "T", "R"];

pub(crate) fn key_name(i: usize) -> &'static str {
    KEY_NAMES.get(i.wrapping_sub(1)).copied().unwrap_or("?")
}

pub(crate) fn push_info_panel_frame(els: &mut Vec<Element>) {
    els.push(Element::sprite(sprites::Sprite::InfoPanel as u16, 227, 2));
}

pub(crate) fn push_keymap(els: &mut Vec<Element>, langbase: &LangBase) {
    push_info_panel_frame(els);
    els.push(Element::right_text(langbase.lstr(330), 308, 9, FONT_GOLD));
    for i in 1..=5 {
        let ii = i as i32;
        els.push(Element::right_text(
            format!("{}: {}", langbase.lstr(330 + i), key_name(i)),
            308,
            9 + ii * 10,
            FONT_GOLD,
        ));
    }
}

pub(crate) fn push_hill_record_info(
    els: &mut Vec<Element>,
    langbase: &LangBase,
    hill_name_k: &str,
    hill_record: Option<&HillRecord>,
) {
    push_info_panel_frame(els);
    els.push(Element::right_text(hill_name_k, 308, 9, FONT_GOLD));
    els.push(Element::text(langbase.lstr(65), 308, 19, FONT_GOLD, true));
    if let Some(record) = hill_record {
        if record.len > 0.0 {
            els.push(Element::right_text(&record.name, 308, 29, FONT_GOLD));
            els.push(Element::text(
                format!("{:.1}m", record.len),
                308,
                39,
                FONT_GOLD,
                true,
            ));
        }
    }
}

pub(crate) fn push_jumper_info_box(
    els: &mut Vec<Element>,
    font: &Font,
    langbase: &LangBase,
    phase_label: &str,
    jumper_name: &str,
    subline: Option<(&str, Rgba)>,
) {
    els.push(Element::sprite(
        sprites::Sprite::JumperInfoBox as u16,
        3,
        150,
    ));
    let label56 = langbase.lstr(56);
    let label_w = font.string_width(label56) as i32;
    els.push(Element::text(phase_label, 12, 160, FONT_GREET, false));
    els.push(Element::text(label56, 12, 172, FONT_GREET, false));
    els.push(Element::text(
        jumper_name,
        12 + label_w,
        172,
        FONT_DEFAULT,
        false,
    ));
    if let Some((text, color)) = subline {
        els.push(Element::text(text, 14 + label_w, 179, color, false));
    }
    els.push(Element::text(langbase.lstr(59), 12, 191, FONT_HELP, false));
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
    prompt::push_wait_for_key(
        els,
        langbase,
        x,
        y,
        bg,
        text_color,
        cursor_color,
        cursor_visible,
    );
}

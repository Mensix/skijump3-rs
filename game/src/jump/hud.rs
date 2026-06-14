use crate::data::records::HillRecord;
use crate::gfx::sprites;
use crate::gfx::theme::{FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP};
use crate::text::lang::LangBase;
use engine::color::Rgba;
use engine::oxide::Font;
use engine::oxide::PaintCx;

const KEY_NAMES: [&str; 5] = ["ARROW UP", "ARROW RIGHT", "ARROW LEFT", "T", "R"];

pub(crate) fn key_name(i: usize) -> &'static str {
    KEY_NAMES.get(i.wrapping_sub(1)).copied().unwrap_or("?")
}

pub(crate) fn push_info_panel_frame(cx: &mut PaintCx<'_>) {
    cx.sprite(sprites::Sprite::InfoPanel as u16, (227, 2));
}

pub(crate) fn push_keymap(cx: &mut PaintCx<'_>, langbase: &LangBase) {
    push_info_panel_frame(cx);
    cx.right_text((308, 9), FONT_GOLD, langbase.lstr(330));
    for i in 1..=5 {
        let ii = i as i32;
        cx.right_text(
            (308, 9 + ii * 10),
            FONT_GOLD,
            format!("{}: {}", langbase.lstr(330 + i), key_name(i)),
        );
    }
}

pub(crate) fn push_hill_record_info(
    cx: &mut PaintCx<'_>,
    langbase: &LangBase,
    hill_name_k: &str,
    hill_record: Option<&HillRecord>,
) {
    push_info_panel_frame(cx);
    cx.right_text((308, 9), FONT_GOLD, hill_name_k);
    cx.right_text((308, 19), FONT_GOLD, langbase.lstr(65));
    if let Some(record) = hill_record {
        if record.len > 0.0 {
            cx.right_text((308, 29), FONT_GOLD, &record.name);
            cx.right_text((308, 39), FONT_GOLD, format!("{:.1}m", record.len));
        }
    }
}

pub(crate) fn push_jumper_info_box(
    cx: &mut PaintCx<'_>,
    font: &Font,
    langbase: &LangBase,
    phase_label: &str,
    jumper_name: &str,
    subline: Option<(&str, Rgba)>,
) {
    cx.sprite(sprites::Sprite::JumperInfoBox as u16, (3, 150));
    let label56 = langbase.lstr(56);
    let label_w = font.string_width(label56) as i32;
    cx.text((12, 160), FONT_GREET, phase_label);
    cx.text((12, 172), FONT_GREET, label56);
    cx.text((12 + label_w, 172), FONT_DEFAULT, jumper_name);
    if let Some((text, color)) = subline {
        cx.text((14 + label_w, 179), color, text);
    }
    cx.text((12, 191), FONT_HELP, langbase.lstr(59));
}

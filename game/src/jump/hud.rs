use crate::data::records::HillRecord;
use crate::gfx::sprites;
use crate::gfx::theme::{FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::save::config::Config;
use crate::text::input::key_name;
use crate::text::lang::LangBase;
use crate::ui::Font;
use crate::ui::UiCanvas;
use engine::color::Rgba;

pub(crate) fn push_info_panel_frame(cx: &mut dyn UiCanvas) {
    cx.sprite(sprites::Sprite::InfoPanel as u16, (227, 2));
}

pub(crate) fn push_keymap(cx: &mut dyn UiCanvas, lang: &LangBase, config: &Config) {
    push_info_panel_frame(cx);
    cx.right_text((308, 9), FONT_GOLD, lang.tr(330));
    let keys = [
        config.key_up,
        config.key_right,
        config.key_left,
        config.key_telemark,
        config.key_replay,
    ];
    for (index, code) in keys.into_iter().enumerate() {
        let key_no = index + 1;
        let key_y = key_no as i32;
        cx.right_text(
            (308, 9 + key_y * 10),
            FONT_GOLD,
            &format!("{}: {}", lang.tr(330 + key_no), key_name(code, lang)),
        );
    }
}

pub(crate) fn push_hill_record_info(
    cx: &mut dyn UiCanvas,
    lang: &LangBase,
    hill_name_k: &str,
    hill_record: Option<&HillRecord>,
    goal_distance: Option<f64>,
) {
    push_info_panel_frame(cx);
    cx.right_text((308, 9), FONT_GOLD, hill_name_k);
    cx.right_text((308, 19), FONT_GOLD, lang.tr(65));
    if let Some(record) = hill_record {
        cx.right_text((308, 29), FONT_GOLD, &record.name);
        cx.right_text((308, 39), FONT_GOLD, &format!("{:.1}m", record.len));
    }
    if let Some(goal) = goal_distance.filter(|goal| *goal > 0.0) {
        cx.right_text(
            (308, 49),
            FONT_GOLD,
            &format!("{}: {goal:.1}m", lang.tr(242)),
        );
    }
}

pub(crate) fn push_jumper_info_box(
    cx: &mut dyn UiCanvas,
    font: &Font,
    lang: &LangBase,
    phase_label: &str,
    jumper_name: &str,
    subline: Option<(&str, Rgba)>,
) {
    cx.sprite(sprites::Sprite::JumperInfoBox as u16, (3, 150));
    let label56 = lang.tr(56);
    let label_w = font.string_width(label56) as i32;
    cx.text((12, 160), FONT_TEAL, phase_label);
    cx.text((12, 172), FONT_TEAL, label56);
    cx.text((12 + label_w, 172), FONT_BODY, jumper_name);
    if let Some((text, color)) = subline {
        cx.text((14 + label_w, 179), color, text);
    }
    cx.text((12, 191), FONT_GRAY, lang.tr(59));
}

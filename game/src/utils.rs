use crate::data::profile::Profile;
use crate::parsers::langbase::LangBase;
use engine::ui::Font;

const LABEL_OFFSET: i32 = 170;
const RIGHT_EDGE: i32 = 316;

pub fn truncate_to_fit(s: &str, font: &Font, max_width: i32) -> String {
    if max_width <= 0 {
        return String::new();
    }
    if font.string_width(s) as i32 <= max_width {
        return s.to_string();
    }
    let mut n = s.to_string();
    while font.string_width(&n) as i32 > max_width && n.len() > 1 {
        n.pop();
    }
    n
}

pub fn replace_label_x(label_width: i32) -> i32 {
    LABEL_OFFSET + label_width
}

pub fn replace_right_text(value: usize) -> String {
    format!("#{}", value)
}

pub fn replace_max_width(value: usize, font: &Font, x: i32) -> i32 {
    RIGHT_EDGE
        .saturating_sub(x + font.string_width(&replace_right_text(value)) as i32 + 4)
        .max(0)
}

pub fn replace_display_name(value: usize, player_names: &[String], font: &Font, x: i32) -> String {
    if value == 0 || value > player_names.len() {
        return String::new();
    }
    let name = &player_names[value - 1];
    let max_w = replace_max_width(value, font, x);
    truncate_to_fit(name, font, max_w)
}

fn lang_str(langbase: &LangBase, index: usize, fallback: &str) -> String {
    let v = langbase.lstr(index);
    if v == "?" {
        fallback.to_string()
    } else {
        v.to_string()
    }
}

pub fn format_profile_value(
    profile: &Profile,
    temp: usize,
    font: &Font,
    player_names: &[String],
    langbase: &LangBase,
) -> String {
    match temp {
        1 => profile.name.clone(),
        2 => profile.real_name.clone(),
        5 => {
            if profile.replace == 0 {
                "-".to_string()
            } else if profile.replace <= player_names.len() {
                let max_w = RIGHT_EDGE
                    .saturating_sub(
                        LABEL_OFFSET + font.string_width(&format!("#{}", profile.replace)) as i32,
                    )
                    .max(0);
                let name = &player_names[profile.replace - 1];
                truncate_to_fit(name, font, max_w)
            } else {
                format!("#{}", profile.replace)
            }
        }
        6 => {
            if profile.coach_style == 0 {
                lang_str(langbase, 9, "None")
            } else {
                lang_str(
                    langbase,
                    361 + profile.coach_style * 40,
                    &format!("Style {}", profile.coach_style),
                )
            }
        }
        7 => lang_str(
            langbase,
            231 + profile.skip_quali,
            match profile.skip_quali {
                0 => "Never",
                1 => "If possible",
                _ => "Always",
            },
        ),
        10 => profile.total_jumps.to_string(),
        11 => profile.world_cups.to_string(),
        12 => profile.legs_won.to_string(),
        13 => profile.world_cups_won.to_string(),
        14 => profile.best_result.clone(),
        15 => profile.best_4h_result.clone(),
        16 => {
            if profile.best_wc_jump == 0 {
                "-".to_string()
            } else {
                format!("{}x {}", profile.best_wc_jump, profile.best_wc_hill)
            }
        }
        17 => {
            if profile.best_jump == 0 {
                "-".to_string()
            } else {
                format!("{}x {}", profile.best_jump, profile.best_hill)
            }
        }
        18 => {
            if profile.koth_level == 0 {
                "-".to_string()
            } else {
                lang_str(
                    langbase,
                    130 + profile.koth_level,
                    &format!("Level {}", profile.koth_level),
                )
            }
        }
        _ => String::new(),
    }
}

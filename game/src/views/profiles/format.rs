use crate::data::profile::Profile;
use crate::text::lang::LangBase;
use crate::text::layout::{lstr, shorten_name};
use engine::ui::Font;

pub fn format_profile_value(
    profile: &Profile,
    field: usize,
    font: &Font,
    player_names: &[String],
    langbase: &LangBase,
) -> String {
    match field {
        1 => profile.name.clone(),
        2 => profile.real_name.clone(),
        5 => {
            if profile.replace == 0 {
                "-".to_string()
            } else if profile.replace <= player_names.len() {
                let x = 170 + font.string_width("Replace:") as i32;
                let max_w = 316i32.saturating_sub(x).max(0);
                let name = &player_names[profile.replace - 1];
                shorten_name(name, font, max_w)
            } else {
                format!("#{}", profile.replace)
            }
        }
        6 => {
            if profile.coach_style == 0 {
                lstr(langbase, 9, "None")
            } else {
                lstr(
                    langbase,
                    361 + profile.coach_style * 40,
                    &format!("Style {}", profile.coach_style),
                )
            }
        }
        7 => lstr(
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
                format!("{}x {}", profile.best_wc_jump, profile.best_wc_hill_display)
            }
        }
        17 => {
            if profile.best_jump == 0 {
                "-".to_string()
            } else {
                format!("{}x {}", profile.best_jump, profile.best_hill_display)
            }
        }
        18 => {
            if profile.koth_level == 0 {
                "-".to_string()
            } else {
                lstr(
                    langbase,
                    130 + profile.koth_level,
                    &format!("Level {}", profile.koth_level),
                )
            }
        }
        _ => String::new(),
    }
}

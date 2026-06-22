use crate::data::profile::Profile;
use crate::text::format::format_decimal;
use crate::text::lang::LangBase;
use crate::text::layout::shorten_name;
use engine::oxide::Font;

pub fn format_profile_value(
    profile: &Profile,
    field: usize,
    font: &Font,
    player_names: &[String],
    lang: &LangBase,
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
                lang.tr(9).to_string()
            } else {
                lang.tr(361 + profile.coach_style * 40).to_string()
            }
        }
        7 => lang.tr(231 + profile.skip_qualification).to_string(),
        10 => profile.total_jumps.to_string(),
        11 => profile.world_cups.to_string(),
        12 => profile.legs_won.to_string(),
        13 => profile.world_cups_won.to_string(),
        14 => profile.best_result.clone(),
        15 => profile.best_4h_result.clone(),
         16 => {
            if profile.best_wc_jump == 0.0 {
                "-".to_string()
            } else {
                format!(
                    "{}x {}",
                    format_decimal(profile.best_wc_jump),
                    profile.best_wc_hill_display
                )
            }
        }
        17 => {
            if profile.best_jump == 0.0 {
                "-".to_string()
            } else {
                format!(
                    "{}x {}",
                    format_decimal(profile.best_jump),
                    profile.best_hill_display
                )
            }
        }
        18 => {
            if profile.koth_level == 0 {
                "-".to_string()
            } else {
                lang.tr(130 + profile.koth_level).to_string()
            }
        }
        _ => String::new(),
    }
}

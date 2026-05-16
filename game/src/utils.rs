use crate::data::profile::Profile;
use crate::parsers::langbase::LangBase;
use engine::ui::Font;

const LABEL_OFFSET: i32 = 170;
const RIGHT_EDGE: i32 = 316;

#[must_use] 
pub fn shorten_name(s: &str, font: &Font, max_width: i32) -> String {
    if max_width <= 0 {
        return String::new();
    }
    if font.string_width(s) as i32 <= max_width {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    for i in 1..chars.len().saturating_sub(1) {
        if chars[i] == ' ' && i + 1 < chars.len() && chars[i + 1] != ' ' {
            let rest: String = chars[i..].iter().collect();
            let abbr = format!("{}.{}", chars[0], rest);
            if font.string_width(&abbr) as i32 <= max_width {
                return abbr;
            }
        }
    }
    for len in (1..chars.len()).rev() {
        let mut n: String = chars[..len].iter().collect();
        n.push('.');
        if font.string_width(&n) as i32 <= max_width {
            return n;
        }
    }
    chars[..1].iter().collect()
}

#[must_use] 
pub fn replace_label_x(label_width: i32) -> i32 {
    LABEL_OFFSET + label_width
}

#[must_use] 
pub fn replace_right_text(value: usize) -> String {
    format!("#{value}")
}

#[must_use] 
pub fn replace_max_width(value: usize, font: &Font, x: i32) -> i32 {
    RIGHT_EDGE
        .saturating_sub(x + font.string_width(&replace_right_text(value)) as i32 + 4)
        .max(0)
}

#[must_use] 
pub fn replace_display_name(value: usize, player_names: &[String], font: &Font, x: i32) -> String {
    if value == 0 || value > player_names.len() {
        return String::new();
    }
    let name = &player_names[value - 1];
    let max_w = replace_max_width(value, font, x);
    shorten_name(name, font, max_w)
}

fn lang_str(langbase: &LangBase, index: usize, fallback: &str) -> String {
    let v = langbase.lstr(index);
    if v == "?" {
        fallback.to_string()
    } else {
        v.to_string()
    }
}

#[must_use] 
pub fn pascal_decode(bytes: &[u8]) -> String {
    fn cp850_to_char(b: u8) -> char {
        match b {
            0x80 => '\u{00C7}',
            0x81 => '\u{00FC}',
            0x82 => '\u{00E9}',
            0x83 => '\u{00E2}',
            0x84 => '\u{00E4}',
            0x85 => '\u{00E0}',
            0x86 => '\u{00E5}',
            0x87 => '\u{00E7}',
            0x88 => '\u{00EA}',
            0x89 => '\u{00EB}',
            0x8A => '\u{00E8}',
            0x8B => '\u{00EF}',
            0x8C => '\u{00EE}',
            0x8D => '\u{00EC}',
            0x8E => '\u{00C4}',
            0x8F => '\u{00C5}',
            0x90 => '\u{00C9}',
            0x91 => '\u{00E6}',
            0x92 => '\u{00C6}',
            0x93 => '\u{00F4}',
            0x94 => '\u{00F6}',
            0x95 => '\u{00F2}',
            0x96 => '\u{00FB}',
            0x97 => '\u{00F9}',
            0x98 | 0xFF => '\u{00FF}',
            0x99 => '\u{00D6}',
            0x9A => '\u{00DC}',
            0x9B => '\u{00F8}',
            0x9C => '\u{00A3}',
            0x9D => '\u{00D8}',
            0x9E => '\u{00D7}',
            0x9F => '\u{0192}',
            0xA0 => '\u{00E1}',
            0xA1 => '\u{00ED}',
            0xA2 => '\u{00F3}',
            0xA3 => '\u{00FA}',
            0xA4 => '\u{00F1}',
            0xA5 => '\u{00D1}',
            0xA6 => '\u{00AA}',
            0xA7 => '\u{00BA}',
            0xA8 => '\u{00BF}',
            0xE1 => '\u{00DF}',
            0xE6 => '\u{00B5}',
            _ => {
                if b.is_ascii() {
                    b as char
                } else {
                    '\u{FFFD}'
                }
            }
        }
    }
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        out.push(cp850_to_char(b));
    }
    out
}

#[must_use] 
pub fn is_computer_name(name: &str) -> bool {
    name.ends_with('\u{00FF}')
}

#[must_use] 
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
                let x = LABEL_OFFSET + font.string_width("Replace:") as i32;
                let max_w = RIGHT_EDGE.saturating_sub(x).max(0);
                let name = &player_names[profile.replace - 1];
                shorten_name(name, font, max_w)
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

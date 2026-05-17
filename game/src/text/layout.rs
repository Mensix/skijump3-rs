use crate::parsers::langbase::LangBase;
use engine::ui::Font;

const RIGHT_EDGE: i32 = 316;

#[must_use]
pub fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|&b| b != b' ' && b != b'\r')
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|&b| b != b' ' && b != b'\r')
        .map_or(0, |p| p + 1);
    &bytes[start..end]
}

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

fn replace_right_text(value: usize) -> String {
    format!("#{value}")
}

fn replace_max_width(value: usize, font: &Font, x: i32) -> i32 {
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

#[must_use]
pub fn is_computer_name(name: &str) -> bool {
    name.ends_with('\u{00FF}')
}

pub fn lstr(langbase: &LangBase, index: usize, fallback: &str) -> String {
    let v = langbase.lstr(index);
    if v == "?" {
        fallback.to_string()
    } else {
        v.to_string()
    }
}

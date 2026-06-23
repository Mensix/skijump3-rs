use engine::oxide::Font;

const RIGHT_EDGE: i32 = 316;

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

pub fn replace_display_name(
    value: Option<usize>,
    player_names: &[String],
    font: &Font,
    x: i32,
) -> String {
    let Some(n) = value else { return String::new() };
    if n >= player_names.len() {
        return String::new();
    }
    let name = &player_names[n];
    let max_w = replace_max_width(n + 1, font, x);
    shorten_name(name, font, max_w)
}

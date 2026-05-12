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

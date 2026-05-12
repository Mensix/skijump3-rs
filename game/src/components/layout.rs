use engine::ui::Element;

const FONT_DEFAULT: u8 = 240;

pub fn top_right_text(version: &str) -> Vec<Element> {
    vec![
        Element::text_color_right("SKI JUMP", 308, 6, FONT_DEFAULT),
        Element::text_color_right("INTERNATIONAL", 308, 18, FONT_DEFAULT),
        Element::text_color(format!("v{}", version), 245, 30, FONT_DEFAULT),
    ]
}

pub fn header_elements(text: &str, x: i32, y: i32, color: u8, bg: u8) -> Vec<Element> {
    vec![
        Element::fillbox(x, y, 100, 6, bg),
        Element::text_color(text, x, y, color),
    ]
}

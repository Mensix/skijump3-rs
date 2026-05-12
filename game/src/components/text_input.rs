use engine::ui::{Component, Element, Event, Font, Key};

pub enum TextInputAction {
    Commit(String),
    Cancel,
}

pub struct TextInput {
    x: i32,
    y: i32,
    max_width: i32,
    bg: u8,
    fg: u8,
    font: Font,
    old: String,
    buf: String,
}

impl TextInput {
    pub fn new(x: i32, y: i32, max_width: i32, old: String, bg: u8, fg: u8, font: Font) -> Self {
        Self {
            x,
            y,
            max_width,
            bg,
            fg,
            font,
            old: old.clone(),
            buf: old,
        }
    }

    pub fn old(&self) -> &str {
        &self.old
    }
}

impl Component for TextInput {
    type Action = TextInputAction;

    fn elements(&self) -> Vec<Element> {
        let cx = self.x + self.font.string_width(&self.buf) as i32;
        vec![
            Element::fillbox(self.x - 2, self.y - 2, self.max_width + 4, 10, self.bg),
            Element::text_color(&self.buf, self.x, self.y, self.fg),
            Element::fillbox(cx, self.y + 6, 5, 1, self.fg),
        ]
    }

    fn handle_event(&mut self, event: &Event) -> Option<Self::Action> {
        match event {
            Event::Keyboard(Key::Backspace) => {
                self.buf.pop();
                None
            }
            Event::Keyboard(Key::Delete) => {
                self.buf.clear();
                None
            }
            Event::Keyboard(Key::Enter) => Some(TextInputAction::Commit(self.buf.clone())),
            Event::Keyboard(Key::Escape) => Some(TextInputAction::Cancel),
            Event::Keyboard(Key::Char(c)) if (' '..='~').contains(c) => {
                let mut next = self.buf.clone();
                next.push(*c);
                if self.font.string_width(&next) as i32 + 7 < self.max_width {
                    self.buf = next;
                }
                None
            }
            _ => None,
        }
    }
}

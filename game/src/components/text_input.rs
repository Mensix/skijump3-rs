use engine::ui::{Blinker, Component, Element, Event, Font, Key};

#[derive(Debug)]
pub enum TextInputAction {
    Commit(String),
    Cancel,
}

#[derive(Debug)]
pub struct TextInput {
    x: i32,
    y: i32,
    max_width: i32,
    bg: u8,
    fg: u8,
    font: Font,
    old: String,
    buf: String,
    blinker: Blinker,
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
            blinker: Blinker::new(),
        }
    }

    pub fn old(&self) -> &str {
        &self.old
    }

    fn reset_cursor(&self) {
        self.blinker.reset();
    }
}

impl Component for TextInput {
    type Action = TextInputAction;

    fn elements(&self) -> Vec<Element> {
        let cx = self.x + self.font.string_width(&self.buf) as i32;
        let mut els = vec![
            Element::fillbox(self.x - 2, self.y - 2, self.max_width + 4, 10, self.bg),
            Element::text_color(&self.buf, self.x, self.y, self.fg),
        ];
        if self.blinker.visible(11, 10) {
            els.push(Element::fillbox(cx, self.y + 6, 5, 1, 240));
        }
        els
    }

    fn handle_event(&mut self, event: &Event) -> Option<Self::Action> {
        self.reset_cursor();
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
            Event::Keyboard(Key::Char(c)) if *c >= ' ' => {
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

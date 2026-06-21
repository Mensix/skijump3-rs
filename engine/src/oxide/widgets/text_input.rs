use crate::color::Rgba;
use crate::oxide::input::{Key, UiEvent};
use crate::oxide::paint::PaintCx;
use crate::oxide::widget::{EventCx, Widget};
use crate::oxide::{Blinker, Font, TextEditState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextInputMessage {
    Commit(String),
    Cancel,
}

#[derive(Debug)]
pub struct TextInput {
    x: i32,
    y: i32,
    max_width: i32,
    bg: Rgba,
    fg: Rgba,
    cursor: Rgba,
    font: Font,
    editor: TextEditState,
    blinker: Blinker,
}

impl TextInput {
    pub fn new(
        x: i32,
        y: i32,
        max_width: i32,
        initial: impl Into<String>,
        max_chars: usize,
        bg: Rgba,
        fg: Rgba,
        cursor: Rgba,
        font: Font,
    ) -> Self {
        Self {
            x,
            y,
            max_width,
            bg,
            fg,
            cursor,
            font,
            editor: TextEditState::new(initial.into(), max_chars),
            blinker: Blinker::new(),
        }
    }

    pub fn value(&self) -> &str {
        self.editor.buffer()
    }
}

impl Widget for TextInput {
    type Message = TextInputMessage;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        self.blinker.reset();
        let msg = match event {
            UiEvent::KeyDown(Key::Backspace) => {
                self.editor.backspace();
                None
            }
            UiEvent::KeyDown(Key::Delete) => {
                self.editor.set_buffer(String::new());
                None
            }
            UiEvent::KeyDown(Key::Enter) => {
                Some(TextInputMessage::Commit(self.editor.buffer().to_string()))
            }
            UiEvent::KeyDown(Key::Escape) => Some(TextInputMessage::Cancel),
            UiEvent::Text(c) if c >= ' ' => {
                if self.font.string_width(self.editor.buffer()) as i32 + 7 < self.max_width {
                    self.editor.insert(c);
                }
                None
            }
            _ => return None,
        };
        cx.consume();
        msg
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        let text = self.editor.buffer();
        let cursor_x = self.x + self.font.string_width(text) as i32;
        cx.fill((self.x - 2, self.y - 2, self.max_width + 4, 10), self.bg);
        cx.text((self.x, self.y), self.fg, text);
        if self.blinker.visible(11, 10) {
            cx.fill((cursor_x, self.y + 6, 5, 1), self.cursor);
        }
    }
}

use engine::oxide::blinker::Blinker;
use engine::oxide::input::{Key, UiEvent};
use engine::oxide::paint::PaintCx;

use crate::gfx::theme::{BG_RED, BLACK, FONT_BODY, FONT_GOLD, FONT_TEAL};

const MAX_INPUT_CHARS: usize = 50;
const MAX_LOG_LINES: usize = 50;
const VISIBLE_LINES: usize = 5;
const LINE_H: i32 = 8;

pub enum ChatAction {
    Send(String),
    Consumed,
    None,
}

#[derive(Debug, Default)]
pub struct ChatPanel {
    lines: Vec<ChatLine>,
    pub input: String,
    blinker: Blinker,
}

#[derive(Debug)]
enum ChatLine {
    System(String),
    Message { from: String, text: String },
}

impl ChatPanel {
    pub fn tick(&mut self) {}

    pub fn event(&mut self, event: UiEvent) -> ChatAction {
        self.blinker.reset();
        match event {
            UiEvent::Text(c) if c >= ' ' => {
                if self.input.len() < MAX_INPUT_CHARS {
                    self.input.push(c);
                }
                ChatAction::Consumed
            }
            UiEvent::KeyDown(Key::Backspace) => {
                self.input.pop();
                ChatAction::Consumed
            }
            UiEvent::KeyDown(Key::Enter) => {
                let text = self.input.trim().to_string();
                self.input.clear();
                if text.is_empty() {
                    ChatAction::Consumed
                } else {
                    ChatAction::Send(text)
                }
            }
            UiEvent::KeyDown(Key::Escape) if !self.input.is_empty() => {
                self.input.clear();
                ChatAction::Consumed
            }
            _ => ChatAction::None,
        }
    }

    pub fn push_system(&mut self, text: impl Into<String>) {
        self.push(ChatLine::System(text.into()));
    }

    pub fn push_message(&mut self, from: impl Into<String>, text: impl Into<String>) {
        self.push(ChatLine::Message {
            from: from.into(),
            text: text.into(),
        });
    }

    pub fn paint(&self, paint: &mut PaintCx<'_>, top_y: i32, local_name: &str) {
        let pad = 2;
        let input_y = top_y + 1 + pad + VISIBLE_LINES as i32 * LINE_H + pad;
        let sep = input_y;
        let panel_h = input_y + LINE_H + pad * 2 - top_y;

        paint.fill((0, top_y, 320, 1), BLACK);
        paint.pattern_fill((0, top_y + 1, 320, sep - top_y - 1), BG_RED);
        paint.fill((0, sep, 320, panel_h - (sep - top_y)), BLACK);

        let msg_y = top_y + 1 + pad;
        let start = self.lines.len().saturating_sub(VISIBLE_LINES);
        for (row, line) in self.lines[start..].iter().enumerate() {
            let y = msg_y + (row as i32) * LINE_H;
            match line {
                ChatLine::System(text) => {
                    paint.text((5, y), FONT_TEAL, format!("* {text}"));
                }
                ChatLine::Message { from, text } => {
                    paint.text((5, y), FONT_GOLD, format!("{from}: "));
                    paint.text((5 + (from.len() as i32 + 2) * 5, y), FONT_BODY, text);
                }
            }
        }

        let prompt = format!("{local_name}: ");
        let text_y = sep + pad;
        paint.text((5, text_y), FONT_GOLD, &prompt);
        let cursor_x = 5 + prompt.len() as i32 * 5;
        if self.input.is_empty() {
            if self.blinker.visible(30, 30) {
                paint.fill((cursor_x, text_y + 5, 5, 1), FONT_BODY);
            }
        } else {
            paint.text((cursor_x, text_y), FONT_BODY, &self.input);
            let end_x = cursor_x + self.input.len() as i32 * 5;
            if self.blinker.visible(30, 30) {
                paint.fill((end_x, text_y + 5, 5, 1), FONT_BODY);
            }
        }
    }

    fn push(&mut self, line: ChatLine) {
        self.lines.push(line);
        if self.lines.len() > MAX_LOG_LINES {
            self.lines.remove(0);
        }
    }
}

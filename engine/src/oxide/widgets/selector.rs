use crate::color::Rgba;
use crate::oxide::input::{Key, UiEvent};
use crate::oxide::paint::PaintCx;
use crate::oxide::widget::{EventCx, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorMessage {
    Commit(usize),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct NumericSelector {
    x: i32,
    y: i32,
    width: i32,
    max: usize,
    value: usize,
    bg: Rgba,
    fg: Rgba,
    display: String,
    wrap: bool,
}

impl NumericSelector {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        max: usize,
        value: usize,
        bg: Rgba,
        fg: Rgba,
        display: impl Into<String>,
    ) -> Self {
        Self {
            x,
            y,
            width,
            max,
            value: value.min(max),
            bg,
            fg,
            display: display.into(),
            wrap: true,
        }
    }

    pub const fn value(&self) -> usize {
        self.value
    }

    pub const fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    fn decrement(&mut self, step: usize) {
        if self.wrap && self.value == 0 {
            self.value = self.max;
        } else {
            self.value = self.value.saturating_sub(step);
        }
    }

    fn increment(&mut self, step: usize) {
        if self.wrap && self.value == self.max {
            self.value = 0;
        } else {
            self.value = (self.value + step).min(self.max);
        }
    }
}

impl Widget for NumericSelector {
    type Message = SelectorMessage;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        let msg = match event {
            UiEvent::KeyDown(Key::Up | Key::Left) => {
                self.decrement(1);
                None
            }
            UiEvent::KeyDown(Key::Down | Key::Right) => {
                self.increment(1);
                None
            }
            UiEvent::KeyDown(Key::Home) => {
                self.value = 0;
                None
            }
            UiEvent::KeyDown(Key::End) => {
                self.value = self.max;
                None
            }
            UiEvent::KeyDown(Key::PageUp) => {
                self.decrement(10);
                None
            }
            UiEvent::KeyDown(Key::PageDown) => {
                self.increment(10);
                None
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                Some(SelectorMessage::Commit(self.value))
            }
            UiEvent::KeyDown(Key::Escape) => Some(SelectorMessage::Cancel),
            _ => return None,
        };
        cx.consume();
        msg
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        let text = if self.display.is_empty() {
            self.value.to_string()
        } else {
            self.display.clone()
        };
        cx.fill((self.x - 2, self.y - 1, self.width, 8), self.bg);
        cx.text((self.x, self.y), self.fg, text);
    }
}

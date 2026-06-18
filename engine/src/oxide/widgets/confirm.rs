use crate::color::Rgba;
use crate::oxide::input::{Key, UiEvent};
use crate::oxide::paint::PaintCx;
use crate::oxide::widget::{EventCx, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmMessage {
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    bg: Rgba,
    border: Rgba,
    fg: Rgba,
    message: String,
    subtitle: Option<String>,
    yes: String,
    no: String,
}

impl ConfirmDialog {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        rect: (i32, i32, i32, i32),
        bg: Rgba,
        border: Rgba,
        fg: Rgba,
        message: impl Into<String>,
        yes: impl Into<String>,
        no: impl Into<String>,
    ) -> Self {
        let (x, y, w, h) = rect;
        Self {
            x,
            y,
            w,
            h,
            bg,
            border,
            fg,
            message: message.into(),
            subtitle: None,
            yes: yes.into(),
            no: no.into(),
        }
    }

    #[must_use]
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

impl Widget for ConfirmDialog {
    type Message = ConfirmMessage;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        let msg = match event {
            UiEvent::Text('y' | 'Y') => Some(ConfirmMessage::Yes),
            UiEvent::Text('n' | 'N') | UiEvent::KeyDown(Key::Escape) => Some(ConfirmMessage::No),
            _ => None,
        };
        if msg.is_some() {
            cx.consume();
        }
        msg
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        cx.fill((self.x, self.y, self.w, self.h), self.border);
        cx.pattern_fill((self.x + 1, self.y + 1, self.w - 2, self.h - 2), self.bg);
        cx.text((self.x + 8, self.y + 8), self.fg, &self.message);
        if let Some(sub) = &self.subtitle {
            cx.text((self.x + 8, self.y + self.h - 22), self.fg, sub);
        }
        cx.text(
            (self.x + 8, self.y + self.h - 14),
            self.fg,
            format!("{}/{}", self.yes, self.no),
        );
    }
}

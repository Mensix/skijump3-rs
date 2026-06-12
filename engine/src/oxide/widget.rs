use crate::oxide::input::UiEvent;
use crate::oxide::paint::PaintCx;

#[derive(Default)]
pub struct EventCx {
    consumed: bool,
}

impl EventCx {
    pub fn consume(&mut self) {
        self.consumed = true;
    }

    #[must_use]
    pub const fn is_consumed(&self) -> bool {
        self.consumed
    }
}

pub trait Widget {
    type Message;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message>;
    fn paint(&self, cx: &mut PaintCx<'_>);
}

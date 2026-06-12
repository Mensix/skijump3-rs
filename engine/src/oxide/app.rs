use crate::oxide::input::UiEvent;
use crate::oxide::paint::PaintCx;
use crate::oxide::route::NavAction;
use crate::oxide::widget::UpdateCx;

pub struct ScreenEventCx<R> {
    action: NavAction<R>,
    consumed: bool,
}

impl<R> Default for ScreenEventCx<R> {
    fn default() -> Self {
        Self {
            action: NavAction::None,
            consumed: false,
        }
    }
}

impl<R> ScreenEventCx<R> {
    pub fn consume(&mut self) {
        self.consumed = true;
    }

    pub fn navigate(&mut self, route: R) {
        self.action = NavAction::Navigate(route);
        self.consume();
    }

    pub fn back(&mut self) {
        self.action = NavAction::Back;
        self.consume();
    }

    pub fn quit(&mut self) {
        self.action = NavAction::Quit;
        self.consume();
    }

    #[must_use]
    pub const fn is_consumed(&self) -> bool {
        self.consumed
    }

    pub fn take_action(&mut self) -> NavAction<R> {
        std::mem::replace(&mut self.action, NavAction::None)
    }
}

pub trait Screen<R> {
    fn update(&mut self, _cx: &mut UpdateCx) {}
    fn event(&mut self, cx: &mut ScreenEventCx<R>, event: UiEvent);
    fn paint(&self, cx: &mut PaintCx<'_>);
}

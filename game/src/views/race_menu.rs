use engine::ui::{
    Element, Event, Key, View, RouteTarget, Font,
    paint::PaintCtx,
};

pub struct RaceMenuView {
    selected: usize,
}

impl RaceMenuView {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl View for RaceMenuView {
    fn elements(&self) -> Vec<Element> {
        vec![]
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::MainMenu),
            Event::Keyboard(Key::Up) => {
                self.selected = self.selected.saturating_sub(1);
                None
            }
            Event::Keyboard(Key::Down) => {
                self.selected += 1;
                None
            }
            Event::Keyboard(Key::Enter) => Some(RouteTarget::Play(0)),
            _ => None,
        }
    }

    fn route(&self) -> Option<RouteTarget> {
        Some(RouteTarget::RaceMenu)
    }

    fn paint(&self, _ctx: &mut PaintCtx, _sprites: &[Vec<u8>], _font: &Font) {
    }
}
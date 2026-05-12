use crate::palette::Palette;
use crate::ui::Element;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Keyboard(Key),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Escape,
    Backspace,
    Delete,
    Char(char),
}

pub trait View<T: Clone + PartialEq + 'static> {
    fn elements(&self) -> Vec<Element>;
    fn handle_event(&mut self, event: Event) -> Option<T>;
    fn apply_palette(&self, _palette: &mut Palette) {}
}

#[allow(clippy::type_complexity)]
pub struct Router<T: Clone + PartialEq + 'static> {
    current: Box<dyn View<T>>,
    current_route: Option<T>,
    history: Vec<T>,
    routes: Vec<(T, Box<dyn Fn() -> Box<dyn View<T>>>)>,
}

impl<T: Clone + PartialEq + 'static> Router<T> {
    #[allow(clippy::type_complexity)]
    pub fn new(
        route: T,
        initial: Box<dyn View<T>>,
        routes: Vec<(T, Box<dyn Fn() -> Box<dyn View<T>>>)>,
    ) -> Self {
        Self {
            current: initial,
            current_route: Some(route),
            history: Vec::with_capacity(4),
            routes,
        }
    }

    pub fn navigate(&mut self, target: T) {
        if let Some(idx) = self.routes.iter().position(|(t, _)| *t == target) {
            if let Some(prev) = self.current_route.clone() {
                self.history.push(prev);
            }
            self.current_route = Some(target);
            self.current = (self.routes[idx].1)();
        }
    }

    pub fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            if let Some(idx) = self.routes.iter().position(|(t, _)| *t == prev) {
                self.current_route = Some(prev);
                self.current = (self.routes[idx].1)();
            }
        }
    }

    pub fn current_view(&self) -> &dyn View<T> {
        &*self.current
    }

    pub fn current_view_mut(&mut self) -> &mut dyn View<T> {
        &mut *self.current
    }

    pub fn handle_event(&mut self, event: &Event) {
        if let Some(target) = self.current.handle_event(*event) {
            self.navigate(target);
        }
    }

    pub fn current_route(&self) -> Option<&T> {
        self.current_route.as_ref()
    }

    pub fn apply_palette(&self, palette: &mut Palette) {
        self.current.apply_palette(palette);
    }
}

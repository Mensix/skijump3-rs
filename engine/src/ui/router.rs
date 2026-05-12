use crate::ui::Element;

#[derive(Clone, Debug)]
pub enum Event {
    Keyboard(Key),
    Click { x: i32, y: i32 },
    Timer(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    Char(char),
}

pub trait View<T: Clone + PartialEq + 'static>: Send + Sync {
    fn elements(&self) -> Vec<Element>;
    fn handle_event(&mut self, event: Event) -> Option<T>;
    fn route(&self) -> Option<T>;
}

pub struct Router<T: Clone + PartialEq + 'static> {
    current: Box<dyn View<T>>,
    current_route: Option<T>,
    history: Vec<T>,
    routes: Vec<(T, Box<dyn Fn() -> Box<dyn View<T>> + Send + Sync>)>,
}

impl<T: Clone + PartialEq + 'static> Router<T> {
    pub fn new(
        route: T,
        initial: Box<dyn View<T>>,
        routes: Vec<(T, Box<dyn Fn() -> Box<dyn View<T>> + Send + Sync>)>,
    ) -> Self {
        Self { current: initial, current_route: Some(route), history: Vec::new(), routes }
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
            self.navigate(prev);
        }
    }

    pub fn current_view(&self) -> &dyn View<T> {
        &*self.current
    }

    pub fn current_view_mut(&mut self) -> &mut dyn View<T> {
        &mut *self.current
    }

    pub fn handle_event(&mut self, event: &Event) {
        if let Some(target) = self.current.handle_event(event.clone()) {
            self.navigate(target);
        }
    }

    pub fn current_route(&self) -> Option<&T> {
        self.current_route.as_ref()
    }
}

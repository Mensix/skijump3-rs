use crate::ui::element::RouteTarget;

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

pub trait View: Send + Sync {
    fn elements(&self) -> Vec<crate::ui::Element>;
    fn handle_event(&mut self, event: Event) -> Option<RouteTarget>;
    fn route(&self) -> Option<RouteTarget>;
}

pub struct Router {
    current: Box<dyn View>,
    current_route: Option<RouteTarget>,
    history: Vec<RouteTarget>,
    routes: Vec<(RouteTarget, Box<dyn Fn() -> Box<dyn View> + Send + Sync>)>,
}

impl Router {
    pub fn new(route: RouteTarget, initial: Box<dyn View>, routes: Vec<(RouteTarget, Box<dyn Fn() -> Box<dyn View> + Send + Sync>)>) -> Self {
        Self { current: initial, current_route: Some(route), history: Vec::new(), routes }
    }

    pub fn navigate(&mut self, target: RouteTarget) {
        if let Some(idx) = self.routes.iter().position(|(t, _)| *t == target) {
            if let Some(prev) = self.current_route {
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

    pub fn current_view(&self) -> &dyn View {
        &*self.current
    }

    pub fn current_view_mut(&mut self) -> &mut dyn View {
        &mut *self.current
    }

    pub fn handle_event(&mut self, event: Event) {
        if let Some(target) = self.current.handle_event(event) {
            self.navigate(target);
        }
    }

    pub fn current_route(&self) -> Option<RouteTarget> {
        self.current_route
    }
}
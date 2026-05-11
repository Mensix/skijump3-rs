use crate::ui::paint::PaintCtx;
use std::any::Any;

pub trait Route: 'static + Send + Sync {
    fn name(&self) -> &str;
    fn eq(&self, other: &dyn Route) -> bool;
}

pub trait View: Send + Sync {
    fn paint(&self, ctx: &mut PaintCtx);
    fn handle_event(&mut self, event: Event);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub enum Event {
    Keyboard(Key),
    Click { x: i32, y: i32 },
    Timer(u32),
}

pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    Char(char),
}

pub struct Router<R: Route> {
    current: Box<dyn View>,
    routes: Vec<(Box<dyn Fn(&R) -> Box<dyn View>>, R)>,
}

impl<R: Route + Clone + 'static> Router<R> {
    pub fn new(initial: Box<dyn View>, routes: Vec<(Box<dyn Fn(&R) -> Box<dyn View>>, R)>) -> Self {
        Self { current: initial, routes }
    }

    pub fn navigate(&mut self, route: R)
    where
        R: Clone,
    {
        for (builder, r) in &self.routes {
            if route.eq(r) {
                self.current = builder(&route);
                return;
            }
        }
    }

    pub fn current_view(&self) -> &dyn View {
        &*self.current
    }

    pub fn current_view_mut(&mut self) -> &mut dyn View {
        self.current.as_mut()
    }

    pub fn handle_event(&mut self, event: Event) {
        self.current.handle_event(event);
    }

    pub fn paint(&self, ctx: &mut PaintCtx) {
        self.current.paint(ctx);
    }
}
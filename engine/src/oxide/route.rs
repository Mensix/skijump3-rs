#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavAction<R> {
    None,
    Navigate(R),
    Back,
    Quit,
}

#[derive(Debug, Clone)]
pub struct Navigator<R> {
    current: R,
    history: Vec<R>,
}

impl<R: Clone + PartialEq> Navigator<R> {
    #[must_use]
    pub fn new(current: R) -> Self {
        Self {
            current,
            history: Vec::with_capacity(4),
        }
    }

    #[must_use]
    pub fn current(&self) -> &R {
        &self.current
    }

    pub fn navigate(&mut self, route: R) {
        if self.current != route {
            self.history.push(self.current.clone());
            self.current = route;
        }
    }

    pub fn back(&mut self) {
        if let Some(route) = self.history.pop() {
            self.current = route;
        }
    }
}

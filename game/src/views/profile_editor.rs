use engine::ui::{Element, Event, Key, View};
use crate::store::StoreRef;
use crate::route::RouteTarget;

pub struct ProfileEditor {
    store: StoreRef,
}

impl ProfileEditor {
    pub fn new(store: StoreRef) -> Self {
        Self { store }
    }
}

impl View<RouteTarget> for ProfileEditor {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];
        let index = self.store.borrow().profiles.edit_index;
        let name = self.store.borrow().profiles.profiles.get(index)
            .map(|p| p.name.clone()).unwrap_or_default();

        els.push(Element::fillbox(0, 0, 320, 200, 0));
        els.push(Element::text_color(format!("Editing: {}", name), 40, 40, 240));
        els.push(Element::text_color("Press Escape to go back", 40, 50, 241));
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::ProfilesList),
            _ => None,
        }
    }
}

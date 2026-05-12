use engine::ui::{Component, Element, Event, Key};

pub enum ConfirmAction {
    Yes,
    No,
}

pub struct ConfirmDialog {
    message: String,
}

impl ConfirmDialog {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Component for ConfirmDialog {
    type Action = ConfirmAction;

    fn elements(&self) -> Vec<Element> {
        vec![
            Element::fillbox(59, 79, 203, 53, 242),
            Element::fillbox(60, 80, 201, 51, 244),
            Element::FillArea { thing: 63 },
            Element::text_color(&self.message, 70, 90, 246),
            Element::text_color("Are you sure?", 70, 110, 246),
            Element::text_color("(Y/N)", 165, 110, 241),
        ]
    }

    fn handle_event(&mut self, event: &Event) -> Option<Self::Action> {
        match event {
            Event::Keyboard(Key::Char('y') | Key::Char('Y')) => Some(ConfirmAction::Yes),
            Event::Keyboard(Key::Char('n') | Key::Char('N') | Key::Escape) => {
                Some(ConfirmAction::No)
            }
            _ => None,
        }
    }
}

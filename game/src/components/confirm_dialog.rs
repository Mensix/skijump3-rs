use crate::parsers::langbase::LangBase;
use engine::ui::{Component, Element, Event, Font, Key};
use std::rc::Rc;

pub enum ConfirmAction {
    Yes,
    No,
}

pub struct ConfirmDialog {
    message: String,
    langbase: Rc<LangBase>,
    font: Font,
}

impl ConfirmDialog {
    pub fn new(message: String, langbase: Rc<LangBase>, font: Font) -> Self {
        Self {
            message,
            langbase,
            font,
        }
    }

    fn lstr(&self, index: usize, fallback: &str) -> String {
        let v = self.langbase.lstr(index);
        if v == "?" {
            fallback.to_string()
        } else {
            v.to_string()
        }
    }
}

impl Component for ConfirmDialog {
    type Action = ConfirmAction;

    fn elements(&self) -> Vec<Element> {
        let str2 = self.lstr(193, "Are you sure?");
        let hint_x = 70 + self.font.string_width(&str2) as i32 + 4;
        vec![
            Element::fillbox(59, 79, 203, 53, 242),
            Element::fillbox(60, 80, 201, 51, 244),
            Element::FillArea { thing: 63 },
            Element::text_color(&self.message, 70, 90, 246),
            Element::text_color(str2, 70, 110, 246),
            Element::text_color("(Y/N)", hint_x, 110, 241),
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

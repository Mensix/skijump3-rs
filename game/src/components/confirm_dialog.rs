use crate::parsers::langbase::LangBase;
use crate::text::layout::lstr;
use engine::ui::{Blinker, Component, Element, Event, Font, Key};
use std::rc::Rc;

#[derive(Debug)]
pub enum ConfirmAction {
    Yes,
    No,
}

#[derive(Debug)]
pub struct ConfirmDialog {
    message: String,
    langbase: Rc<LangBase>,
    font: Font,
    blinker: Blinker,
}

impl ConfirmDialog {
    #[must_use]
    pub fn new(message: String, langbase: Rc<LangBase>, font: Font) -> Self {
        Self {
            message,
            langbase,
            font,
            blinker: Blinker::new(),
        }
    }
}

impl Component for ConfirmDialog {
    type Action = ConfirmAction;

    fn elements(&self) -> Vec<Element> {
        let str2 = lstr(&self.langbase, 193, "Are you sure?");
        let hint_x = 70 + self.font.string_width(&str2) as i32 + 4;
        let cursor_x = hint_x + 25;
        let mut els = vec![
            Element::fillbox(59, 79, 203, 53, 242),
            Element::fillbox(60, 80, 201, 51, 244),
            Element::FillArea { thing: 63 },
            Element::text(&self.message, 70, 90, 246, false),
            Element::text(str2, 70, 110, 246, false),
            Element::text("(Y/N)", hint_x, 110, 241, false),
            Element::fillbox(cursor_x - 2, 108, 9, 11, 243),
        ];
        if self.blinker.visible(11, 10) {
            els.push(Element::fillbox(cursor_x, 116, 5, 1, 240));
        }
        els
    }

    fn handle_event(&mut self, event: &Event) -> Option<Self::Action> {
        self.blinker.reset();
        match event {
            Event::Keyboard(Key::Char('y' | 'Y')) => Some(ConfirmAction::Yes),
            Event::Keyboard(Key::Char('n' | 'N') | Key::Escape) => Some(ConfirmAction::No),
            _ => None,
        }
    }
}

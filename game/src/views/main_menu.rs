use std::sync::Arc;
use engine::ui::{Element, Event, View, Component};
use crate::parsers::langbase::LangBase;
use crate::components::menu::{Menu, MenuItem};
use crate::components::layout;
use crate::route::RouteTarget;

const FONT_DEFAULT: u8 = 240;
const FONT_HEADER: u8 = 246;
const BG_ERASE: u8 = 8;

pub struct MainMenuView {
    menu: Menu,
    langbase: Arc<LangBase>,
    version: String,
}

impl MainMenuView {
    pub fn new(langbase: Arc<LangBase>, version: String) -> Self {
        let items = vec![
            MenuItem { num: 1, label: 20, y_off: 0 },
            MenuItem { num: 2, label: 21, y_off: 0 },
            MenuItem { num: 3, label: 22, y_off: 0 },
            MenuItem { num: 4, label: 23, y_off: 0 },
            MenuItem { num: 5, label: 24, y_off: 0 },
            MenuItem { num: 6, label: 25, y_off: 0 },
            MenuItem { num: 0, label: 26, y_off: 12 },
        ];
        Self {
            menu: Menu::new(11, 97, 108, 12, items, &langbase, FONT_DEFAULT, FONT_DEFAULT),
            langbase,
            version,
        }
    }
}

impl View<RouteTarget> for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        els.extend(layout::header_elements(self.langbase.lstr(17), 11, 80, FONT_HEADER, BG_ERASE));

        els.extend(self.menu.elements());

        els.push(Element::text_color(self.langbase.lstr(34), 170, 51, FONT_DEFAULT));

        els.extend(layout::top_right_text(&self.version));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(0) => Some(RouteTarget::Quit),
            Some(1) => Some(RouteTarget::JumpMenu),
            Some(2) => Some(RouteTarget::Profiles),
            Some(3) => Some(RouteTarget::OptionsMenu),
            Some(4) => Some(RouteTarget::MainMenu),
            Some(5) => Some(RouteTarget::MainMenu),
            Some(6) => Some(RouteTarget::MainMenu),
            Some(7) => Some(RouteTarget::Quit),
            _ => None,
        }
    }
}

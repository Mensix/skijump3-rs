use std::sync::Arc;
use engine::ui::{Element, Event, View, Component};
use crate::parsers::langbase::LangBase;
use crate::components::menu::{Menu, MenuItem};
use crate::components::layout;
use crate::route::RouteTarget;

const FONT_DEFAULT: u8 = 240;
const FONT_HEADER: u8 = 246;
const BG_ERASE: u8 = 8;
const BG_LIST: u8 = 8;

pub struct JumpMenuView {
    menu: Menu,
    langbase: Arc<LangBase>,
    version: String,
}

impl JumpMenuView {
    pub fn new(langbase: Arc<LangBase>, version: String) -> Self {
        let items = vec![
            MenuItem { num: 1, label: 27, y_off: 0 },
            MenuItem { num: 2, label: 28, y_off: 0 },
            MenuItem { num: 3, label: 29, y_off: 0 },
            MenuItem { num: 4, label: 30, y_off: 0 },
            MenuItem { num: 5, label: 31, y_off: 0 },
            MenuItem { num: 6, label: 32, y_off: 0 },
            MenuItem { num: 0, label: 33, y_off: 12 },
        ];
        Self {
            menu: Menu::new(11, 97, 108, 12, items, &langbase, FONT_DEFAULT, FONT_DEFAULT),
            langbase,
            version,
        }
    }
}

impl View<RouteTarget> for JumpMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        // FillBox(1,94,116,199,8) — background for item list
        els.push(Element::fillbox(1, 94, 116, 106, BG_LIST));

        // header from MainMenuText(1): lstr(18) at (11,80) fontcolor(246)
        els.extend(layout::header_elements(self.langbase.lstr(18), 11, 80, FONT_HEADER, BG_ERASE));

        // items from Menu component
        els.extend(self.menu.elements());

        // right side text
        els.extend(layout::top_right_text(&self.version));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(0) => Some(RouteTarget::MainMenu),
            Some(1) => Some(RouteTarget::MainMenu),
            Some(2) => Some(RouteTarget::MainMenu),
            Some(3) => Some(RouteTarget::MainMenu),
            Some(4) => Some(RouteTarget::MainMenu),
            Some(5) => Some(RouteTarget::MainMenu),
            Some(6) => Some(RouteTarget::MainMenu),
            Some(7) => Some(RouteTarget::MainMenu),
            _ => None,
        }
    }
}

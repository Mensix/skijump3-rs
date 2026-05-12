use engine::ui::{Element, Event, View, Component};
use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::route::RouteTarget;

const FONT_DEFAULT: u8 = 240;
const FONT_HEADER: u8 = 246;
const BG_ERASE: u8 = 8;

pub struct MainMenuView {
    menu: Menu,
    layout: MainLayout,
}

impl MainMenuView {
    pub fn new(layout: MainLayout) -> Self {
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
            menu: Menu::new(11, 97, 108, 12, items, &layout.langbase, FONT_DEFAULT, FONT_DEFAULT),
            layout,
        }
    }
}

impl View<RouteTarget> for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut content = vec![];

        content.extend(layout::header_elements(self.layout.langbase.lstr(17), 11, 80, FONT_HEADER, BG_ERASE));
        content.extend(self.menu.elements());

        self.layout.wrap(content)
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

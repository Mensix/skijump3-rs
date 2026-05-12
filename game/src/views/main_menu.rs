use engine::ui::{Element, Event, View, Component};
use crate::parsers::langbase::LangBase;
use crate::components::menu::{Menu, MenuItem};
use crate::route::RouteTarget;

pub struct MainMenuView {
    menu: Menu,
    langbase: LangBase,
    version: String,
}

impl MainMenuView {
    pub fn new(langbase: LangBase, version: String) -> Self {
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
            menu: Menu::new(11, 97, 108, 12, items, &langbase, 240, 240),
            langbase,
            version,
        }
    }
}

impl View<RouteTarget> for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        els.push(Element::fillbox(11, 80, 100, 6, 8));

        els.push(Element::text_color(self.langbase.lstr(17), 11, 80, 246));

        els.extend(self.menu.elements());

        els.push(Element::text_color(self.langbase.lstr(34), 170, 51, 240));

        els.push(Element::text_color_right("SKI JUMP", 308, 6, 240));
        els.push(Element::text_color_right("SKI JUMP INTERNATIONAL", 308, 18, 240));
        els.push(Element::text_color_right(format!("v{}", self.version), 245, 30, 240));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(0) => Some(RouteTarget::Quit),
            Some(1) => Some(RouteTarget::Play(1)),
            Some(2) => Some(RouteTarget::Profiles),
            Some(3) => Some(RouteTarget::OptionsMenu),
            Some(4) => Some(RouteTarget::Play(1)),
            Some(5) => Some(RouteTarget::Play(1)),
            Some(6) => Some(RouteTarget::Play(1)),
            Some(7) => Some(RouteTarget::Quit),
            _ => None,
        }
    }

    fn route(&self) -> Option<RouteTarget> {
        Some(RouteTarget::MainMenu)
    }
}

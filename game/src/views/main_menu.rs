use engine::ui::{Element, Event, View, RouteTarget};
use crate::parsers::langbase::LangBase;
use crate::components::menu::{Menu, MenuItem};

pub struct MainMenuView {
    menu: Menu,
    langbase: LangBase,
    version: String,
}

impl MainMenuView {
    pub fn new(langbase: LangBase, version: String) -> Self {
        Self { menu: Menu::new(), langbase, version }
    }
}

impl View for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        // from MainMenuText(0): header lstr(17) at (11,80) fontcolor(246)
        els.push(Element::text_color(self.langbase.lstr(17), 11, 80, 246));

        // items from the Menu component
        let items = [
            MenuItem { num: 1, label: 20 },
            MenuItem { num: 2, label: 21 },
            MenuItem { num: 3, label: 22 },
            MenuItem { num: 4, label: 23 },
            MenuItem { num: 5, label: 24 },
            MenuItem { num: 6, label: 25 },
            MenuItem { num: 0, label: 26 },
        ];
        els.extend(self.menu.elements(11, 98, 108, 12, &items, &self.langbase, 240, 240));

        // from drawmainmenu: lstr(34) at (170,51) fontcolor(240)
        els.push(Element::text_color(self.langbase.lstr(34), 170, 51, 240));

        // from MainMenuText(0): right side text
        els.push(Element::text_color("SKI JUMP", 308, 6, 240));
        els.push(Element::text_color("SKI JUMP INTERNATIONAL", 308, 18, 240));
        els.push(Element::text_color(format!("v{}", self.version), 245, 30, 240));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event, 6) {
            Some(0) => Some(RouteTarget::Quit),
            Some(1) => Some(RouteTarget::Play(1)),
            Some(2) => Some(RouteTarget::Profiles),
            Some(3) => Some(RouteTarget::OptionsMenu),
            Some(4) => Some(RouteTarget::Play(1)),
            Some(5) => Some(RouteTarget::Play(1)),
            Some(6) => Some(RouteTarget::Play(1)),
            _ => None,
        }
    }

    fn route(&self) -> Option<RouteTarget> {
        Some(RouteTarget::MainMenu)
    }
}

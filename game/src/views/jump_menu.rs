use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::palette_consts::{FONT_DEFAULT, BG_LIST, FONT_HEADER, BG_ERASE};
use crate::route::RouteTarget;
use engine::ui::{Component, Element, Event, View};

pub struct JumpMenuView {
    menu: Menu,
    layout: MainLayout,
}

impl JumpMenuView {
    #[must_use] 
    pub fn new(layout: MainLayout) -> Self {
        let items = vec![
            MenuItem {
                num: 1,
                label: 27,
                y_off: 0,
            },
            MenuItem {
                num: 2,
                label: 28,
                y_off: 0,
            },
            MenuItem {
                num: 3,
                label: 29,
                y_off: 0,
            },
            MenuItem {
                num: 4,
                label: 30,
                y_off: 0,
            },
            MenuItem {
                num: 5,
                label: 31,
                y_off: 0,
            },
            MenuItem {
                num: 6,
                label: 32,
                y_off: 0,
            },
            MenuItem {
                num: 0,
                label: 33,
                y_off: 12,
            },
        ];
        Self {
            menu: Menu::new(
                11,
                97,
                108,
                12,
                items,
                &layout.langbase,
                FONT_DEFAULT,
                FONT_DEFAULT,
            ),
            layout,
        }
    }
}

impl View<RouteTarget> for JumpMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = self.layout.background();
        els.extend(self.layout.jumpers());
        els.extend(self.layout.registration());
        els.push(Element::fillbox(1, 94, 116, 106, BG_LIST));
        els.extend(layout::header_elements(
            self.layout.langbase.lstr(18),
            11,
            80,
            FONT_HEADER,
            BG_ERASE,
        ));
        els.extend(self.menu.elements());
        els.extend(self.layout.footer());
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(6) => Some(RouteTarget::Practice),
            Some(0) | Some(1) | Some(2) | Some(3) | Some(4) | Some(5) | Some(7) => {
                Some(RouteTarget::MainMenu)
            }
            _ => None,
        }
    }
}

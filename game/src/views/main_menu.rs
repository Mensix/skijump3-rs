use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::StoreRef;
use engine::ui::{Component, Element, Event, View};

pub struct MainMenuView {
    menu: Menu,
    layout: MainLayout,
    store: StoreRef,
}

impl MainMenuView {
    pub fn new(layout: MainLayout, store: StoreRef) -> Self {
        let items = vec![
            MenuItem {
                num: 1,
                label: 20,
                y_off: 0,
            },
            MenuItem {
                num: 2,
                label: 21,
                y_off: 0,
            },
            MenuItem {
                num: 3,
                label: 22,
                y_off: 0,
            },
            MenuItem {
                num: 4,
                label: 23,
                y_off: 0,
            },
            MenuItem {
                num: 5,
                label: 24,
                y_off: 0,
            },
            MenuItem {
                num: 6,
                label: 25,
                y_off: 0,
            },
            MenuItem {
                num: 0,
                label: 26,
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
            store,
        }
    }
}

impl View<RouteTarget> for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut content = vec![];

        content.extend(layout::header_elements(
            self.layout.langbase.lstr(17),
            11,
            80,
            FONT_HEADER,
            BG_ERASE,
        ));
        content.extend(self.menu.elements());

        // Profile names (Pascal drawmainmenu)
        let pb = self.store.profiles.borrow();
        for (i, &profile_idx) in pb.active_order.iter().enumerate() {
            if profile_idx >= pb.profiles.len() {
                continue;
            }
            let profile = &pb.profiles[profile_idx];
            let y = (i as i32) * 9 + 64;
            content.push(Element::text_color_right(format!("{}.", i + 1), 162, y, FONT_HELP));
            content.push(Element::text_color(&profile.name, 170, y, FONT_HELP));
        }

        // Registered version text (Pascal newregtext + regendtext)
        content.push(Element::fillbox(128, 155, 185, 1, 9));
        content.push(Element::text_color(
            format!(
                "{} {}",
                self.layout.langbase.lstr(35),
                self.layout.langbase.lstr(36)
            ),
            132,
            163,
            FONT_DEFAULT,
        ));
        content.push(Element::fillbox(132, 175, 177, 22, 248));
        content.push(Element::text_color(
            "EVERYONE - THANKS FOR THE SUPPORT!",
            140,
            177,
            FONT_NEW,
        ));

        self.layout.wrap(content)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(0) => Some(RouteTarget::Quit),
            Some(1) => Some(RouteTarget::JumpMenu),
            Some(2) => Some(RouteTarget::ProfilesList),
            Some(3) => Some(RouteTarget::OptionsMenu),
            Some(4) => Some(RouteTarget::HallOfFame),
            Some(5) => Some(RouteTarget::HillRecords),
            Some(6) => Some(RouteTarget::MainMenu),
            Some(7) => Some(RouteTarget::Quit),
            _ => None,
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        apply_logo_tint(palette, 0);
    }
}

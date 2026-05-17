use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{FONT_DEFAULT, FONT_HEADER, BG_ERASE, apply_logo_tint};
use crate::route::RouteTarget;
use crate::store::StoreRef;
use engine::ui::{Component, Element, Event, View};

pub struct MainMenuView {
    menu: Menu,
    layout: MainLayout,
}

impl MainMenuView {
    #[allow(clippy::needless_pass_by_value)]
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
        let selection = store
            .selected_main_menu
            .get()
            .min(items.len().saturating_sub(1));
        let mut menu = Menu::new(
            11,
            97,
            108,
            12,
            items,
            &layout.langbase,
            FONT_DEFAULT,
            FONT_DEFAULT,
        );
        menu.set_selected(selection);
        Self { menu, layout }
    }
}

impl View<RouteTarget> for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = self.layout.background();
        els.extend(self.layout.jumpers());
        els.extend(self.layout.registration());
        els.extend(layout::header_elements(
            self.layout.langbase.lstr(17),
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
            Some(1) => Some(RouteTarget::JumpMenu),
            Some(2) => Some(RouteTarget::ProfilesList),
            Some(3) => Some(RouteTarget::OptionsMenu),
            Some(4) => Some(RouteTarget::HallOfFame),
            Some(5) => Some(RouteTarget::HillRecords),
            Some(6) => Some(RouteTarget::Replays),
            Some(0 | 7) => Some(RouteTarget::Quit),
            _ => None,
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        apply_logo_tint(palette, 0);
    }
}

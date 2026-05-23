use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{apply_logo_tint, BG_ERASE, FONT_DEFAULT, FONT_HEADER};
use crate::route::RouteTarget;
use crate::store::StoreRef;
use engine::ui::{Component, Element, Event, View};

pub struct MainMenuView {
    menu: Menu,
    layout: MainLayout,
}

const MENU_ACTIONS: &[Option<RouteTarget>] = &[
    Some(RouteTarget::JumpMenu),
    Some(RouteTarget::ProfilesList),
    Some(RouteTarget::OptionsMenu),
    Some(RouteTarget::HallOfFame),
    Some(RouteTarget::HillRecords),
    Some(RouteTarget::Replays),
    Some(RouteTarget::Quit),
];

impl MainMenuView {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(layout: MainLayout, store: StoreRef) -> Self {
        let items = vec![
            MenuItem::new(1, 20),
            MenuItem::new(2, 21),
            MenuItem::new(3, 22),
            MenuItem::new(4, 23),
            MenuItem::new(5, 24),
            MenuItem::new(6, 25),
            MenuItem::with_y(0, 26, 12),
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
            Some(0) | Some(7) => Some(RouteTarget::Quit),
            Some(n) => MENU_ACTIONS.get(n - 1).and_then(|&a| a),
            _ => None,
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        apply_logo_tint(palette, 0);
    }
}

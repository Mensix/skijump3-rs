use crate::components::menu::{Menu, MenuItem};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Component, Element, Event, Key, View};

pub struct PracticeView {
    resources: ResourcesRef,
    store: StoreRef,
    menu: Menu,
    start: usize,
    total: usize,
}

impl PracticeView {
    fn page_items(&self) -> usize {
        (self.total.saturating_sub(self.start)).min(20)
    }

    fn has_more(&self) -> bool {
        self.total > 20
    }

    fn item_row(&self, idx: usize) -> usize {
        let page_n = self.page_items();
        if idx < page_n {
            idx
        } else if self.has_more() && idx == page_n {
            idx + 1
        } else {
            idx + 2
        }
    }

    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let total = resources.hills.len();
        let selected = store
            .selected_hill
            .borrow()
            .saturating_sub(1)
            .min(total.saturating_sub(1));
        let start = if total > 20 { selected / 20 * 20 } else { 0 };
        let page_n = (total.saturating_sub(start)).min(20);
        let n = page_n + if total > 20 { 1 } else { 0 } + 1;
        let items = (0..n)
            .map(|_| MenuItem {
                num: 0,
                label: 0,
                y_off: 0,
            })
            .collect();
        let mut menu = Menu::new(
            110,
            11,
            170,
            8,
            items,
            &resources.langbase,
            FONT_DEFAULT,
            FONT_DEFAULT,
        )
        .with_labels(false)
        .with_box(false);
        menu.set_selected(selected.saturating_sub(start).min(page_n.saturating_sub(1)));

        Self {
            menu,
            resources,
            store,
            start,
            total,
        }
    }

    fn rebuild_menu(&self) -> Menu {
        let page_n = self.page_items();
        let n = page_n + if self.has_more() { 1 } else { 0 } + 1;
        let items = (0..n)
            .map(|_| MenuItem {
                num: 0,
                label: 0,
                y_off: 0,
            })
            .collect();
        Menu::new(
            110,
            11,
            170,
            8,
            items,
            &self.resources.langbase,
            FONT_DEFAULT,
            FONT_DEFAULT,
        )
        .with_labels(false)
        .with_box(false)
    }

    fn confirm(&mut self) -> Option<RouteTarget> {
        let sel = self.menu.selected();
        if self.has_more() && sel == self.page_items() {
            self.start = (self.start + 20) % self.total;
            self.menu = self.rebuild_menu();
            None
        } else if sel == self.menu.item_count() - 1 {
            Some(RouteTarget::MainMenu)
        } else {
            *self.store.selected_hill.borrow_mut() = self.start + sel + 1;
            Some(RouteTarget::Jump)
        }
    }
}

impl View<RouteTarget> for PracticeView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![
            Element::fillbox(0, 0, 320, 200, 0),
            Element::fillbox(0, 0, 11, 200, 245),
            Element::fillbox(12, 0, 296, 200, 243),
            Element::fillbox(309, 0, 11, 200, 245),
            Element::FillArea { thing: 63 },
            Element::sprite(61, 30, 8),
            Element::text_color(self.resources.langbase.lstr(151), 30, 31, FONT_DEFAULT),
            Element::text_color(self.resources.langbase.lstr(152), 30, 41, FONT_DEFAULT),
            Element::text_color(self.resources.langbase.lstr(153), 30, 51, FONT_DEFAULT),
        ];

        let page_n = self.page_items();
        for i in 0..page_n {
            let pascal_idx = self.start + i + 1;
            let y = self.item_row(i) as i32 * 8 + 10;
            els.push(Element::text_color_right(
                format!("{}.", i + 1),
                130,
                y,
                FONT_GOLD,
            ));
            if let Some(hill) = self.resources.hills.hill(pascal_idx) {
                els.push(Element::text_color(&hill.name, 140, y, FONT_DEFAULT));
                let name_w = self.resources.font.string_width(&hill.name) as i32;
                els.push(Element::text_color(
                    format!("K{}", hill.kr),
                    145 + name_w,
                    y,
                    FONT_GREET,
                ));
            }
        }

        if self.has_more() {
            let y = self.item_row(page_n) as i32 * 8 + 10;
            els.push(Element::text_color(
                self.resources.langbase.lstr(156),
                140,
                y,
                FONT_GREET,
            ));
        }

        let exit_idx = self.menu.item_count() - 1;
        let y = self.item_row(exit_idx) as i32 * 8 + 10;
        els.push(Element::text_color_right("0.", 130, y, FONT_GOLD));
        els.push(Element::text_color(
            self.resources.langbase.lstr(154),
            140,
            y,
            FONT_DEFAULT,
        ));

        // Selection box at the correct screen row
        let bx = 104;
        let by = 8 + self.item_row(self.menu.selected()) as i32 * 8;
        els.push(Element::box_(bx, by, 171, 9, FONT_DEFAULT));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match &event {
            Event::Keyboard(Key::Escape) | Event::Keyboard(Key::Char('0')) => {
                return Some(RouteTarget::MainMenu);
            }
            Event::Keyboard(Key::Char(ch)) if '1' <= *ch && *ch <= '9' => {
                let n = *ch as usize - '0' as usize;
                let menu_n = self.menu.item_count();
                if n <= menu_n {
                    self.menu.set_selected(n - 1);
                    return self.confirm();
                }
                return None;
            }
            _ => {}
        }
        if let Some(_idx) = self.menu.handle_event(&event) {
            self.confirm()
        } else {
            None
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        apply_menu_tint(palette, 3, 0);
    }
}

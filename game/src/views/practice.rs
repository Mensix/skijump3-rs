use crate::components::menu::{Menu, MenuItem};
use crate::palette_consts::{FONT_DEFAULT, FONT_GOLD, FONT_GREET, apply_menu_tint};
use crate::route::RouteTarget;
use crate::sprites;
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
        if self.has_more() && idx == self.page_items() {
            idx + 1
        } else {
            idx
        }
    }

    fn exit_row(&self) -> usize {
        self.page_items() + if self.has_more() { 3 } else { 2 }
    }

    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let total = resources.hills.len();
        let selected = store
            .practice
            .selected_hill
            .get()
            .saturating_sub(1)
            .min(total.saturating_sub(1));
        let start = if total > 20 { selected / 20 * 20 } else { 0 };
        let page_n = (total.saturating_sub(start)).min(20);
        let n = page_n + usize::from(total > 20);
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
        .with_box(false)
        .with_exit(154, 16);
        menu.set_selected(selected.saturating_sub(start).min(page_n.saturating_sub(1)));

        Self {
            resources,
            store,
            menu,
            start,
            total,
        }
    }

    fn rebuild_menu(&self) -> Menu {
        let page_n = self.page_items();
        let n = page_n + usize::from(self.has_more());
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
        .with_exit(154, 16)
    }

    fn confirm(&mut self) -> Option<RouteTarget> {
        let sel = self.menu.selected();
        if self.menu.has_exit() && sel == self.menu.item_count() {
            return Some(RouteTarget::MainMenu);
        }
        if self.has_more() && sel == self.page_items() {
            self.start = (self.start + 20) % self.total;
            self.menu = self.rebuild_menu();
            None
        } else {
            let hill_idx = self.start + sel + 1;
            self.store.practice.selected_hill.set(hill_idx);
            self.store.selected_hill.set(hill_idx);
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
            Element::sprite(sprites::LOGO_SPRITE, 30, 8),
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

        let y = (self.exit_row() - 1) as i32 * 8 + 10;
        els.push(Element::text_color_right("0.", 130, y, FONT_DEFAULT));
        els.push(Element::text_color(
            self.resources.langbase.lstr(154),
            140,
            y,
            FONT_DEFAULT,
        ));

        // Selection box at the correct screen row.
        // Pascal MakeMenu positions EXIT box at index items+2 (1-based)
        // which is row items+1 (0-based). Our exit_row = items+2 (0-based),
        // so subtract 1 for the box to match Pascal.
        let bx = 104;
        let sel = self.menu.selected();
        let sel_row = if self.menu.has_exit() && sel == self.menu.item_count() {
            self.exit_row() - 1
        } else {
            self.item_row(sel)
        };
        let by = 8 + sel_row as i32 * 8;
        els.push(Element::box_(bx, by, 171, 9, FONT_DEFAULT));

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match &event {
            Event::Keyboard(Key::Escape) => {
                return Some(RouteTarget::Back);
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
            Event::Keyboard(_) => {}
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

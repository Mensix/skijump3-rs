use crate::gfx::sprites;
use crate::gfx::theme::{BG_LEFT, BLACK, FILL_DIM, FONT_DEFAULT, FONT_GOLD, FONT_GREET};
use crate::route::RouteTarget;
use crate::store::ResourcesRef;
use crate::text::format;
use engine::oxide::input::Key;
use engine::oxide::widget::EventCx;
use engine::oxide::widgets::menu::{MenuItem as OxideMenuItem, PixelMenu};
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent, Widget};

pub struct KothHillPickerView {
    resources: ResourcesRef,
    menu: PixelMenu,
    start: usize,
    total: usize,
}

impl KothHillPickerView {
    pub fn new(resources: ResourcesRef) -> Self {
        let kothmaki = resources.save_manager.config.borrow().kothmaki;
        let total = resources.hills.len();
        let start = if kothmaki > 0 {
            ((kothmaki as usize - 1) / 20) * 20
        } else {
            0
        };
        let page_n = (total.saturating_sub(start)).min(20);
        let n = page_n + usize::from(total > 20);
        let items = (0..n).map(|_| OxideMenuItem::new(0, "")).collect();
        let mut menu = PixelMenu::new(110, 11, 170, 8, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false)
            .trailing("", 16);
        if kothmaki == 0 {
            menu.set_selected(menu.item_count());
        } else if kothmaki as usize - 1 < start + page_n {
            menu.set_selected(kothmaki as usize - 1 - start);
        }

        Self {
            resources,
            menu,
            start,
            total,
        }
    }

    fn page_items(&self) -> usize {
        (self.total.saturating_sub(self.start)).min(20)
    }

    fn has_more(&self) -> bool {
        self.total > self.start + 20
    }

    fn item_row(&self, idx: usize) -> usize {
        if self.has_more() && idx == self.page_items() {
            idx + 1
        } else {
            idx
        }
    }

    fn exit_row(&self) -> usize {
        self.page_items() + if self.has_more() { 4 } else { 3 }
    }

    fn rebuild_menu(&self) -> PixelMenu {
        let page_n = self.page_items();
        let n = page_n + usize::from(self.has_more());
        let items = (0..n).map(|_| OxideMenuItem::new(0, "")).collect();
        PixelMenu::new(110, 11, 170, 8, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false)
            .trailing("", 16)
    }

    fn select_hill(&mut self) {
        let sel = self.menu.selected();
        if self.has_more() && sel == self.page_items() {
            self.start = if self.start + 20 >= self.total { 0 } else { self.start + 20 };
            self.menu = self.rebuild_menu();
        } else if sel == self.menu.item_count() {
            self.resources
                .save_manager
                .update_config(|cfg| cfg.kothmaki = 0);
        } else {
            let hill_idx = self.start + sel;
            self.resources
                .save_manager
                .update_config(|cfg| cfg.kothmaki = hill_idx as i32 + 1);
        }
    }

    fn confirm_event(&mut self, ecx: &mut EventCx, event: UiEvent) -> bool {
        match event {
            UiEvent::Text(ch) if ch.is_ascii_digit() && ch != '0' => {
                let n = ch as usize - '0' as usize;
                let menu_n = self.menu.item_count();
                if n <= menu_n {
                    self.menu.set_selected(n - 1);
                    self.select_hill();
                    return true;
                }
                return false;
            }
            _ => {}
        }
        if let Some(_idx) = self.menu.event(ecx, event) {
            self.select_hill();
            true
        } else {
            false
        }
    }
}

impl Screen<RouteTarget> for KothHillPickerView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::Quit | UiEvent::Tick => return,
            UiEvent::KeyDown(Key::Escape) => {
                cx.back();
                return;
            }
            _ => {}
        }
        let mut ecx = EventCx::default();
        if self.confirm_event(&mut ecx, event) {
            cx.back();
        }
        if ecx.is_consumed() {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 11, 200), FILL_DIM);
        cx.pattern_fill((12, 0, 296, 200), BG_LEFT);
        cx.pattern_fill((309, 0, 11, 200), FILL_DIM);
        cx.sprite(sprites::Sprite::Logo as u16, (30, 8));
        cx.text((30, 31), FONT_DEFAULT, self.resources.langbase.lstr(151));
        cx.text((30, 41), FONT_DEFAULT, self.resources.langbase.lstr(152));
        cx.text((30, 51), FONT_DEFAULT, self.resources.langbase.lstr(153));

        let page_n = self.page_items();
        for i in 0..page_n {
            let idx = self.start + i;
            let y = self.item_row(i) as i32 * 8 + 10;
            cx.right_text((130, y), FONT_GOLD, format::ordinal_dot(i + 1));
            if let Some(hill) = self.resources.hills.hill(idx) {
                cx.text((140, y), FONT_DEFAULT, &hill.name);
                let name_w = self.resources.font.string_width(&hill.name) as i32;
                cx.text((145 + name_w, y), FONT_GREET, format!("K{}", hill.kr));
            }
        }

        if self.has_more() {
            let y = self.item_row(page_n) as i32 * 8 + 10;
            cx.text((140, y), FONT_GREET, self.resources.langbase.lstr(156));
        }

        // "0. RANDOM WC HILL" at bottom
        let y = (self.exit_row() - 1) as i32 * 8 + 10;
        cx.right_text((130, y), FONT_DEFAULT, "0.");
        cx.text((140, y), FONT_DEFAULT, self.resources.langbase.lstr(155));

        // Selection box
        let bx = 104;
        let sel = self.menu.selected();
        let sel_row = if sel == self.menu.item_count() {
            self.exit_row() - 1
        } else {
            self.item_row(sel)
        };
        let by = 8 + sel_row as i32 * 8;
        cx.stroke((bx, by, 171, 9), FONT_DEFAULT);
    }
}

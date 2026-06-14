use crate::competition::factory;
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_LEFT, BLACK, DITHER_FILL_COLORS, FILL_DIM, FONT_DEFAULT, FONT_GOLD, FONT_GREET,
};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format;
use engine::oxide::input::Key;
use engine::oxide::widget::EventCx;
use engine::oxide::widgets::menu::{MenuItem as OxideMenuItem, PixelMenu};
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent, Widget};

pub struct TrainingSetupView {
    resources: ResourcesRef,
    store: StoreRef,
    menu: PixelMenu,
    start: usize,
    total: usize,
}

impl TrainingSetupView {
    fn page_items(&self) -> usize {
        (self.total.saturating_sub(self.start)).min(20)
    }

    const fn has_more(&self) -> bool {
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
        let selected = store.practice_hill().min(total.saturating_sub(1));
        let start = if total > 20 { selected / 20 * 20 } else { 0 };
        let page_n = (total.saturating_sub(start)).min(20);
        let n = page_n + usize::from(total > 20);
        let items = (0..n).map(|_| OxideMenuItem::new(0, "")).collect();
        let mut menu = PixelMenu::new(110, 11, 170, 8, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false)
            .with_exit("", 16);
        menu.set_selected(selected.saturating_sub(start).min(page_n.saturating_sub(1)));

        Self {
            resources,
            store,
            menu,
            start,
            total,
        }
    }

    fn rebuild_menu(&self) -> PixelMenu {
        let page_n = self.page_items();
        let n = page_n + usize::from(self.has_more());
        let items = (0..n).map(|_| OxideMenuItem::new(0, "")).collect();
        PixelMenu::new(110, 11, 170, 8, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false)
            .with_exit("", 16)
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
            let hill_idx = self.start + sel;
            self.store.set_practice_hill(hill_idx);
            self.store.set_selected_hill(hill_idx);
            self.store.start_active(factory::training());
            Some(RouteTarget::Jump)
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.fill((0, 0, 11, 200), FILL_DIM);
        cx.fill((12, 0, 296, 200), BG_LEFT);
        cx.fill((309, 0, 11, 200), FILL_DIM);
        cx.dither_fill(63, DITHER_FILL_COLORS);
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

        let y = (self.exit_row() - 1) as i32 * 8 + 10;
        cx.right_text((130, y), FONT_DEFAULT, "0.");
        cx.text((140, y), FONT_DEFAULT, self.resources.langbase.lstr(154));

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
        cx.stroke((bx, by, 171, 9), FONT_DEFAULT);
    }

    fn handle_input(&mut self, ecx: &mut EventCx, event: UiEvent) -> Option<RouteTarget> {
        match event {
            UiEvent::KeyDown(Key::Escape) => {
                return Some(RouteTarget::Back);
            }
            UiEvent::Text(ch) if ch.is_ascii_digit() && ch != '0' => {
                let n = ch as usize - '0' as usize;
                let menu_n = self.menu.item_count();
                if n <= menu_n {
                    self.menu.set_selected(n - 1);
                    return self.confirm();
                }
                return None;
            }
            _ => {}
        }
        if let Some(_idx) = self.menu.event(ecx, event) {
            self.confirm()
        } else {
            None
        }
    }
}

impl Screen<RouteTarget> for TrainingSetupView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::Quit | UiEvent::Tick => return,
            _ => {}
        }
        let mut ecx = EventCx::default();
        if let Some(route) = self.handle_input(&mut ecx, event) {
            cx.navigate(route);
        }
        if ecx.is_consumed() {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

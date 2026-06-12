use crate::competition::factory;
use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{BG_LEFT, BLACK, FILL_DIM, FONT_DEFAULT, FONT_GOLD, FONT_GREET};
use crate::gfx::sprites;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Component, Event, Key};

pub struct TrainingSetupView {
    resources: ResourcesRef,
    store: StoreRef,
    menu: Menu,
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
        let items = (0..n).map(|_| MenuItem::new(0, 0)).collect();
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
        let items = (0..n).map(|_| MenuItem::new(0, 0)).collect();
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
        cx.dither_fill(63);
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

    fn handle_input(&mut self, event: Event) -> Option<RouteTarget> {
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
        self.menu
            .handle_event(&event)
            .and_then(|_idx| self.confirm())
    }
}

impl Screen<RouteTarget> for TrainingSetupView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_from_ui(event) else {
            return;
        };
        if let Some(route) = self.handle_input(event) {
            cx.navigate(route);
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

fn input_from_ui(event: UiEvent) -> Option<Event> {
    match event {
        UiEvent::KeyDown(key) => Some(Event::Keyboard(key)),
        UiEvent::Text(c) => Some(Event::Keyboard(Key::Char(c))),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

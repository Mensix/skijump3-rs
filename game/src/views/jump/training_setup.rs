use crate::competition::factory;
use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_TEAL};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::text::format;
use engine::oxide::input::Key;
use engine::oxide::widget::EventCx;
use engine::oxide::widgets::menu::{MenuItem as OxideMenuItem, PixelMenu};
use engine::oxide::{PaintCx, ScreenEventCx, UiEvent, Widget};

pub struct TrainingSetupView {
    resources: ResourcesRef,
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
        idx
    }

    fn exit_row(&self) -> usize {
        self.page_items() + 3
    }

    pub fn new(resources: ResourcesRef, state: &GameState) -> Self {
        let total = resources.hills.len();
        let selected = state.practice_hill.min(total.saturating_sub(1));
        let start = if total > 20 { selected / 20 * 20 } else { 0 };
        let page_n = (total.saturating_sub(start)).min(20);
        let n = page_n + usize::from(total > 20);
        let items = (0..n).map(|_| OxideMenuItem::new(0, "")).collect();
        let mut menu = PixelMenu::new(110, 11, 170, 8, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false)
            .trailing("", 16)
            .with_return_index(true);
        menu.set_selected(selected.saturating_sub(start).min(page_n.saturating_sub(1)));

        Self {
            resources,
            menu,
            start,
            total,
        }
    }

    fn rebuild_menu(&self) -> PixelMenu {
        let page_n = self.page_items();
        let n = page_n + usize::from(self.has_more());
        let items = (0..n).map(|_| OxideMenuItem::new(0, "")).collect();
        PixelMenu::new(110, 11, 170, 8, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false)
            .trailing("", 16)
            .with_return_index(true)
    }

    fn confirm(&mut self, state: &mut GameState) -> Option<RouteTarget> {
        let sel = self.menu.selected();
        if self.menu.has_trailing() && sel == self.menu.item_count() {
            return Some(RouteTarget::MainMenu);
        }
        if self.has_more() && sel == self.page_items() {
            self.start = if self.start + 20 >= self.total {
                0
            } else {
                self.start + 20
            };
            self.menu = self.rebuild_menu();
            None
        } else {
            let hill_idx = self.start + sel;
            state.practice_hill = hill_idx;
            state.start_active(factory::training());
            Some(RouteTarget::Jump)
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        let lang = &self.resources.langbase;
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 11, 200), FILL_GRAY);
        cx.pattern_fill((12, 0, 296, 200), BG_PURPLE);
        cx.pattern_fill((309, 0, 11, 200), FILL_GRAY);
        cx.sprite(sprites::Sprite::Logo as u16, (30, 8));
        cx.text((30, 31), FONT_BODY, lang.tr(151));
        cx.text((30, 41), FONT_BODY, lang.tr(152));
        cx.text((30, 51), FONT_BODY, lang.tr(153));

        let page_n = self.page_items();
        for i in 0..page_n {
            let idx = self.start + i;
            let y = self.item_row(i) as i32 * 8 + 10;
            cx.right_text((130, y), FONT_GOLD, format::ordinal_dot(i + 1));
            if let Some(hill) = self.resources.hills.hill(idx) {
                cx.text((140, y), FONT_BODY, &hill.name);
                let name_w = self.resources.font.string_width(&hill.name) as i32;
                cx.text((145 + name_w, y), FONT_TEAL, format!("K{}", hill.kr));
            }
        }

        if self.has_more() {
            let y = self.item_row(page_n) as i32 * 8 + 10;
            cx.text((140, y), FONT_TEAL, lang.tr(156));
        }

        let y = (self.exit_row() - 1) as i32 * 8 + 10;
        cx.right_text((130, y), FONT_BODY, "0.");
        cx.text((140, y), FONT_BODY, lang.tr(154));

        // Selection box at the correct screen row.
        // Pascal MakeMenu positions EXIT box at index items+2 (1-based)
        // which is row items+1 (0-based). Our exit_row = items+2 (0-based),
        // so subtract 1 for the box to match Pascal.
        let bx = 104;
        let sel = self.menu.selected();
        let sel_row = if self.menu.has_trailing() && sel == self.menu.item_count() {
            self.exit_row() - 1
        } else {
            self.item_row(sel)
        };
        let by = 8 + sel_row as i32 * 8;
        cx.stroke((bx, by, 171, 9), FONT_BODY);
    }

    fn handle_input(
        &mut self,
        ecx: &mut EventCx,
        state: &mut GameState,
        event: UiEvent,
    ) -> Option<RouteTarget> {
        if matches!(event, UiEvent::KeyDown(Key::Escape)) {
            return Some(RouteTarget::Back);
        }
        if let Some(_idx) = self.menu.event(ecx, event) {
            self.confirm(state)
        } else {
            None
        }
    }
}

impl GameScreen for TrainingSetupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::Quit | UiEvent::Tick => return,
            _ => {}
        }
        let mut ecx = EventCx::default();
        if let Some(route) = self.handle_input(&mut ecx, cx.state, event) {
            nav.navigate(route);
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(paint);
    }
}

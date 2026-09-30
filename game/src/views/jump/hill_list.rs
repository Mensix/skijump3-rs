use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_TEAL};
use crate::store::ResourcesRef;
use crate::text::format;
use crate::ui::UiCanvas;
use crate::ui::{EventCx, MenuAction, MenuItem, PixelMenu, UiEvent};

pub(crate) enum HillPick {
    Exit,
    More,
    Hill(usize),
}

pub(crate) struct HillListPicker {
    resources: ResourcesRef,
    menu: PixelMenu,
    start: usize,
    total: usize,
}

impl HillListPicker {
    pub(crate) fn new(resources: ResourcesRef, selected: usize) -> Self {
        let total = resources.hills.len();
        let selected = selected.min(total.saturating_sub(1));
        let start = if total > 20 { selected / 20 * 20 } else { 0 };
        let page_n = total.saturating_sub(start).min(20);
        let n = page_n + usize::from(total > 20);
        let mut items: Vec<MenuItem> = (0..n).map(|i| MenuItem::new(i as u8, "")).collect();
        items.push(MenuItem::new(n as u8, "").with_gap_before(16));
        let mut menu = PixelMenu::new(110, 11, 170, 8, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        menu.set_selected(selected.saturating_sub(start).min(page_n.saturating_sub(1)));

        Self {
            resources,
            menu,
            start,
            total,
        }
    }

    pub(crate) fn page_items(&self) -> usize {
        (self.total.saturating_sub(self.start)).min(20)
    }

    pub(crate) fn has_more(&self) -> bool {
        Self::has_more_at(self.total, self.start)
    }

    const fn has_more_at(total: usize, start: usize) -> bool {
        total > start.saturating_add(20)
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

    fn set_page(&mut self) {
        let page_n = self.page_items();
        let n = page_n + usize::from(self.has_more());
        let mut items: Vec<MenuItem> = (0..n).map(|i| MenuItem::new(i as u8, "")).collect();
        items.push(MenuItem::new(n as u8, "").with_gap_before(16));
        self.menu.set_items(items);
    }

    fn turn_page(&mut self) {
        self.start = if self.start + 20 >= self.total {
            0
        } else {
            self.start + 20
        };
        self.set_page();
        self.menu.set_selected(0);
    }

    pub(crate) fn confirm(&mut self, ecx: &mut EventCx, event: UiEvent) -> Option<HillPick> {
        match self.menu.event_action(ecx, event)? {
            MenuAction::Item(idx) if idx == self.menu.item_count() - 1 => Some(HillPick::Exit),
            MenuAction::Item(idx) if self.has_more() && idx == self.page_items() => {
                self.turn_page();
                Some(HillPick::More)
            }
            MenuAction::Item(idx) => Some(HillPick::Hill(self.start + idx)),
        }
    }

    pub(crate) fn paint(&self, cx: &mut dyn UiCanvas, exit_label_id: usize) {
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
            cx.right_text((130, y), FONT_GOLD, &format::ordinal_dot(i + 1));
            if let Some(hill) = self.resources.hills.hill(idx) {
                cx.text((140, y), FONT_BODY, &hill.name);
                let name_w = self.resources.font.string_width(&hill.name) as i32;
                cx.text((145 + name_w, y), FONT_TEAL, &format!("K{}", hill.kr));
            }
        }

        if self.has_more() {
            let y = self.item_row(page_n) as i32 * 8 + 10;
            cx.text((140, y), FONT_TEAL, lang.tr(156));
        }

        let y = (self.exit_row() - 1) as i32 * 8 + 10;
        cx.right_text((130, y), FONT_BODY, "0.");
        cx.text((140, y), FONT_BODY, lang.tr(exit_label_id));

        let bx = 104;
        let sel = self.menu.selected();
        let sel_row = if sel == self.menu.item_count() - 1 {
            self.exit_row() - 1
        } else {
            self.item_row(sel)
        };
        let by = 8 + sel_row as i32 * 8;
        cx.stroke((bx, by, 171, 9), FONT_BODY);
    }
}

#[cfg(test)]
mod tests {
    use super::HillListPicker;

    #[test]
    fn last_page_does_not_have_next_option() {
        assert!(!HillListPicker::has_more_at(21, 20));
        assert!(HillListPicker::has_more_at(41, 20));
    }
}

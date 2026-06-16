use crate::competition::factory;
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BLACK, FILL_GRAY, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::route::RouteTarget;
use crate::store::{GameStateRef, ResourcesRef};
use crate::text::format;
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};

const MAX_HILLS: usize = 40;

pub struct CustomCupSetupView {
    resources: ResourcesRef,
    store: GameStateRef,
    selected: Vec<usize>,
    preview: usize,
    all_hill_count: usize,
}

impl CustomCupSetupView {
    pub fn new(resources: ResourcesRef, store: GameStateRef) -> Self {
        let count = resources.hills.len();
        Self {
            resources,
            store,
            selected: vec![0],
            preview: 0,
            all_hill_count: count,
        }
    }

    fn paint_hill(&self, cx: &mut PaintCx<'_>, slot: usize, hill_idx: usize, is_preview: bool) {
        let (x, y) = if slot < 20 {
            (17, slot as i32 * 7 + 39)
        } else {
            (162, (slot as i32 - 20) * 7 + 39)
        };
        cx.fill((x, y - 1, 143, 9), FILL_PURPLE);
        if let Some(h) = self.resources.hills.hill(hill_idx) {
            if is_preview {
                cx.text((x + 15, y), FONT_GRAY, &h.name);
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                cx.text((x + 18 + name_w, y), FONT_GRAY, kr_str);
            } else {
                let num_str = format::ordinal_dot(slot + 1);
                cx.right_text((x + 14, y), FONT_GOLD, num_str);
                cx.text((x + 15, y), FONT_BODY, &h.name);
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                cx.text((x + 18 + name_w, y), FONT_TEAL, kr_str);
            }
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        let lang = &self.resources.langbase;
        let help_line = format!("{}, {}, {}", lang.lstr(285), lang.lstr(286), lang.lstr(287));
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 11, 200), FILL_GRAY);
        cx.pattern_fill((12, 0, 296, 200), BG_PURPLE);
        cx.pattern_fill((309, 0, 11, 200), FILL_GRAY);
        cx.sprite(sprites::Sprite::Logo as u16, (30, 8));
        cx.text((68, 8), FONT_BODY, lang.lstr(118));
        cx.text((78, 16), FONT_GRAY, lang.lstr(119));
        cx.text((78, 23), FONT_GRAY, help_line);
        cx.text((78, 30), FONT_GRAY, lang.lstr(288));

        for (i, &hill_idx) in self.selected.iter().enumerate() {
            self.paint_hill(cx, i, hill_idx, false);
        }

        let preview_slot = self.selected.len();
        if preview_slot < MAX_HILLS && self.preview < self.all_hill_count {
            self.paint_hill(cx, preview_slot, self.preview, true);
        }
    }

    fn handle_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        match event {
            UiEvent::KeyDown(Key::Escape) => Some(RouteTarget::Back),
            UiEvent::KeyDown(Key::Enter) => {
                if self.selected.is_empty() {
                    return None;
                }
                let state = self.store.borrow();
                let comp = factory::custom_cup(
                    &state.profiles,
                    self.resources.player_names(state.config.namenumber as usize),
                    self.selected.clone(),
                    0,
                );
                drop(state);
                self.store.borrow_mut().start_active(comp);
                Some(RouteTarget::CompetitionJump)
            }
            UiEvent::KeyDown(Key::Left) => {
                if self.preview > 0 {
                    self.preview -= 1;
                }
                None
            }
            UiEvent::KeyDown(Key::Right) => {
                if self.preview + 1 < self.all_hill_count {
                    self.preview += 1;
                }
                None
            }
            UiEvent::KeyDown(Key::Home | Key::Delete) => {
                self.preview = 0;
                None
            }
            UiEvent::KeyDown(Key::End) => {
                self.preview = self.all_hill_count - 1;
                None
            }
            UiEvent::KeyDown(Key::PageUp) => {
                self.preview = self.preview.saturating_sub(5);
                None
            }
            UiEvent::KeyDown(Key::PageDown) => {
                self.preview = (self.preview + 5).min(self.all_hill_count - 1);
                None
            }
            UiEvent::KeyDown(Key::Down) | UiEvent::Text(' ') => {
                if self.selected.len() < MAX_HILLS {
                    self.selected.push(self.preview);
                }
                None
            }
            UiEvent::KeyDown(Key::Up | Key::Backspace) => {
                self.selected.pop();
                None
            }
            _ => None,
        }
    }
}

impl Screen<RouteTarget> for CustomCupSetupView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let Some(route) = self.handle_input(event) {
            if route == RouteTarget::Back {
                cx.back();
            } else {
                cx.navigate(route);
            }
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

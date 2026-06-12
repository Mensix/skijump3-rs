use crate::competition::factory;
use crate::gfx::palette::{FILL_BORDER, FONT_DEFAULT, FONT_GREET, FONT_HEADER, FONT_HELP};
use crate::gfx::sprites;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Event, Key};

const MAX_HILLS: usize = 40;

pub struct CustomCupSetupView {
    resources: ResourcesRef,
    store: StoreRef,
    selected: Vec<usize>,
    preview: usize,
    all_hill_count: usize,
}

impl CustomCupSetupView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
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
        cx.fill((x, y - 1, 143, 9), FILL_BORDER);
        if let Some(h) = self.resources.hills.hill(hill_idx) {
            if is_preview {
                cx.text((x + 15, y), FONT_HELP, &h.name);
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                cx.text((x + 18 + name_w, y), FONT_HELP, kr_str);
            } else {
                let num_str = format::ordinal_dot(slot + 1);
                cx.right_text((x + 14, y), FONT_HEADER, num_str);
                cx.text((x + 15, y), FONT_DEFAULT, &h.name);
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                cx.text((x + 18 + name_w, y), FONT_GREET, kr_str);
            }
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        let lang = &self.resources.langbase;
        let help_line = format!("{}, {}, {}", lang.lstr(285), lang.lstr(286), lang.lstr(287));
        cx.fill((0, 0, 320, 200), crate::gfx::palette::BLACK);
        cx.fill((0, 0, 11, 200), crate::gfx::palette::FILL_DIM);
        cx.fill((12, 0, 296, 200), crate::gfx::palette::BG_LEFT);
        cx.fill((309, 0, 11, 200), crate::gfx::palette::FILL_DIM);
        cx.dither_fill(63);
        cx.sprite(sprites::Sprite::Logo as u16, (30, 8));
        cx.text((68, 8), FONT_DEFAULT, lang.lstr(118));
        cx.text((78, 16), FONT_HELP, lang.lstr(119));
        cx.text((78, 23), FONT_HELP, help_line);
        cx.text((78, 30), FONT_HELP, lang.lstr(288));

        for (i, &hill_idx) in self.selected.iter().enumerate() {
            self.paint_hill(cx, i, hill_idx, false);
        }

        let preview_slot = self.selected.len();
        if preview_slot < MAX_HILLS && self.preview < self.all_hill_count {
            self.paint_hill(cx, preview_slot, self.preview, true);
        }
    }

    fn handle_input(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::Back),
            Event::Keyboard(Key::Enter) => {
                if self.selected.is_empty() {
                    return None;
                }
                let profiles = self.store.profiles();
                let comp = factory::custom_cup(
                    &profiles,
                    self.resources.player_names(),
                    self.selected.clone(),
                    0,
                );
                drop(profiles);
                self.store.start_active(comp);
                Some(RouteTarget::CompetitionJump)
            }
            Event::Keyboard(Key::Left) => {
                if self.preview > 0 {
                    self.preview -= 1;
                }
                None
            }
            Event::Keyboard(Key::Right) => {
                if self.preview + 1 < self.all_hill_count {
                    self.preview += 1;
                }
                None
            }
            Event::Keyboard(Key::Home | Key::Delete) => {
                self.preview = 0;
                None
            }
            Event::Keyboard(Key::End) => {
                self.preview = self.all_hill_count - 1;
                None
            }
            Event::Keyboard(Key::PageUp) => {
                self.preview = self.preview.saturating_sub(5);
                None
            }
            Event::Keyboard(Key::PageDown) => {
                self.preview = (self.preview + 5).min(self.all_hill_count - 1);
                None
            }
            Event::Keyboard(Key::Down | Key::Char(' ')) => {
                if self.selected.len() < MAX_HILLS {
                    self.selected.push(self.preview);
                }
                None
            }
            Event::Keyboard(Key::Up | Key::Backspace) => {
                self.selected.pop();
                None
            }
            _ => None,
        }
    }
}

impl Screen<RouteTarget> for CustomCupSetupView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_from_ui(event) else {
            return;
        };
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

fn input_from_ui(event: UiEvent) -> Option<Event> {
    match event {
        UiEvent::KeyDown(key) => Some(Event::Keyboard(key)),
        UiEvent::Text(c) => Some(Event::Keyboard(Key::Char(c))),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

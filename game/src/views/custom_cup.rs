use crate::competition::factory;
use crate::components::screen::new_screen;
use crate::gfx::palette::{FILL_BORDER, FONT_DEFAULT, FONT_GREET, FONT_HEADER, FONT_HELP};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format;
use engine::oxide::legacy::{event_from_ui, paint_elements};
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Element, Event, Key};

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

    fn hill_elements(&self, slot: usize, hill_idx: usize, is_preview: bool) -> Vec<Element> {
        let (x, y) = if slot < 20 {
            (17, slot as i32 * 7 + 39)
        } else {
            (162, (slot as i32 - 20) * 7 + 39)
        };
        let mut els = vec![Element::fillbox(x, y - 1, 143, 9, FILL_BORDER)];
        if let Some(h) = self.resources.hills.hill(hill_idx) {
            if is_preview {
                els.push(Element::text(&h.name, x + 15, y, FONT_HELP, false));
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                els.push(Element::text(&kr_str, x + 18 + name_w, y, FONT_HELP, false));
            } else {
                let num_str = format::ordinal_dot(slot + 1);
                els.push(Element::right_text(&num_str, x + 14, y, FONT_HEADER));
                els.push(Element::text(&h.name, x + 15, y, FONT_DEFAULT, false));
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                els.push(Element::text(
                    &kr_str,
                    x + 18 + name_w,
                    y,
                    FONT_GREET,
                    false,
                ));
            }
        }
        els
    }

    fn legacy_elements(&self) -> Vec<Element> {
        let lang = &self.resources.langbase;
        let help_line = format!("{}, {}, {}", lang.lstr(285), lang.lstr(286), lang.lstr(287));
        let mut els = new_screen(2);
        els.push(Element::text(
            lang.lstr(118).to_string(),
            68,
            8,
            FONT_DEFAULT,
            false,
        ));
        els.push(Element::text(
            lang.lstr(119).to_string(),
            78,
            16,
            FONT_HELP,
            false,
        ));
        els.push(Element::text(help_line, 78, 23, FONT_HELP, false));
        els.push(Element::text(
            lang.lstr(288).to_string(),
            78,
            30,
            FONT_HELP,
            false,
        ));

        for (i, &hill_idx) in self.selected.iter().enumerate() {
            els.extend(self.hill_elements(i, hill_idx, false));
        }

        let preview_slot = self.selected.len();
        if preview_slot < MAX_HILLS && self.preview < self.all_hill_count {
            els.extend(self.hill_elements(preview_slot, self.preview, true));
        }

        els
    }

    fn legacy_handle_event(&mut self, event: Event) -> Option<RouteTarget> {
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
        let Some(event) = event_from_ui(event) else {
            return;
        };
        if let Some(route) = self.legacy_handle_event(event) {
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
        paint_elements(cx, &self.legacy_elements());
    }
}

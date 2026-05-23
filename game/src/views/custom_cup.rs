use crate::competition::builder::build_custom_competition;
use crate::gfx::palette::{apply_menu_tint, BG_LEFT, FONT_DEFAULT, FONT_GREET, FONT_HEADER, FONT_HELP};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::ui::{Element, Event, Key, View};

const MAX_HILLS: usize = 40;

pub struct CustomCupSetupView {
    store: StoreRef,
    resources: ResourcesRef,
    selected: Vec<usize>,
    preview: usize,
    all_hill_count: usize,
}

impl CustomCupSetupView {
    pub fn new(store: StoreRef, resources: ResourcesRef) -> Self {
        let count = resources.hills.len();
        Self {
            store,
            resources,
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
        let mut els = vec![Element::fillbox(x, y - 1, 143, 9, 248)];
        if let Some(h) = self.resources.hills.hill(hill_idx) {
            if is_preview {
                els.push(Element::text(&h.name, x + 15, y, FONT_HELP, false));
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                els.push(Element::text(&kr_str, x + 18 + name_w, y, FONT_HELP, false));
            } else {
                let num_str = format!("{}.", slot + 1);
                els.push(Element::right_text(&num_str, x + 14, y, FONT_HEADER));
                els.push(Element::text(&h.name, x + 15, y, FONT_DEFAULT, false));
                let name_w = self.resources.font.string_width(&h.name) as i32;
                let kr_str = format!("K{}", h.kr);
                els.push(Element::text(&kr_str, x + 18 + name_w, y, FONT_GREET, false));
            }
        }
        els
    }
}

impl View<RouteTarget> for CustomCupSetupView {
    fn elements(&self) -> Vec<Element> {
        let lang = &self.resources.langbase;
        let help_line = format!(
            "{}, {}, {}",
            lang.lstr(285),
            lang.lstr(286),
            lang.lstr(287)
        );
        let mut els = vec![
            Element::fillbox(0, 0, 320, 200, 0),
            Element::fillbox(0, 0, 11, 200, 245),
            Element::fillbox(12, 0, 296, 200, BG_LEFT),
            Element::fillbox(309, 0, 11, 200, 245),
            Element::fill_area(63),
            Element::Sprite(61, 30, 8),
            Element::text(lang.lstr(118).to_string(), 68, 8, FONT_DEFAULT, false),
            Element::text(lang.lstr(119).to_string(), 78, 16, FONT_HELP, false),
            Element::text(help_line, 78, 23, FONT_HELP, false),
            Element::text(lang.lstr(288).to_string(), 78, 30, FONT_HELP, false),
        ];

        for (i, &hill_idx) in self.selected.iter().enumerate() {
            els.extend(self.hill_elements(i, hill_idx, false));
        }

        let preview_slot = self.selected.len();
        if preview_slot < MAX_HILLS && self.preview < self.all_hill_count {
            els.extend(self.hill_elements(preview_slot, self.preview, true));
        }

        els
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        apply_menu_tint(palette, 3, 0);
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::Back),
            Event::Keyboard(Key::Enter) => {
                if self.selected.is_empty() {
                    return None;
                }
                let profiles = self.store.profiles.borrow();
                let comp = build_custom_competition(
                    &profiles,
                    &self.resources.player_names,
                    self.selected.clone(),
                    0,
                );
                drop(profiles);
                self.store.competition.start(comp);
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
            Event::Keyboard(Key::Home) | Event::Keyboard(Key::Delete) => {
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
            Event::Keyboard(Key::Down) | Event::Keyboard(Key::Char(' ')) => {
                if self.selected.len() < MAX_HILLS {
                    self.selected.push(self.preview);
                }
                None
            }
            Event::Keyboard(Key::Up) | Event::Keyboard(Key::Backspace) => {
                self.selected.pop();
                None
            }
            _ => None,
        }
    }
}

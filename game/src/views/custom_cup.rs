use crate::competition::factory;
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BLACK, FILL_GRAY, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{GameStateRef, ResourcesRef};
use crate::text::format;
use engine::oxide::input::Key;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use serde::{Deserialize, Serialize};

const MAX_HILLS: usize = 40;
const CUSTOM_CUP_DIR: &str = "custom_cups";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomCupMode {
    Browse,
    SaveInput,
    ConfirmOverwrite,
    Load,
    ConfirmDelete,
    Message,
}

#[derive(Debug, Deserialize, Serialize)]
struct CustomCupFile {
    format_version: u32,
    hills: Vec<usize>,
}

pub struct CustomCupSetupView {
    resources: ResourcesRef,
    store: GameStateRef,
    save_manager: SaveRef,
    selected: Vec<usize>,
    preview: usize,
    all_hill_count: usize,
    mode: CustomCupMode,
    filename_input: String,
    load_entries: Vec<String>,
    load_index: usize,
    message: String,
}

impl CustomCupSetupView {
    pub fn new(resources: ResourcesRef, store: GameStateRef, save_manager: SaveRef) -> Self {
        let count = resources.hills.len();
        Self {
            resources,
            store,
            save_manager,
            selected: vec![0],
            preview: 0,
            all_hill_count: count,
            mode: CustomCupMode::Browse,
            filename_input: "CUSTOM".to_string(),
            load_entries: Vec::new(),
            load_index: 0,
            message: String::new(),
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

        cx.text(
            (250, 30),
            FONT_GRAY,
            self.store.borrow().config.setfile.as_str(),
        );

        match self.mode {
            CustomCupMode::Browse => {}
            CustomCupMode::SaveInput => self.paint_save_input(cx),
            CustomCupMode::ConfirmOverwrite => {
                self.paint_confirm(cx, "FILE ALREADY EXISTS.", "WRITE OVER? (Y/N):")
            }
            CustomCupMode::Load => self.paint_load(cx),
            CustomCupMode::ConfirmDelete => {
                self.paint_confirm(cx, "DELETE SELECTED SET?", "ARE YOU SURE? (Y/N):")
            }
            CustomCupMode::Message => self.paint_message(cx),
        }
    }

    fn paint_modal_box(&self, cx: &mut PaintCx<'_>, rect: (i32, i32, i32, i32)) {
        cx.fill((rect.0 - 1, rect.1 - 1, rect.2 + 2, rect.3 + 2), BLACK);
        cx.pattern_fill(rect, BG_PURPLE);
    }

    fn paint_save_input(&self, cx: &mut PaintCx<'_>) {
        self.paint_modal_box(cx, (75, 70, 170, 50));
        cx.text((85, 75), FONT_BODY, "Save Custom Hill Set");
        cx.text((85, 85), FONT_GOLD, "Filename");
        cx.fill((85, 95, 95, 20), BLACK);
        cx.text((95, 102), FONT_BODY, &self.filename_input);
        let cursor_x = 95 + self.resources.font.string_width(&self.filename_input) as i32;
        cx.fill((cursor_x, 108, 5, 1), FONT_BODY);
    }

    fn paint_load(&self, cx: &mut PaintCx<'_>) {
        self.paint_modal_box(cx, (75, 70, 170, 80));
        cx.text((85, 75), FONT_BODY, "Load Custom Hill Set");
        if self.load_entries.is_empty() {
            cx.text((95, 102), FONT_GRAY, "No custom sets found");
            return;
        }
        cx.text((85, 85), FONT_GOLD, "Filename");
        cx.fill((85, 95, 95, 20), BLACK);
        cx.text((95, 102), FONT_BODY, &self.load_entries[self.load_index]);
        cx.right_text(
            (220, 102),
            FONT_GOLD,
            format!("{} / {}", self.load_index + 1, self.load_entries.len()),
        );
        cx.text((85, 130), FONT_GRAY, "Enter load, Del delete, Esc exit");
    }

    fn paint_confirm(&self, cx: &mut PaintCx<'_>, line1: &str, line2: &str) {
        self.paint_modal_box(cx, (60, 80, 201, 51));
        cx.text((80, 90), FONT_GOLD, line1);
        cx.text((80, 110), FONT_GOLD, line2);
        cx.fill((190 - 2, 110 - 2, 9, 11), BG_PURPLE);
        cx.fill((190, 116, 5, 1), FONT_BODY);
    }

    fn paint_message(&self, cx: &mut PaintCx<'_>) {
        self.paint_modal_box(cx, (75, 80, 170, 40));
        cx.text((85, 92), FONT_BODY, &self.message);
        cx.text((85, 108), FONT_GRAY, "Press a key...");
    }

    fn handle_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        if self.mode != CustomCupMode::Browse {
            return self.handle_modal_input(event);
        }

        match event {
            UiEvent::KeyDown(Key::Escape) => Some(RouteTarget::Back),
            UiEvent::KeyDown(Key::Enter) => self.start_custom_cup(),
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
            UiEvent::Text('l' | 'L') => {
                self.open_load();
                None
            }
            UiEvent::Text('s' | 'S') => {
                self.filename_input = self.store.borrow().config.setfile.clone();
                if self.filename_input == "TEMP" || self.filename_input.is_empty() {
                    self.filename_input = "CUSTOM".to_string();
                }
                self.mode = CustomCupMode::SaveInput;
                None
            }
            UiEvent::Text('r' | 'R') => {
                self.randomize_selection();
                None
            }
            _ => None,
        }
    }

    fn handle_modal_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        match self.mode {
            CustomCupMode::SaveInput => self.handle_save_input(event),
            CustomCupMode::ConfirmOverwrite => self.handle_overwrite_confirm(event),
            CustomCupMode::Load => self.handle_load_input(event),
            CustomCupMode::ConfirmDelete => self.handle_delete_confirm(event),
            CustomCupMode::Message => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    self.mode = CustomCupMode::Browse;
                }
                None
            }
            CustomCupMode::Browse => None,
        }
    }

    fn handle_save_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        match event {
            UiEvent::KeyDown(Key::Escape) => self.mode = CustomCupMode::Browse,
            UiEvent::KeyDown(Key::Enter) => {
                self.filename_input = normalize_set_name(&self.filename_input);
                if self.filename_input.is_empty() {
                    self.show_message("Invalid filename");
                } else if self
                    .resources
                    .files
                    .exists_save(&set_path(&self.filename_input))
                {
                    self.mode = CustomCupMode::ConfirmOverwrite;
                } else {
                    self.save_current_set();
                }
            }
            UiEvent::KeyDown(Key::Backspace) => {
                self.filename_input.pop();
            }
            UiEvent::Text(ch) if self.filename_input.len() < 8 && ch.is_ascii_alphanumeric() => {
                self.filename_input.push(ch.to_ascii_uppercase());
            }
            _ => {}
        }
        None
    }

    fn handle_overwrite_confirm(&mut self, event: UiEvent) -> Option<RouteTarget> {
        match event {
            UiEvent::Text('y' | 'Y') => self.save_current_set(),
            UiEvent::Text('n' | 'N') | UiEvent::KeyDown(Key::Escape) => {
                self.mode = CustomCupMode::Browse;
            }
            _ => {}
        }
        None
    }

    fn handle_load_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        match event {
            UiEvent::KeyDown(Key::Escape) => self.mode = CustomCupMode::Browse,
            UiEvent::KeyDown(Key::Enter) => self.load_selected_set(),
            UiEvent::KeyDown(Key::Delete) => self.mode = CustomCupMode::ConfirmDelete,
            UiEvent::KeyDown(Key::Left | Key::Up) | UiEvent::Text('-') => {
                self.load_index = self.load_index.saturating_sub(1);
            }
            UiEvent::KeyDown(Key::Right | Key::Down) | UiEvent::Text('+' | ' ') => {
                if self.load_index + 1 < self.load_entries.len() {
                    self.load_index += 1;
                }
            }
            UiEvent::KeyDown(Key::Home) => self.load_index = 0,
            UiEvent::KeyDown(Key::End) => {
                self.load_index = self.load_entries.len().saturating_sub(1);
            }
            _ => {}
        }
        None
    }

    fn handle_delete_confirm(&mut self, event: UiEvent) -> Option<RouteTarget> {
        match event {
            UiEvent::Text('y' | 'Y') => {
                if let Some(name) = self.load_entries.get(self.load_index).cloned() {
                    if let Err(e) = self.resources.files.delete_save(&set_path(&name)) {
                        self.show_message(&format!("Delete failed: {e}"));
                    } else {
                        self.open_load();
                    }
                }
            }
            UiEvent::Text('n' | 'N') | UiEvent::KeyDown(Key::Escape) => {
                self.mode = CustomCupMode::Load;
            }
            _ => {}
        }
        None
    }

    fn start_custom_cup(&mut self) -> Option<RouteTarget> {
        if self.selected.is_empty() {
            return None;
        }
        let state = self.store.borrow();
        let comp = factory::custom_cup(
            &state.profiles,
            self.resources
                .player_names(state.config.namenumber as usize),
            self.selected.clone(),
            0,
        );
        drop(state);
        self.store.borrow_mut().start_active(comp);
        Some(RouteTarget::CompetitionJump)
    }

    fn open_load(&mut self) {
        self.load_entries = self
            .resources
            .files
            .list_save_subdir_by_ext(CUSTOM_CUP_DIR, "toml")
            .unwrap_or_default()
            .into_iter()
            .map(|name| name.trim_end_matches(".toml").to_string())
            .collect();
        self.load_index = self
            .load_index
            .min(self.load_entries.len().saturating_sub(1));
        self.mode = if self.load_entries.is_empty() {
            self.message = "No custom sets found".to_string();
            CustomCupMode::Message
        } else {
            CustomCupMode::Load
        };
    }

    fn load_selected_set(&mut self) {
        let Some(name) = self.load_entries.get(self.load_index).cloned() else {
            return;
        };
        let result = self
            .resources
            .files
            .read(&set_path(&name))
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| toml::from_str::<CustomCupFile>(&text).ok())
            .filter(|file| file.format_version == 1);
        let Some(file) = result else {
            self.show_message("Load failed");
            return;
        };
        let hills: Vec<usize> = file
            .hills
            .into_iter()
            .filter(|&idx| idx < self.all_hill_count)
            .take(MAX_HILLS)
            .collect();
        if hills.is_empty() {
            self.show_message("Load failed");
            return;
        }
        self.selected = hills;
        self.preview = self.selected.last().copied().unwrap_or(0);
        self.update_setfile(name);
        self.mode = CustomCupMode::Browse;
    }

    fn save_current_set(&mut self) {
        let name = normalize_set_name(&self.filename_input);
        let file = CustomCupFile {
            format_version: 1,
            hills: self.selected.clone(),
        };
        let Ok(bytes) = toml::to_string(&file).map(String::into_bytes) else {
            self.show_message("Save failed");
            return;
        };
        if let Err(e) = self.resources.files.write(&set_path(&name), &bytes) {
            self.show_message(&format!("Save failed: {e}"));
            return;
        }
        self.filename_input = name.clone();
        self.update_setfile(name);
        self.show_message("Custom set saved");
    }

    fn randomize_selection(&mut self) {
        let target = if self.selected.len() >= 20 {
            MAX_HILLS
        } else {
            20
        };
        let wc_hill_count = self.all_hill_count.min(20).max(1);
        let mut state = self.store.borrow_mut();
        while self.selected.len() < target && self.selected.len() < MAX_HILLS {
            let idx = state.rng.random_i32(wc_hill_count as i32).max(0) as usize;
            self.selected.push(idx);
        }
    }

    fn show_message(&mut self, message: &str) {
        self.message = message.to_string();
        self.mode = CustomCupMode::Message;
    }

    fn update_setfile(&self, name: String) {
        let mut state = self.store.borrow_mut();
        state.config.setfile = name;
        if let Err(e) = self.save_manager.save_config(&state.config) {
            eprintln!("Warning: failed to save config: {e}");
        }
    }
}

fn normalize_set_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(8)
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

fn set_path(name: &str) -> String {
    format!("{CUSTOM_CUP_DIR}/{}.toml", normalize_set_name(name))
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

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

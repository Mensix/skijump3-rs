use crate::competition::factory;
use crate::competition::types::{CustomCupScoring, MAX_CUSTOM_CUP_HILLS};
use crate::components::modal::{alert_box, confirmation_choice, ConfirmationChoice, Modal};
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_PURPLE, BLACK, FILL_GRAY, FILL_PURPLE, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::rng::Random;
use crate::route::RouteTarget;
use crate::save::custom_cup::{
    custom_cup_path, load_custom_cup_file, named_custom_cup_source, normalize_custom_cup_name,
    resolve_custom_cup_hills, save_custom_cup_file, CustomCupFile, CUSTOM_CUP_DIR, FORMAT_VERSION,
    MAX_CUSTOM_CUP_NAME_LEN,
};
use crate::screen::{GameCx, GameScreen, NavSignal, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::text::format;
use crate::text::format::format_decimal;
use crate::text::layout::shorten_name;
use crate::ui::UiCanvas;
use crate::ui::{Key, ScreenEventCx, UiEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomCupMode {
    Browse,
    SaveInput,
    ConfirmOverwrite,
    Load,
    ConfirmDelete,
    Message,
    Result(u8, i32),
}

fn shortcut(label: &str) -> Option<char> {
    let mut in_marker = false;
    label.chars().find_map(|ch| {
        if ch == '(' {
            in_marker = true;
            return None;
        }
        if in_marker && ch == ')' {
            in_marker = false;
            return None;
        }
        in_marker.then_some(ch.to_ascii_lowercase())
    })
}

fn randomize_unique_hills(selected: &mut Vec<usize>, hill_count: usize, rng: &mut Random) {
    let target = if selected.len() >= 20 { 40 } else { 20 };
    let target = target.min(MAX_CUSTOM_CUP_HILLS);
    let base_hill_count = hill_count.min(20);
    while selected.len() < target && base_hill_count > 0 {
        selected.push(rng.random_i32(base_hill_count as i32).max(0) as usize);
    }
}

pub struct CustomCupSetupView {
    resources: ResourcesRef,
    selected: Vec<usize>,
    preview: usize,
    all_hill_count: usize,
    scoring: CustomCupScoring,
    source_name: Option<String>,
    mode: CustomCupMode,
    filename_input: String,
    load_entries: Vec<String>,
    load_index: usize,
    message: String,
}

impl CustomCupSetupView {
    pub fn new(resources: ResourcesRef) -> Self {
        let count = resources.hills.len();
        Self {
            resources,
            selected: (count > 0).then_some(0).into_iter().collect(),
            preview: 0,
            all_hill_count: count,
            scoring: CustomCupScoring::default(),
            source_name: None,
            mode: CustomCupMode::Browse,
            filename_input: "CUSTOM".to_string(),
            load_entries: Vec::new(),
            load_index: 0,
            message: String::new(),
        }
    }

    fn paint_hill(&self, cx: &mut dyn UiCanvas, slot: usize, hill_idx: usize, is_preview: bool) {
        let (x, y) = if slot < 20 {
            (17, slot as i32 * 7 + 39)
        } else {
            (162, (slot as i32 - 20) * 7 + 39)
        };
        cx.fill((x, y - 1, 143, 9), FILL_PURPLE);
        if let Some(h) = self.resources.hills.hill(hill_idx) {
            let name = shorten_name(&h.name, &self.resources.font, 104);
            if is_preview {
                cx.text((x + 15, y), FONT_GRAY, &name);
                let name_w = self.resources.font.string_width(&name) as i32;
                let kr_str = format!("K{}", h.kr);
                cx.text((x + 18 + name_w, y), FONT_GRAY, &kr_str);
            } else {
                let num_str = format::ordinal_dot(slot + 1);
                cx.right_text((x + 14, y), FONT_GOLD, &num_str);
                let name_color = if hill_idx >= self.resources.hills.original_count() {
                    FONT_TEAL
                } else {
                    FONT_BODY
                };
                cx.text((x + 15, y), name_color, &name);
                let name_w = self.resources.font.string_width(&name) as i32;
                let kr_str = format!("K{}", h.kr);
                cx.text((x + 18 + name_w, y), name_color, &kr_str);
            }
        }
    }

    fn paint_content(&self, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        let help_line = format!("{}, {}, {}", lang.tr(285), lang.tr(286), lang.tr(287));
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 11, 200), FILL_GRAY);
        cx.pattern_fill((12, 0, 296, 200), BG_PURPLE);
        cx.pattern_fill((309, 0, 11, 200), FILL_GRAY);
        cx.sprite(sprites::Sprite::Logo as u16, (30, 8));
        cx.text((68, 8), FONT_BODY, lang.tr(118));
        cx.text((78, 16), FONT_GRAY, lang.tr(119));
        cx.text((78, 23), FONT_GRAY, &help_line);
        let scoring_label = match self.scoring {
            CustomCupScoring::WorldCupPoints => lang.tr(288),
            CustomCupScoring::AggregateJumpPoints => lang.tr(289),
        };
        cx.text((78, 30), FONT_GRAY, scoring_label);

        for (i, &hill_idx) in self.selected.iter().enumerate() {
            self.paint_hill(cx, i, hill_idx, false);
        }

        let preview_slot = self.selected.len();
        if preview_slot < MAX_CUSTOM_CUP_HILLS && self.preview < self.all_hill_count {
            self.paint_hill(cx, preview_slot, self.preview, true);
        }

        if let Some(source_name) = &self.source_name {
            cx.text((250, 30), FONT_GRAY, &format!("({source_name})"));
        }

        match self.mode {
            CustomCupMode::Browse => {}
            CustomCupMode::SaveInput => self.paint_save_input(cx),
            CustomCupMode::ConfirmOverwrite => self.paint_confirm(
                cx,
                &format!("FILE {}.TOML {}", self.filename_input, lang.tr(345)),
                lang.tr(346),
            ),
            CustomCupMode::Load => self.paint_load(cx),
            CustomCupMode::ConfirmDelete => {
                let name = self
                    .load_entries
                    .get(self.load_index)
                    .map_or("".to_string(), |name| format!("{name}.TOML"));
                self.paint_confirm(cx, &format!("DELETE {name}"), lang.tr(193))
            }
            CustomCupMode::Message => self.paint_message(cx),
            CustomCupMode::Result(phase, result) => Modal::result(phase, result).paint(cx, lang),
        }
    }

    fn paint_modal_box(&self, cx: &mut dyn UiCanvas, rect: (i32, i32, i32, i32)) {
        alert_box(cx, rect, BG_PURPLE);
    }

    fn paint_save_input(&self, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        self.paint_modal_box(cx, (75, 70, 170, 50));
        cx.text((85, 75), FONT_BODY, lang.tr(117));
        cx.text((85, 85), FONT_GOLD, lang.tr(273));
        cx.fill((85, 95, 95, 20), BLACK);
        cx.text((95, 102), FONT_BODY, &self.filename_input);
        let cursor_x = 95 + self.resources.font.string_width(&self.filename_input) as i32;
        cx.fill((cursor_x, 108, 5, 1), FONT_BODY);
    }

    fn paint_load(&self, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        self.paint_modal_box(cx, (75, 70, 170, 80));
        cx.text((85, 75), FONT_BODY, lang.tr(116));
        if self.load_entries.is_empty() {
            cx.text((95, 102), FONT_GRAY, lang.tr(356));
            return;
        }
        cx.text((85, 85), FONT_GOLD, lang.tr(273));
        cx.fill((85, 95, 95, 20), BLACK);
        cx.text((95, 102), FONT_BODY, &self.load_entries[self.load_index]);
        cx.right_text(
            (220, 102),
            FONT_GOLD,
            &format!("{} / {}", self.load_index + 1, self.load_entries.len()),
        );
        cx.text((85, 121), FONT_GOLD, lang.tr(115));
        if let Some(file) = self
            .load_entries
            .get(self.load_index)
            .and_then(|name| load_custom_cup_file(&self.resources.files, name))
        {
            if let Some(record) = file.records.iter().find(|record| !record.name.is_empty()) {
                cx.text(
                    (95, 130),
                    FONT_BODY,
                    &shorten_name(&record.name, &self.resources.font, 90),
                );
                cx.text((95, 139), FONT_GRAY, &record.time);
                let score = match file.scoring {
                    CustomCupScoring::WorldCupPoints => record.score.to_string(),
                    CustomCupScoring::AggregateJumpPoints => format_decimal(record.score),
                };
                cx.right_text((232, 130), FONT_GOLD, &score);
            } else {
                cx.text((95, 130), FONT_GRAY, "-");
            }
        }
    }

    fn paint_confirm(&self, cx: &mut dyn UiCanvas, line1: &str, line2: &str) {
        Modal::confirm(line1, line2).paint(cx, &self.resources.langbase);
    }

    fn paint_message(&self, cx: &mut dyn UiCanvas) {
        let lang = &self.resources.langbase;
        self.paint_modal_box(cx, (59, 79, 202, 42));
        cx.text((85, 92), FONT_BODY, &self.message);
        cx.text((85, 108), FONT_GRAY, lang.tr(15));
    }

    fn handle_input(
        &mut self,
        state: &mut GameState,
        event: UiEvent,
        cx: &Persistence,
    ) -> NavSignal {
        if self.mode != CustomCupMode::Browse {
            return self.handle_modal_input(state, event, cx);
        }
        if matches!(event, UiEvent::KeyDown(Key::F10)) {
            return NavSignal::Back;
        }

        match event {
            UiEvent::KeyDown(Key::Enter) => self.start_custom_cup(state),
            UiEvent::KeyDown(Key::Left) => {
                if self.preview > 0 {
                    self.preview -= 1;
                }
                NavSignal::None
            }
            UiEvent::KeyDown(Key::Right) => {
                if self.preview + 1 < self.all_hill_count {
                    self.preview += 1;
                }
                NavSignal::None
            }
            UiEvent::KeyDown(Key::Home | Key::Delete) => {
                self.preview = 0;
                NavSignal::None
            }
            UiEvent::KeyDown(Key::End) => {
                self.preview = self.all_hill_count.saturating_sub(1);
                NavSignal::None
            }
            UiEvent::KeyDown(Key::PageUp) => {
                self.preview = self.preview.saturating_sub(5);
                NavSignal::None
            }
            UiEvent::KeyDown(Key::PageDown) => {
                self.preview = (self.preview + 5).min(self.all_hill_count.saturating_sub(1));
                NavSignal::None
            }
            UiEvent::KeyDown(Key::Down) | UiEvent::Text(' ') => {
                if self.all_hill_count > 0 && self.selected.len() < MAX_CUSTOM_CUP_HILLS {
                    self.selected.push(self.preview);
                    self.source_name = None;
                }
                NavSignal::None
            }
            UiEvent::KeyDown(Key::Up | Key::Backspace) => {
                if self.selected.pop().is_some() {
                    self.source_name = None;
                }
                NavSignal::None
            }
            UiEvent::Text(ch)
                if shortcut(self.resources.langbase.tr(285)) == Some(ch.to_ascii_lowercase()) =>
            {
                self.open_load();
                NavSignal::None
            }
            UiEvent::Text(ch)
                if shortcut(self.resources.langbase.tr(286)) == Some(ch.to_ascii_lowercase()) =>
            {
                self.filename_input = state.config.last_custom_cup_file.clone();
                if self.filename_input == "TEMP" || self.filename_input.is_empty() {
                    self.filename_input = "CUSTOM".to_string();
                }
                self.mode = CustomCupMode::SaveInput;
                NavSignal::None
            }
            UiEvent::Text(ch)
                if shortcut(self.resources.langbase.tr(287)) == Some(ch.to_ascii_lowercase()) =>
            {
                self.randomize_selection(state);
                NavSignal::None
            }
            UiEvent::Text(ch)
                if shortcut(self.resources.langbase.tr(288)) == Some(ch.to_ascii_lowercase()) =>
            {
                self.scoring = match self.scoring {
                    CustomCupScoring::WorldCupPoints => CustomCupScoring::AggregateJumpPoints,
                    CustomCupScoring::AggregateJumpPoints => CustomCupScoring::WorldCupPoints,
                };
                self.source_name = None;
                NavSignal::None
            }
            UiEvent::KeyDown(Key::Escape | Key::F10) => NavSignal::Back,
            _ => NavSignal::None,
        }
    }

    fn handle_modal_input(
        &mut self,
        state: &mut GameState,
        event: UiEvent,
        cx: &Persistence,
    ) -> NavSignal {
        match self.mode {
            CustomCupMode::SaveInput => self.handle_save_input(state, event, cx),
            CustomCupMode::ConfirmOverwrite => self.handle_overwrite_confirm(state, event, cx),
            CustomCupMode::Load => self.handle_load_input(state, event, cx),
            CustomCupMode::ConfirmDelete => self.handle_delete_confirm(event),
            CustomCupMode::Message => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    self.mode = CustomCupMode::Browse;
                }
                NavSignal::None
            }
            CustomCupMode::Result(phase, result) => {
                let modal = Modal::result(phase, result);
                if modal.event(event, &self.resources.langbase).is_some() {
                    self.mode = CustomCupMode::Browse;
                }
                NavSignal::None
            }
            CustomCupMode::Browse => NavSignal::None,
        }
    }

    fn handle_save_input(
        &mut self,
        state: &mut GameState,
        event: UiEvent,
        cx: &Persistence,
    ) -> NavSignal {
        match event {
            UiEvent::KeyDown(Key::Enter) => {
                self.filename_input = normalize_custom_cup_name(&self.filename_input);
                if self.filename_input.is_empty() {
                    self.show_localized_message(353);
                } else if self
                    .resources
                    .files
                    .exists_save(&custom_cup_path(&self.filename_input))
                {
                    self.mode = CustomCupMode::ConfirmOverwrite;
                } else {
                    self.save_current_set(state, cx);
                }
            }
            UiEvent::KeyDown(Key::Backspace | Key::Delete) => {
                self.filename_input.pop();
            }
            UiEvent::Text(ch)
                if self.filename_input.len() < MAX_CUSTOM_CUP_NAME_LEN
                    && ch.is_ascii_graphic()
                    && !matches!(ch, '/' | '\\' | '.') =>
            {
                self.filename_input.push(ch.to_ascii_uppercase());
            }
            UiEvent::KeyDown(Key::Escape) => self.mode = CustomCupMode::Browse,
            _ => {}
        }
        NavSignal::None
    }

    fn handle_overwrite_confirm(
        &mut self,
        state: &mut GameState,
        event: UiEvent,
        cx: &Persistence,
    ) -> NavSignal {
        match event {
            UiEvent::Text(c)
                if confirmation_choice(c, &self.resources.langbase)
                    == Some(ConfirmationChoice::Yes) =>
            {
                self.save_current_set(state, cx)
            }
            UiEvent::Text(c)
                if confirmation_choice(c, &self.resources.langbase)
                    == Some(ConfirmationChoice::No) =>
            {
                self.mode = CustomCupMode::Browse;
            }
            UiEvent::KeyDown(Key::Escape) => self.mode = CustomCupMode::Browse,
            _ => {}
        }
        NavSignal::None
    }

    fn handle_load_input(
        &mut self,
        state: &mut GameState,
        event: UiEvent,
        cx: &Persistence,
    ) -> NavSignal {
        match event {
            UiEvent::KeyDown(Key::Enter) => self.load_selected_set(state, cx),
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
            UiEvent::KeyDown(Key::Escape) => self.mode = CustomCupMode::Browse,
            _ => {}
        }
        NavSignal::None
    }

    fn handle_delete_confirm(&mut self, event: UiEvent) -> NavSignal {
        match event {
            UiEvent::Text(c)
                if confirmation_choice(c, &self.resources.langbase)
                    == Some(ConfirmationChoice::Yes) =>
            {
                if let Some(name) = self.load_entries.get(self.load_index).cloned() {
                    let path = custom_cup_path(&name);
                    self.resources.files.delete_save(&path);
                    if self.resources.files.exists_save(&path) {
                        self.show_localized_message(353);
                    } else {
                        if self.source_name.as_deref() == named_custom_cup_source(&name).as_deref()
                        {
                            self.source_name = None;
                        }
                        self.load_entries.remove(self.load_index);
                        if self.load_entries.is_empty() {
                            self.mode = CustomCupMode::Result(4, 0);
                        } else {
                            self.load_index = self.load_index.min(self.load_entries.len() - 1);
                            self.mode = CustomCupMode::Load;
                        }
                    }
                }
            }
            UiEvent::Text(c)
                if confirmation_choice(c, &self.resources.langbase)
                    == Some(ConfirmationChoice::No) =>
            {
                self.mode = CustomCupMode::Load;
            }
            UiEvent::KeyDown(Key::Escape) => self.mode = CustomCupMode::Load,
            _ => {}
        }
        NavSignal::None
    }

    fn start_custom_cup(&mut self, state: &mut GameState) -> NavSignal {
        if self.selected.is_empty() {
            return NavSignal::None;
        }
        let mut comp = factory::custom_cup(
            &state.profiles,
            self.resources
                .player_names(state.config.name_set_index as usize),
            self.selected.clone(),
            state.config.training_rounds as usize,
            state.config.unique_computer_names != 0,
            state.config.ko_system != 0,
            self.scoring,
        );
        if let Some(competition) = comp.individual_mut() {
            competition.custom_cup_file = self.source_name.clone();
        }
        state.start_active(comp);
        NavSignal::Route(RouteTarget::CompetitionJump)
    }

    fn open_load(&mut self) {
        self.load_entries = self
            .resources
            .files
            .list_save_subdir_by_ext(CUSTOM_CUP_DIR, "toml")
            .into_iter()
            .map(|name| name.trim_end_matches(".toml").to_string())
            .collect();
        self.load_index = self
            .load_index
            .min(self.load_entries.len().saturating_sub(1));
        self.mode = if self.load_entries.is_empty() {
            CustomCupMode::Result(4, 0)
        } else {
            CustomCupMode::Load
        };
    }

    fn load_selected_set(&mut self, state: &mut GameState, cx: &Persistence) {
        let Some(name) = self.load_entries.get(self.load_index).cloned() else {
            return;
        };
        let Some(file) = load_custom_cup_file(&self.resources.files, &name) else {
            self.mode = CustomCupMode::Message;
            self.message = self.resources.langbase.tr(356).to_string();
            return;
        };
        let Some(hills) = resolve_custom_cup_hills(&file, &self.resources.hills) else {
            self.mode = CustomCupMode::Message;
            self.message = self.resources.langbase.tr(356).to_string();
            return;
        };
        if hills.is_empty() {
            self.mode = CustomCupMode::Message;
            self.message = self.resources.langbase.tr(356).to_string();
            return;
        }
        self.selected = hills;
        self.scoring = file.scoring;
        self.source_name = named_custom_cup_source(&name);
        self.preview = 0;
        if !self.update_last_custom_cup_file(state, name, cx) {
            self.show_localized_message(353);
            return;
        }
        self.mode = CustomCupMode::Browse;
    }

    fn save_current_set(&mut self, state: &mut GameState, cx: &Persistence) {
        let name = normalize_custom_cup_name(&self.filename_input);
        let records = load_custom_cup_file(&self.resources.files, &name)
            .map_or_else(Vec::new, |file| file.records);
        let file = CustomCupFile {
            format_version: FORMAT_VERSION,
            hill_refs: self
                .selected
                .iter()
                .filter_map(|&idx| self.resources.hills.hill(idx))
                .map(|hill| hill.record_key.clone())
                .collect(),
            scoring: self.scoring,
            records,
        };
        if !save_custom_cup_file(&self.resources.files, &name, &file) {
            self.mode = CustomCupMode::Result(2, 1);
            return;
        }
        self.filename_input = name.clone();
        self.source_name = named_custom_cup_source(&name);
        if !self.update_last_custom_cup_file(state, name, cx) {
            self.show_localized_message(353);
            return;
        }
        self.mode = CustomCupMode::Result(2, 0);
    }

    fn randomize_selection(&mut self, state: &mut GameState) {
        randomize_unique_hills(&mut self.selected, self.all_hill_count, &mut state.rng);
        self.source_name = None;
    }

    fn show_message(&mut self, message: &str) {
        self.message = message.to_string();
        self.mode = CustomCupMode::Message;
    }

    fn show_localized_message(&mut self, index: usize) {
        let message = self.resources.langbase.tr(index).to_string();
        self.show_message(&message);
    }

    fn update_last_custom_cup_file(
        &self,
        state: &mut GameState,
        name: String,
        cx: &Persistence,
    ) -> bool {
        let Some(name) = named_custom_cup_source(&name) else {
            return true;
        };
        state.config.last_custom_cup_file = name;
        cx.save_config(&state.config)
    }
}

impl GameScreen for CustomCupSetupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let persistence = cx.persistence();
        self.handle_input(cx.state, event, &persistence)
            .dispatch(nav);
    }

    fn paint(&mut self, _cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint);
    }
}

#[cfg(test)]
mod tests {
    use super::randomize_unique_hills;
    use crate::rng::Random;

    #[test]
    fn randomization_preserves_positions_and_fills_from_base_hills() {
        let mut selected = vec![0, 0, 22];
        randomize_unique_hills(&mut selected, 25, &mut Random::new(0));

        assert_eq!(&selected[..3], &[0, 0, 22]);
        assert_eq!(selected.len(), 20);
        assert!(selected[3..].iter().all(|&idx| idx < 20));
    }

    #[test]
    fn randomization_fills_to_forty_when_already_at_least_twenty() {
        let mut selected = (0..20).collect();
        randomize_unique_hills(&mut selected, 20, &mut Random::new(0));

        assert_eq!(selected.len(), 40);
        assert_eq!(&selected[..20], &(0..20).collect::<Vec<_>>());
        assert!(selected[20..].iter().all(|&idx| idx < 20));
    }

    #[test]
    fn randomization_does_not_roll_from_an_empty_hill_catalog() {
        let mut selected = vec![7];
        randomize_unique_hills(&mut selected, 0, &mut Random::new(0));

        assert_eq!(selected, vec![7]);
    }
}

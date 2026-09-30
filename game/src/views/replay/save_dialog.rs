use crate::components::modal::{confirmation_choice, ConfirmationChoice, Modal};
use crate::components::page_nav::cycle_index;
use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD};
use crate::jump::replay::ReplayTrace;
use crate::screen::Persistence;
use crate::store::ResourcesRef;
use crate::text::format::display_timestamp_now;
use crate::ui::UiCanvas;
use crate::ui::{Blinker, Key, TextEditState, UiEvent};

#[derive(Debug, Clone, Copy)]
enum SaveField {
    Author,
    Name,
    Filename,
}

impl SaveField {
    const fn idx(self) -> usize {
        match self {
            Self::Author => 0,
            Self::Name => 1,
            Self::Filename => 2,
        }
    }

    const fn max_len(self) -> usize {
        match self {
            Self::Author | Self::Name => usize::MAX,
            Self::Filename => 8,
        }
    }
}

#[derive(Debug, Clone)]
enum SaveDialogState {
    Inactive,
    Browse {
        selected: usize,
    },
    EditField {
        field: SaveField,
        editor: TextEditState,
    },
    ConfirmOverwrite {
        filename: String,
    },
    SaveResult {
        result: i32,
    },
}

pub enum SaveAction {
    Consumed,
    SaveReplay,
}

pub struct SaveReplayDialog {
    resources: ResourcesRef,
    state: SaveDialogState,
    author: String,
    name: String,
    filename: String,
    distance: String,
    hill_name: String,
    cursor_blink: Blinker,
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

impl SaveReplayDialog {
    pub fn new(resources: ResourcesRef) -> Self {
        Self {
            resources,
            state: SaveDialogState::Inactive,
            author: String::new(),
            name: String::new(),
            filename: "TEMP".to_string(),
            distance: String::new(),
            hill_name: String::new(),
            cursor_blink: Blinker::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, SaveDialogState::Inactive)
    }

    pub fn save_result(&self) -> Option<i32> {
        match self.state {
            SaveDialogState::SaveResult { result } => Some(result),
            _ => None,
        }
    }

    fn is_form_active(&self) -> bool {
        matches!(
            self.state,
            SaveDialogState::Browse { .. } | SaveDialogState::EditField { .. }
        )
    }

    pub fn open(
        &mut self,
        initial_author: String,
        initial_name: String,
        distance: String,
        hill_name: String,
    ) {
        self.author = initial_author;
        self.name = initial_name;
        self.distance = distance;
        self.hill_name = hill_name;
        self.filename = "TEMP".to_string();
        self.state = SaveDialogState::Browse { selected: 0 };
    }

    fn edit_field(&mut self, field: SaveField, value: String) -> SaveAction {
        self.cursor_blink.reset();
        self.state = SaveDialogState::EditField {
            field,
            editor: TextEditState::new(value, field.max_len()),
        };
        SaveAction::Consumed
    }

    fn activate_item(&mut self, selected: usize) -> SaveAction {
        match selected {
            0 => self.edit_field(SaveField::Author, self.author.clone()),
            1 => self.edit_field(SaveField::Name, self.name.clone()),
            2 => self.edit_field(SaveField::Filename, self.filename.clone()),
            3 => {
                self.state = SaveDialogState::Inactive;
                SaveAction::Consumed
            }
            4 => {
                self.cursor_blink.reset();
                let safe = sanitize_filename(&self.filename);
                if self.resources.files.exists_save(&format!("{safe}.SJR")) {
                    self.state = SaveDialogState::ConfirmOverwrite { filename: safe };
                } else {
                    return SaveAction::SaveReplay;
                }
                SaveAction::Consumed
            }
            _ => SaveAction::Consumed,
        }
    }

    fn handle_browse_digit(&mut self, c: char) -> SaveAction {
        match browse_digit_index(c) {
            Some(selected) => {
                self.state = SaveDialogState::Browse { selected };
                SaveAction::Consumed
            }
            _ => SaveAction::Consumed,
        }
    }

    pub fn write_replay(&mut self, trace: &ReplayTrace, cx: &Persistence) {
        let safe_name = sanitize_filename(&self.filename);
        let filename = format!("{safe_name}.SJR");
        let mut trace = trace.clone();
        trace.meta.author.clone_from(&self.author);
        trace.meta.name.clone_from(&self.name);
        trace.meta.saved_at = display_timestamp_now();
        let result = i32::from(!cx.write_file(&filename, &trace.to_sjr_bytes()));
        self.state = SaveDialogState::SaveResult { result };
    }

    fn paint_save_result(&self, cx: &mut dyn UiCanvas, result: i32) {
        Modal::result(1, result).paint(cx, &self.resources.langbase);
    }

    pub fn paint(&self, cx: &mut dyn UiCanvas) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
        cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);
        cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

        let is_overlay = matches!(
            self.state,
            SaveDialogState::ConfirmOverwrite { .. } | SaveDialogState::SaveResult { .. }
        );
        if is_overlay || self.is_form_active() {
            self.paint_form(cx, !is_overlay);
        }
        if let SaveDialogState::ConfirmOverwrite { ref filename } = self.state {
            self.paint_overwrite(cx, filename);
        }
        if let Some(result) = self.save_result() {
            self.paint_save_result(cx, result);
        }
    }

    fn paint_form(&self, cx: &mut dyn UiCanvas, show_box: bool) {
        let lang = &self.resources.langbase;
        let selected_idx = match self.state {
            SaveDialogState::Browse { selected } => selected,
            SaveDialogState::EditField { ref field, .. } => field.idx(),
            SaveDialogState::SaveResult { .. } => 4,
            _ => 0,
        };
        let editing = matches!(self.state, SaveDialogState::EditField { .. });
        let editing_field = match self.state {
            SaveDialogState::EditField { ref field, .. } => Some(field.idx()),
            _ => None,
        };

        cx.text(
            (30, 6),
            FONT_BODY,
            &format!("{}: {}µ at {}", lang.tr(25), self.distance, self.hill_name),
        );

        for i in 0..5 {
            let yy = (i * 16 + 42) as i32;
            let final_yy = if i == 4 { yy + 16 } else { yy };
            let label_color = if i < 4 { FONT_BODY } else { FONT_GOLD };
            let label = match i {
                0..=2 => format!("{}. {}", i + 1, lang.tr(291 + i)),
                3 => format!("4. {}", lang.tr(295)),
                4 => format!("5. {}", lang.tr(296)),
                _ => String::new(),
            };
            cx.text((18, final_yy), label_color, &label);

            if i < 3 {
                let is_editing = editing && editing_field == Some(i);
                let value = if is_editing {
                    if let SaveDialogState::EditField { ref editor, .. } = self.state {
                        editor.buffer().to_string()
                    } else {
                        String::new()
                    }
                } else {
                    match i {
                        0 => self.author.clone(),
                        1 => self.name.clone(),
                        2 => self.filename.clone(),
                        _ => String::new(),
                    }
                };

                if is_editing {
                    let fw = match editing_field {
                        Some(2) => 60,
                        _ => 134,
                    };
                    cx.fill((146, final_yy - 2, fw, 10), BLACK);
                }
                cx.text((148, final_yy), FONT_GOLD, &value);

                if is_editing {
                    if let SaveDialogState::EditField { ref editor, .. } = self.state {
                        let cx_pos = 148
                            + self
                                .resources
                                .font
                                .string_width(&editor.buffer()[..editor.cursor_byte()])
                                as i32;
                        if self.cursor_blink.visible(11, 10) {
                            cx.fill((cx_pos, final_yy + 6, 5, 1), FONT_BODY);
                        }
                    }
                }
            }
        }

        if show_box {
            let box_y = if selected_idx < 4 {
                36 + selected_idx * 16
            } else {
                36 + 5 * 16
            };
            cx.stroke((9, box_y as i32, 135, 17), FONT_BODY);
        }
    }

    fn paint_overwrite(&self, cx: &mut dyn UiCanvas, filename: &str) {
        let lang = &self.resources.langbase;
        Modal::confirm(format!("{}.SJR {}", filename, lang.tr(345)), lang.tr(346)).paint(cx, lang);
    }

    pub fn handle_event(&mut self, event: &UiEvent) -> Option<SaveAction> {
        let state = self.state.clone();
        match state {
            SaveDialogState::Browse { selected } => match event {
                UiEvent::KeyDown(Key::Escape) => {
                    self.state = SaveDialogState::Inactive;
                    Some(SaveAction::Consumed)
                }
                UiEvent::KeyDown(Key::Up) => {
                    let next = cycle_index(selected, 5, -1);
                    self.state = SaveDialogState::Browse { selected: next };
                    Some(SaveAction::Consumed)
                }
                UiEvent::KeyDown(Key::Down) => {
                    let next = cycle_index(selected, 5, 1);
                    self.state = SaveDialogState::Browse { selected: next };
                    Some(SaveAction::Consumed)
                }
                UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                    Some(self.activate_item(selected))
                }
                UiEvent::Text(c) => Some(self.handle_browse_digit(*c)),
                _ => Some(SaveAction::Consumed),
            },
            SaveDialogState::EditField { field, mut editor } => match event {
                UiEvent::KeyDown(Key::Escape) => {
                    self.cursor_blink.reset();
                    self.state = SaveDialogState::Browse {
                        selected: field.idx(),
                    };
                    Some(SaveAction::Consumed)
                }
                UiEvent::KeyDown(Key::Enter) => {
                    self.cursor_blink.reset();
                    let buf = editor.buffer().to_string();
                    match field {
                        SaveField::Author => self.author = buf,
                        SaveField::Name => self.name = buf,
                        SaveField::Filename => self.filename = buf,
                    }
                    self.state = SaveDialogState::Browse {
                        selected: field.idx(),
                    };
                    Some(SaveAction::Consumed)
                }
                UiEvent::KeyDown(Key::Backspace) => {
                    if editor.backspace() {
                        self.cursor_blink.reset();
                        self.state = SaveDialogState::EditField { field, editor };
                    }
                    Some(SaveAction::Consumed)
                }
                UiEvent::KeyDown(Key::Delete) => {
                    editor.set_buffer(String::new());
                    self.cursor_blink.reset();
                    self.state = SaveDialogState::EditField { field, editor };
                    Some(SaveAction::Consumed)
                }
                UiEvent::Text(c) => {
                    if editor.insert(*c) {
                        self.cursor_blink.reset();
                        self.state = SaveDialogState::EditField { field, editor };
                    }
                    Some(SaveAction::Consumed)
                }
                _ => Some(SaveAction::Consumed),
            },
            SaveDialogState::ConfirmOverwrite { .. } => match event {
                UiEvent::KeyDown(Key::Escape) => {
                    self.cursor_blink.reset();
                    self.state = SaveDialogState::Browse { selected: 2 };
                    Some(SaveAction::Consumed)
                }
                UiEvent::Text(c)
                    if confirmation_choice(*c, &self.resources.langbase)
                        == Some(ConfirmationChoice::Yes) =>
                {
                    Some(SaveAction::SaveReplay)
                }
                UiEvent::Text(c)
                    if confirmation_choice(*c, &self.resources.langbase)
                        == Some(ConfirmationChoice::No) =>
                {
                    self.state = SaveDialogState::Browse { selected: 2 };
                    Some(SaveAction::Consumed)
                }
                _ => Some(SaveAction::Consumed),
            },
            SaveDialogState::SaveResult { result } => {
                if Modal::result(1, result)
                    .event(*event, &self.resources.langbase)
                    .is_some()
                {
                    self.state = SaveDialogState::Browse { selected: 4 };
                }
                Some(SaveAction::Consumed)
            }
            SaveDialogState::Inactive => Some(SaveAction::Consumed),
        }
    }
}

fn browse_digit_index(c: char) -> Option<usize> {
    c.to_digit(10)
        .filter(|digit| (1..=5).contains(digit))
        .map(|digit| digit as usize - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_filenames_have_a_character_limit() {
        assert_eq!(SaveField::Author.max_len(), usize::MAX);
        assert_eq!(SaveField::Name.max_len(), usize::MAX);
        assert_eq!(SaveField::Filename.max_len(), 8);
    }

    #[test]
    fn displayed_save_shortcuts_map_to_zero_based_items() {
        assert_eq!(browse_digit_index('1'), Some(0));
        assert_eq!(browse_digit_index('5'), Some(4));
        assert_eq!(browse_digit_index('0'), None);
        assert_eq!(browse_digit_index('6'), None);
    }
}

use crate::components::page_nav::cycle_index;
use crate::gfx::sprites;
use crate::gfx::theme::{
    BG_LEFT, BG_RIGHT, BLACK, DITHER_FILL_COLORS, FILL_DIM, FONT_DEFAULT, FONT_GOLD,
};
use crate::jump::replay::ReplayTrace;
use crate::store::ResourcesRef;
use engine::oxide::input::{Key, UiEvent};
use engine::oxide::PaintCx;
use engine::oxide::{Blinker, TextEditState};

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
            Self::Author | Self::Name => 130,
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
        match c.to_digit(10) {
            Some(d) if (1..=5).contains(&d) => {
                self.state = SaveDialogState::Browse {
                    selected: d as usize - 1,
                };
                SaveAction::Consumed
            }
            Some(0) => SaveAction::Consumed,
            _ => SaveAction::Consumed,
        }
    }

    pub fn write_replay(&mut self, trace: &ReplayTrace) {
        let safe_name = sanitize_filename(&self.filename);
        let filename = format!("{safe_name}.SJR");
        if let Err(e) = self.resources.files.write(&filename, &trace.to_sjr_bytes()) {
            eprintln!("Warning: failed to save replay {filename}: {e}");
        }
        self.state = SaveDialogState::Inactive;
    }

    pub fn paint(&self, cx: &mut PaintCx<'_>) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.fill((0, 0, 320, 19), FILL_DIM);
        cx.fill((0, 20, 320, 180), BG_LEFT);
        cx.dither_fill(63, DITHER_FILL_COLORS);
        cx.sprite(sprites::Sprite::Logo as u16, (5, 2));

        let is_overlay = matches!(self.state, SaveDialogState::ConfirmOverwrite { .. });
        if is_overlay || self.is_form_active() {
            self.paint_form(cx, !is_overlay);
        }
        if let SaveDialogState::ConfirmOverwrite { ref filename } = self.state {
            self.paint_overwrite(cx, filename);
        }
    }

    fn paint_form(&self, cx: &mut PaintCx<'_>, show_box: bool) {
        let selected_idx = match self.state {
            SaveDialogState::Browse { selected } => selected,
            SaveDialogState::EditField { ref field, .. } => field.idx(),
            _ => 0,
        };
        let editing = matches!(self.state, SaveDialogState::EditField { .. });
        let editing_field = match self.state {
            SaveDialogState::EditField { ref field, .. } => Some(field.idx()),
            _ => None,
        };

        cx.text(
            (30, 6),
            FONT_DEFAULT,
            format!(
                "{}: {}µ at {}",
                self.resources.langbase.lstr(25),
                self.distance,
                self.hill_name
            ),
        );

        for i in 0..5 {
            let yy = (i * 16 + 42) as i32;
            let final_yy = if i == 4 { yy + 16 } else { yy };
            let label_color = if i < 4 { FONT_DEFAULT } else { FONT_GOLD };
            let label = match i {
                0..=2 => format!("{}. {}", i + 1, self.resources.langbase.lstr(291 + i)),
                3 => format!("4. {}", self.resources.langbase.lstr(295)),
                4 => format!("5. {}", self.resources.langbase.lstr(296)),
                _ => String::new(),
            };
            cx.text((18, final_yy), label_color, label);

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
                cx.text((148, final_yy), FONT_GOLD, value);

                if is_editing {
                    if let SaveDialogState::EditField { ref editor, .. } = self.state {
                        let cx_pos = 148
                            + self
                                .resources
                                .font
                                .string_width(&editor.buffer()[..editor.cursor_byte()])
                                as i32;
                        if self.cursor_blink.visible(11, 10) {
                            cx.fill((cx_pos, final_yy + 6, 5, 1), FONT_DEFAULT);
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
            cx.stroke((9, box_y as i32, 135, 17), FONT_DEFAULT);
        }
    }

    fn paint_overwrite(&self, cx: &mut PaintCx<'_>, filename: &str) {
        cx.fill((59, 79, 203, 53), BLACK);
        cx.fill((60, 80, 201, 51), BG_RIGHT);
        cx.text(
            (80, 90),
            FONT_GOLD,
            format!("{}.SJR {}", filename, self.resources.langbase.lstr(345)),
        );
        cx.text(
            (80, 110),
            FONT_GOLD,
            format!("{} (Y/N):", self.resources.langbase.lstr(346)),
        );
        cx.fill((190 - 2, 110 - 2, 9, 11), BG_LEFT);
        if self.cursor_blink.visible(11, 10) {
            cx.fill((190, 110 + 6, 5, 1), FONT_DEFAULT);
        }
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
                    self.state = SaveDialogState::Browse { selected: 2 };
                    Some(SaveAction::Consumed)
                }
                UiEvent::Text(c) if *c == 'y' || *c == 'Y' => Some(SaveAction::SaveReplay),
                UiEvent::Text(c) if *c == 'n' || *c == 'N' => {
                    self.state = SaveDialogState::Browse { selected: 2 };
                    Some(SaveAction::Consumed)
                }
                _ => Some(SaveAction::Consumed),
            },
            SaveDialogState::Inactive => Some(SaveAction::Consumed),
        }
    }
}

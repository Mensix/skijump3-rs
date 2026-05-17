use crate::components::screen;
use crate::gfx::palette::{FONT_DEFAULT, FONT_GOLD};
use crate::jump::replay::ReplayTrace;
use crate::store::ResourcesRef;
use engine::ui::{Blinker, Component, Element, Event, Key, TextEditState};
use std::path::Path;

#[derive(Debug, Clone)]
enum SaveField {
    Author,
    Name,
    Filename,
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

const fn field_idx(field: &SaveField) -> usize {
    match field {
        SaveField::Author => 0,
        SaveField::Name => 1,
        SaveField::Filename => 2,
    }
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

    fn activate_item(&mut self, selected: usize) -> SaveAction {
        match selected {
            0 => {
                self.cursor_blink.reset();
                self.state = SaveDialogState::EditField {
                    field: SaveField::Author,
                    editor: TextEditState::new(self.author.clone(), 130),
                };
                SaveAction::Consumed
            }
            1 => {
                self.cursor_blink.reset();
                self.state = SaveDialogState::EditField {
                    field: SaveField::Name,
                    editor: TextEditState::new(self.name.clone(), 130),
                };
                SaveAction::Consumed
            }
            2 => {
                self.cursor_blink.reset();
                self.state = SaveDialogState::EditField {
                    field: SaveField::Filename,
                    editor: TextEditState::new(self.filename.clone(), 8),
                };
                SaveAction::Consumed
            }
            3 => {
                self.state = SaveDialogState::Inactive;
                SaveAction::Consumed
            }
            4 => {
                self.cursor_blink.reset();
                let filename = self.filename.clone();
                if Path::new(&format!("{filename}.SJR")).exists() {
                    self.state = SaveDialogState::ConfirmOverwrite { filename };
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
        let filename = format!("{}.SJR", self.filename);
        let _ = std::fs::write(&filename, trace.to_sjr_bytes());
        self.state = SaveDialogState::Inactive;
    }
}

impl Component for SaveReplayDialog {
    type Action = SaveAction;

    fn elements(&self) -> Vec<Element> {
        let mut els = screen::new_screen(1);

        let overlay = matches!(self.state, SaveDialogState::ConfirmOverwrite { .. });
        if overlay
            || matches!(self.state, SaveDialogState::Browse { .. })
            || matches!(self.state, SaveDialogState::EditField { .. })
        {
            let selected_idx = match self.state {
                SaveDialogState::Browse { selected } => selected,
                SaveDialogState::EditField { ref field, .. } => field_idx(field),
                SaveDialogState::ConfirmOverwrite { .. } => 4,
                _ => 0,
            };
            let editing = matches!(self.state, SaveDialogState::EditField { .. });
            let editing_field = match self.state {
                SaveDialogState::EditField { ref field, .. } => Some(field_idx(field)),
                _ => None,
            };

            els.push(Element::text(
                format!(
                    "{}: {}µ at {}",
                    self.resources.langbase.lstr(25),
                    self.distance,
                    self.hill_name
                ),
                30,
                6,
                FONT_DEFAULT,
                false,
            ));

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
                els.push(Element::text(&label, 18, final_yy, label_color, false));

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
                        let (fw, fh) = match editing_field {
                            Some(2) => (60, 11),
                            _ => (134, 10),
                        };
                        els.push(Element::fillbox(146, final_yy - 2, fw, fh, 242));
                    }
                    els.push(Element::text(&value, 148, final_yy, FONT_GOLD, false));

                    if is_editing {
                        if let SaveDialogState::EditField { ref editor, .. } = self.state {
                            let cx = 148
                                + self
                                    .resources
                                    .font
                                    .string_width(&editor.buffer()[..editor.cursor_byte()])
                                    as i32;
                            if self.cursor_blink.visible(11, 10) {
                                els.push(Element::fillbox(cx, final_yy + 6, 5, 1, FONT_DEFAULT));
                            }
                        }
                    }
                }
            }

            if !overlay {
                let box_y = if selected_idx < 4 {
                    36 + selected_idx * 16
                } else {
                    36 + 5 * 16
                };
                els.push(Element::box_(9, box_y as i32, 135, 17, FONT_DEFAULT));
            }
        }

        if let SaveDialogState::ConfirmOverwrite { ref filename } = self.state {
            els.push(Element::fillbox(59, 79, 203, 53, 242));
            els.push(Element::fillbox(60, 80, 201, 51, 244));
            els.push(Element::FillArea { thing: 63 });

            els.push(Element::text(
                format!("{}.SJR {}", filename, self.resources.langbase.lstr(345)),
                80,
                90,
                FONT_GOLD,
                false,
            ));
            els.push(Element::text(
                format!("{} (Y/N):", self.resources.langbase.lstr(346)),
                80,
                110,
                FONT_GOLD,
                false,
            ));
            els.push(Element::fillbox(188, 108, 9, 11, 243));
            if self.cursor_blink.visible(11, 10) {
                els.push(Element::fillbox(190, 116, 5, 1, FONT_DEFAULT));
            }
        }

        els
    }

    fn handle_event(&mut self, event: &Event) -> Option<SaveAction> {
        let state = self.state.clone();
        match state {
            SaveDialogState::Browse { selected } => match event {
                Event::Keyboard(Key::Escape) => {
                    self.state = SaveDialogState::Inactive;
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Up) if selected > 0 => {
                    let next = if selected == 4 { 3 } else { selected - 1 };
                    self.state = SaveDialogState::Browse { selected: next };
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Up) => {
                    self.state = SaveDialogState::Browse { selected: 4 };
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Down) if selected < 4 => {
                    let next = if selected == 3 { 4 } else { selected + 1 };
                    self.state = SaveDialogState::Browse { selected: next };
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Down) => {
                    self.state = SaveDialogState::Browse { selected: 0 };
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Enter | Key::Char(' ')) => Some(self.activate_item(selected)),
                Event::Keyboard(Key::Char(c)) => Some(self.handle_browse_digit(*c)),
                _ => Some(SaveAction::Consumed),
            },
            SaveDialogState::EditField { field, mut editor } => match event {
                Event::Keyboard(Key::Escape) => {
                    self.cursor_blink.reset();
                    self.state = SaveDialogState::Browse {
                        selected: field_idx(&field),
                    };
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Enter) => {
                    self.cursor_blink.reset();
                    let buf = editor.buffer().to_string();
                    match field {
                        SaveField::Author => self.author = buf,
                        SaveField::Name => self.name = buf,
                        SaveField::Filename => self.filename = buf,
                    }
                    self.state = SaveDialogState::Browse {
                        selected: field_idx(&field),
                    };
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Backspace) => {
                    if editor.backspace() {
                        self.cursor_blink.reset();
                        self.state = SaveDialogState::EditField { field, editor };
                    }
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Char(c)) => {
                    if editor.insert(*c) {
                        self.cursor_blink.reset();
                        self.state = SaveDialogState::EditField { field, editor };
                    }
                    Some(SaveAction::Consumed)
                }
                _ => Some(SaveAction::Consumed),
            },
            SaveDialogState::ConfirmOverwrite { .. } => match event {
                Event::Keyboard(Key::Escape) => {
                    self.state = SaveDialogState::Browse { selected: 2 };
                    Some(SaveAction::Consumed)
                }
                Event::Keyboard(Key::Char('y' | 'Y')) => Some(SaveAction::SaveReplay),
                Event::Keyboard(Key::Char('n' | 'N')) => {
                    self.state = SaveDialogState::Browse { selected: 2 };
                    Some(SaveAction::Consumed)
                }
                _ => Some(SaveAction::Consumed),
            },
            SaveDialogState::Inactive => Some(SaveAction::Consumed),
        }
    }
}

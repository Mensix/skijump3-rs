use crate::components::modal::alert_prompt;
use crate::gfx::theme::{BG_PURPLE, BG_RED, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::route::RouteTarget;
use crate::store::{GameStateRef, ResourcesRef};
use engine::oxide::input::Key;
use engine::oxide::widgets::menu::{MenuItem, PixelMenu};
use engine::oxide::widgets::text_input::{TextInput, TextInputMessage};
use engine::oxide::Blinker;
use engine::oxide::Widget;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
enum EditMode {
    Viewing,
    Editing {
        field: usize,
        input: TextInput,
    },
    Alert {
        message: String,
        subtitle: String,
        old_value: String,
        field: usize,
    },
    ConfirmOverwrite {
        filename: String,
    },
}

#[derive(Deserialize, Serialize)]
struct CustomHillCatalogToml {
    id: String,
    name: String,
    hills: Vec<CustomHillToml>,
}

#[derive(Deserialize, Serialize)]
struct CustomHillToml {
    id: String,
    name: String,
    terrain_index: String,
    kr: i64,
    front_index: String,
    back_index: String,
    back_brightness: i64,
    back_mirror: bool,
    vx_final: i64,
    pk_hundred: i64,
    pl_save_ten_thousand: i64,
    author: String,
    checksum: i64,
    profile_checksum: i64,
}

pub struct EditHillView {
    resources: ResourcesRef,
    store: GameStateRef,
    menu: PixelMenu,
    mode: EditMode,
    values: [String; 12],
    initial_values: [String; 12],
    blinker: Blinker,
}

impl EditHillView {
    pub fn new(resources: ResourcesRef, store: GameStateRef) -> Self {
        let items = (1..=12).map(|n| MenuItem::new(n, "")).collect();
        let menu = PixelMenu::new(10, 8, 110, 13, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .trailing("", 13);
        let values = [
            "Default".into(),
            "120".into(),
            "1".into(),
            "0".into(),
            "100".into(),
            "0".into(),
            "100".into(),
            "100".into(),
            "6".into(),
            "Unknown".into(),
            "NEW1".into(),
            "".into(),
        ];
        let values = store
            .borrow_mut()
            .nav_edit_hill
            .take()
            .and_then(|filename| Self::load_values(&resources, &filename))
            .unwrap_or(values);
        let initial_values = values.clone();
        Self {
            resources,
            store,
            menu,
            mode: EditMode::Viewing,
            values,
            initial_values,
            blinker: Blinker::new(),
        }
    }

    fn load_values(resources: &ResourcesRef, filename: &str) -> Option<[String; 12]> {
        let path = format!("custom_hills/{filename}.toml");
        let data = resources.files.read(&path).ok()?;
        let text = std::str::from_utf8(&data).ok()?;
        let catalog = toml::from_str::<CustomHillCatalogToml>(text).ok()?;
        let hill = catalog.hills.first()?;
        Some([
            hill.name.clone(),
            hill.kr.to_string(),
            hill.front_index.clone(),
            hill.back_index.clone(),
            hill.back_brightness.to_string(),
            (if hill.back_mirror { 1 } else { 0 }).to_string(),
            (hill.vx_final - 40).to_string(),
            hill.pk_hundred.to_string(),
            (hill.pl_save_ten_thousand - 3204).to_string(),
            hill.author.clone(),
            filename.to_string(),
            String::new(),
        ])
    }

    fn has_changes(&self) -> bool {
        self.values != self.initial_values
    }

    fn build_custom_hill_toml(&self) -> CustomHillCatalogToml {
        let filename = &self.values[10];
        CustomHillCatalogToml {
            id: filename.clone(),
            name: format!("{} custom hill", filename),
            hills: vec![CustomHillToml {
                id: filename.clone(),
                name: self.values[0].clone(),
                terrain_index: self.values[2].clone(),
                kr: self.values[1].parse().unwrap_or(120),
                front_index: self.values[2].clone(),
                back_index: self.values[3].clone(),
                back_brightness: self.values[4].parse().unwrap_or(100),
                back_mirror: self.values[5].parse::<i64>().unwrap_or(0) != 0,
                vx_final: self.values[6].parse::<i64>().unwrap_or(100) + 40,
                pk_hundred: self.values[7].parse().unwrap_or(100),
                pl_save_ten_thousand: self.values[8].parse::<i64>().unwrap_or(6) + 3204,
                author: self.values[9].clone(),
                checksum: 0,
                profile_checksum: 0,
            }],
        }
    }

    fn save(&self) {
        let toml = self.build_custom_hill_toml();
        if let Ok(data) = toml::to_string(&toml) {
            let filename = &self.values[10];
            let path = format!("custom_hills/{filename}.toml");
            let _ = self.resources.files.write(&path, data.as_bytes());
        }
    }

    fn start_edit(&mut self, field: usize) {
        let yy = 10 + ((field - 1) * 13) as i32;
        let value = self.values[field - 1].clone();
        let (max_width, max_chars) = match field {
            1 | 10 => (150, 130),
            3 | 4 => (20, 3),
            2 | 5 | 6 | 7 | 8 | 9 => (30, 10),
            11 => (62, 8),
            _ => (150, 130),
        };
        let input = TextInput::new(
            120,
            yy,
            max_width,
            value,
            max_chars,
            BLACK,
            FONT_GOLD,
            FONT_BODY,
            self.resources.font.clone(),
        );
        self.mode = EditMode::Editing { field, input };
    }

    fn commit_edit(&mut self, mut value: String) {
        let field = match &self.mode {
            EditMode::Editing { field, .. } => *field,
            _ => return,
        };
        match field {
            2 => self.validate_or_keep(field, &mut value, 40, 300),
            3 | 4 => {
                if value.is_empty() || !value.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return;
                }
                let prefix = if field == 3 { "front" } else { "back" };
                let path = format!("hills/generated/HILL{value}/{prefix}_visual.png");
                if self.resources.files.read(&path).is_err() {
                    let label = if field == 3 { "FRONT" } else { "BACK" };
                    let old = self.values[field - 1].clone();
                    self.mode = EditMode::Alert {
                        field,
                        old_value: old,
                        message: format!("INVALID {label} INDEX VALUE."),
                        subtitle: format!("FILE {label}{value}.PNG DOESN'T EXIST."),
                    };
                    return;
                }
            }
            5 => self.validate_or_keep(field, &mut value, 0, 255),
            6 => self.validate_or_keep(field, &mut value, 0, 1),
            7 => self.validate_or_keep(field, &mut value, 60, 145),
            8 => self.validate_or_keep(field, &mut value, 50, 150),
            9 => self.validate_or_keep(field, &mut value, 0, 30),
            11 => value.truncate(8),
            _ => {}
        }
        self.values[field - 1] = value;
        self.mode = EditMode::Viewing;
    }

    fn validate_or_keep(&self, field: usize, value: &mut String, low: i32, high: i32) {
        if !Self::validate_int(value, low, high) {
            *value = self.values[field - 1].clone();
        }
    }

    fn validate_int(value: &mut String, low: i32, high: i32) -> bool {
        if let Ok(n) = value.trim().parse::<i32>() {
            *value = n.clamp(low, high).to_string();
            true
        } else {
            false
        }
    }

    fn cancel_edit(&mut self) {
        self.mode = EditMode::Viewing;
    }

    fn is_numeric_field(field: usize) -> bool {
        matches!(field, 2 | 5 | 6 | 7 | 8 | 9)
    }

    fn is_valid_filename(s: &str) -> bool {
        if s.is_empty() || s.len() > 8 {
            return false;
        }
        s.chars().all(|c| c.is_ascii_alphanumeric())
    }
}

impl Screen<RouteTarget> for EditHillView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match &mut self.mode {
            EditMode::ConfirmOverwrite { .. } => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    if matches!(event, UiEvent::Text('Y' | 'y')) {
                        self.save();
                        cx.back();
                    } else {
                        self.mode = EditMode::Viewing;
                        self.menu.set_selected(10); // back to FILENAME field
                    }
                }
                cx.consume();
                return;
            }
            EditMode::Alert {
                field, old_value, ..
            } => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    self.values[*field - 1] = old_value.clone();
                    self.mode = EditMode::Viewing;
                }
                cx.consume();
                return;
            }
            EditMode::Editing { field, input } => {
                let event = if Self::is_numeric_field(*field) {
                    match event {
                        UiEvent::Text(c) if !c.is_ascii_digit() => return,
                        _ => event,
                    }
                } else {
                    event
                };
                let mut ecx = engine::oxide::widget::EventCx::default();
                match input.event(&mut ecx, event) {
                    Some(TextInputMessage::Commit(value)) => {
                        self.commit_edit(value);
                    }
                    Some(TextInputMessage::Cancel) => {
                        self.cancel_edit();
                    }
                    None => {}
                }
                if ecx.is_consumed() {
                    cx.consume();
                }
                return;
            }
            EditMode::Viewing => {}
        }

        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(0) => {
                if !self.has_changes() {
                    cx.back();
                } else {
                    let filename = &self.values[10];
                    if !Self::is_valid_filename(filename) {
                        self.mode = EditMode::Alert {
                            field: 11,
                            old_value: self.values[10].clone(),
                            message: "INVALID FILENAME.".to_string(),
                            subtitle: "ENTER 1-8 ALPHANUMERIC CHARACTERS.".to_string(),
                        };
                        return;
                    }
                    let path = format!("custom_hills/{filename}.toml");
                    if self.resources.files.read(&path).is_ok() {
                        self.mode = EditMode::ConfirmOverwrite {
                            filename: filename.clone(),
                        };
                    } else {
                        self.save();
                        cx.back();
                    }
                }
            }
            Some(12) => {
                self.values = self.initial_values.clone();
                self.store.borrow_mut().nav_edit_hill = None;
                cx.back();
            }
            Some(n @ 1..=11) => {
                if matches!(event, UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ')) {
                    self.start_edit(n);
                }
            }
            _ => {}
        }
        if ecx.is_consumed() {
            cx.consume();
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let xx = 15i32;
        let xx2 = 120i32;

        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 320, 200), BG_PURPLE);

        let labels = [
            "HILL.NAME",
            "K.POINT",
            "FRONT INDEX",
            "BACK INDEX",
            "BACK BRIGHT",
            "MIRROR BACK",
            "VX.FINAL",
            "ADJUST.K",
            "AIR.P.PLUS",
            "AUTHOR",
            "FILENAME",
            "EXIT, DON'T SAVE",
        ];

        let descriptions = [
            "",
            "CRITICAL POINT (40-300\u{00b5})",
            "FRONT*.PCX (000-ZZZ)",
            "BACK*.PCX (000-ZZZ)",
            "PERCENTS OF ORIGINAL (0-255)",
            "0-NO, 1-MIRROR (0-1)",
            "TAKE-OFF SPEED (60-145 KM/H)",
            "MOVE K-POINT ON HILL (50-150)",
            "+ 990 MBAR (0-30)",
            "",
            "     8 CHARS MAX",
            "",
        ];

        for temp in 1..=12 {
            let yy = 10 + ((temp - 1) * 13) as i32;
            let label = if temp < 12 {
                format!("{}. {}:", temp, labels[temp - 1])
            } else {
                format!("{}. {}", temp, labels[temp - 1])
            };

            cx.text((xx, yy), FONT_BODY, &label);

            if !descriptions[temp - 1].is_empty() {
                cx.text((xx2 + 40, yy), FONT_GRAY, descriptions[temp - 1]);
            }

            if !self.values[temp - 1].is_empty() {
                cx.text((xx2, yy), FONT_GOLD, &self.values[temp - 1]);
            }
        }

        cx.text((xx, 179), FONT_GOLD, "0. EXIT and SAVE");

        if matches!(self.mode, EditMode::Viewing | EditMode::Editing { .. }) {
            self.menu.paint(cx);
        }

        match &self.mode {
            EditMode::Editing { input, .. } => {
                input.paint(cx);
            }
            EditMode::Alert {
                ref message,
                ref subtitle,
                ..
            } => {
                let prompt = self.resources.langbase.lstr(15);
                alert_prompt(
                    cx,
                    BG_RED,
                    message,
                    format!("{subtitle}  {prompt}"),
                    191,
                    self.blinker.visible(11, 10),
                );
            }
            EditMode::ConfirmOverwrite { filename } => {
                let prompt = self.resources.langbase.lstr(346);
                alert_prompt(
                    cx,
                    BG_RED,
                    format!("FILE {filename}.TOML ALREADY EXISTS."),
                    format!("{} (Y/N):", prompt),
                    191,
                    self.blinker.visible(11, 10),
                );
            }
            EditMode::Viewing => {}
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

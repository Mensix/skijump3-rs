use crate::components::modal::alert_prompt;
use crate::gfx::theme::{BG_PURPLE, BLACK, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use engine::oxide::input::Key;
use engine::oxide::widgets::menu::{MenuItem, PixelMenu};
use engine::oxide::widgets::text_input::{TextInput, TextInputMessage};
use engine::oxide::Widget;
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx, UiEvent};
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
    menu: PixelMenu,
    mode: EditMode,
    values: [String; 12],
    initial_values: [String; 12],
}

impl EditHillView {
    pub fn new(resources: ResourcesRef, initial_filename: Option<String>) -> Self {
        let items = (1..=12).map(|n| MenuItem::new(n, "")).collect();
        let menu = PixelMenu::new(10, 8, 110, 13, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .trailing("", 13);
        let default_values = [
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
        let values = initial_filename
            .and_then(|filename| Self::load_values(&resources, &filename))
            .unwrap_or(default_values);
        let initial_values = values.clone();
        Self {
            resources,
            menu,
            mode: EditMode::Viewing,
            values,
            initial_values,
        }
    }

    fn load_values(resources: &ResourcesRef, filename: &str) -> Option<[String; 12]> {
        let path = format!("custom_hills/{filename}.toml");
        let data = resources.files.read(&path);
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
            self.resources.files.write(&path, data.as_bytes());
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
                if self.resources.files.read(&path).is_empty() {
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

impl GameScreen for EditHillView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match &mut self.mode {
            EditMode::ConfirmOverwrite { .. } => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    if matches!(event, UiEvent::Text('Y' | 'y')) {
                        self.save();
                        nav.back();
                    } else {
                        self.mode = EditMode::Viewing;
                        self.menu.set_selected(10);
                    }
                }
                nav.consume();
                return;
            }
            EditMode::Alert {
                field, old_value, ..
            } => {
                if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                    self.values[*field - 1] = old_value.clone();
                    self.mode = EditMode::Viewing;
                }
                nav.consume();
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
                    nav.consume();
                }
                return;
            }
            EditMode::Viewing => {}
        }

        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(0) => {
                if !self.has_changes() {
                    nav.back();
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
                    if !self.resources.files.read(&path).is_empty() {
                        self.mode = EditMode::ConfirmOverwrite {
                            filename: filename.clone(),
                        };
                    } else {
                        self.save();
                        nav.back();
                    }
                }
            }
            Some(12) => {
                self.values = self.initial_values.clone();
                cx.state.nav_edit_hill = None;
                nav.back();
            }
            Some(n @ 1..=11) => {
                if matches!(event, UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ')) {
                    self.start_edit(n);
                }
            }
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        let lang = &self.resources.langbase;
        let xx = 15i32;
        let xx2 = 120i32;

        paint.fill((0, 0, 320, 200), BLACK);
        paint.pattern_fill((0, 0, 320, 200), BG_PURPLE);

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

        for temp in 0..12 {
            let yy = 10 + (temp * 13) as i32;
            let label = if temp < 11 {
                format!("{}. {}:", temp + 1, labels[temp])
            } else {
                format!("{}. {}", temp + 1, labels[temp])
            };

            paint.text((xx, yy), FONT_BODY, &label);

            if !descriptions[temp].is_empty() {
                paint.text((xx2 + 40, yy), FONT_GRAY, descriptions[temp]);
            }

            if !self.values[temp].is_empty() {
                paint.text((xx2, yy), FONT_GOLD, &self.values[temp]);
            }
        }

        paint.text((xx, 179), FONT_GOLD, format!("0. {}", lang.tr(296)));

        if matches!(self.mode, EditMode::Viewing | EditMode::Editing { .. }) {
            self.menu.paint(paint);
        }

        match &self.mode {
            EditMode::Editing { input, .. } => {
                input.paint(paint);
            }
            EditMode::Alert {
                ref message,
                ref subtitle,
                ..
            } => {
                let prompt = lang.tr(15);
                alert_prompt(paint, message, format!("{subtitle}  {prompt}"), false);
            }
            EditMode::ConfirmOverwrite { filename } => {
                let prompt = lang.tr(346);
                alert_prompt(
                    paint,
                    format!("FILE {filename}.TOML ALREADY EXISTS."),
                    prompt,
                    true,
                );
            }
            EditMode::Viewing => {}
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

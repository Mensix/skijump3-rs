use crate::gfx::theme::{BG_LEFT, BG_RIGHT, BLACK, FONT_DEFAULT, FONT_GOLD, FONT_HELP};
use engine::oxide::Blinker;
use crate::route::RouteTarget;
use crate::store::ResourcesRef;
use engine::oxide::input::Key;
use engine::oxide::widgets::menu::{MenuItem, PixelMenu};
use engine::oxide::widgets::text_input::{TextInput, TextInputMessage};
use engine::oxide::Widget;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};

#[derive(Debug)]
enum EditMode {
    Viewing,
    Editing { field: usize, input: TextInput },
    Alert { message: String, subtitle: String, old_value: String, field: usize },
}

pub struct EditHillView {
    resources: ResourcesRef,
    menu: PixelMenu,
    mode: EditMode,
    values: [String; 12],
    blinker: Blinker,
}

impl EditHillView {
    pub fn new(resources: ResourcesRef) -> Self {
        let items = (1..=12)
            .map(|n| MenuItem::new(n, ""))
            .collect();
        let menu = PixelMenu::new(10, 8, 110, 13, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_exit("", 13);
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
        Self {
            resources,
            menu,
            mode: EditMode::Viewing,
            values,
            blinker: Blinker::new(),
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
            FONT_DEFAULT,
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
            2 => Self::validate_int(&mut value, 40, 300),
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
            5 => Self::validate_int(&mut value, 0, 255),
            6 => Self::validate_int(&mut value, 0, 1),
            7 => Self::validate_int(&mut value, 60, 145),
            8 => Self::validate_int(&mut value, 50, 150),
            9 => Self::validate_int(&mut value, 0, 30),
            11 => value.truncate(8),
            _ => {}
        }
        self.values[field - 1] = value;
        self.mode = EditMode::Viewing;
    }

    fn validate_int(value: &mut String, low: i32, high: i32) {
        match value.trim().parse::<i32>() {
            Ok(n) => {
                *value = n.clamp(low, high).to_string();
            }
            Err(_) => {}
        }
    }

    fn cancel_edit(&mut self) {
        self.mode = EditMode::Viewing;
    }

    fn is_numeric_field(field: usize) -> bool {
        matches!(field, 2 | 5 | 6 | 7 | 8 | 9)
    }
}

impl Screen<RouteTarget> for EditHillView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match &mut self.mode {
            EditMode::Alert { field, old_value, .. } => {
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
            Some(0) => cx.navigate(RouteTarget::Back),
            Some(12) => cx.navigate(RouteTarget::Back),
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

    fn paint(&self, cx: &mut PaintCx<'_>) {
        let xx = 15i32;
        let xx2 = 120i32;

        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 320, 200), BG_LEFT);

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

            cx.text((xx, yy), FONT_DEFAULT, &label);

            if !descriptions[temp - 1].is_empty() {
                cx.text((xx2 + 40, yy), FONT_HELP, descriptions[temp - 1]);
            }

            if !self.values[temp - 1].is_empty() {
                cx.text((xx2, yy), FONT_GOLD, &self.values[temp - 1]);
            }
        }

        cx.text((xx, 179), FONT_GOLD, "0. EXIT and SAVE");

        match &self.mode {
            EditMode::Editing { input, .. } => {
                input.paint(cx);
            }
            EditMode::Alert { ref message, ref subtitle, .. } => {
                cx.fill((59, 79, 203, 54), BLACK);
                cx.fill((60, 80, 201, 52), BG_RIGHT);
                cx.pattern_fill((60, 80, 201, 52), BG_RIGHT);
                cx.text((80, 90), FONT_GOLD, message);
                cx.text((80, 100), FONT_GOLD, subtitle);
                cx.text((190, 110), FONT_DEFAULT, self.resources.langbase.lstr(15));
                if self.blinker.visible(11, 10) {
                    cx.fill((191, 110, 5, 7), FONT_DEFAULT);
                }
            }
            EditMode::Viewing => {}
        }

        self.menu.paint(cx);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

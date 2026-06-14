use crate::gfx::theme::{BG_LEFT, BLACK, FONT_DEFAULT, FONT_GOLD, FONT_HELP};
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
    Editing(TextInput),
}

pub struct EditHillView {
    resources: ResourcesRef,
    menu: PixelMenu,
    mode: EditMode,
    values: [String; 12],
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
        }
    }

    fn start_edit(&mut self, field: usize) {
        let yy = 10 + ((field - 1) * 13) as i32;
        let value = self.values[field - 1].clone();
        let input = TextInput::new(
            120,
            yy,
            150,
            value,
            130,
            BLACK,
            FONT_GOLD,
            FONT_DEFAULT,
            self.resources.font.clone(),
        );
        self.mode = EditMode::Editing(input);
    }

    fn commit_edit(&mut self, mut value: String) {
        let field = self.menu.selected() + 1;
        match field {
            2 => Self::validate_int(&mut value, 40, 300),
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
}

impl Screen<RouteTarget> for EditHillView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match &mut self.mode {
            EditMode::Editing(input) => {
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
            EditMode::Editing(input) => {
                input.paint(cx);
            }
            EditMode::Viewing => {}
        }

        self.menu.paint(cx);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

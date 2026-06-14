use crate::gfx::theme::{BG_LEFT, BLACK, FONT_DEFAULT, FONT_GOLD, FONT_HELP};
use crate::route::RouteTarget;
use crate::store::ResourcesRef;
use engine::oxide::widgets::menu::{MenuItem, PixelMenu};
use engine::oxide::Widget;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};

pub struct EditHillView {
    resources: ResourcesRef,
    menu: PixelMenu,
}

impl EditHillView {
    pub fn new(resources: ResourcesRef) -> Self {
        let items = (1..=12)
            .map(|n| MenuItem::new(n, ""))
            .collect();
        let menu = PixelMenu::new(10, 8, 110, 13, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false);
        Self { resources, menu }
    }
}

impl Screen<RouteTarget> for EditHillView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(0) | Some(12) => cx.navigate(RouteTarget::Back),
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

        let values = [
            "Default",
            "120",
            "1",
            "0",
            "100",
            "0",
            "100",
            "100",
            "6",
            "Unknown",
            "NEW1",
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

            if !values[temp - 1].is_empty() {
                cx.text((xx2, yy), FONT_GOLD, values[temp - 1]);
            }
        }

        cx.text((xx, 179), FONT_GOLD, "0. EXIT and SAVE");

        self.menu.paint(cx);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

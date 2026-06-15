use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_TEAL};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use engine::oxide::widgets::menu::MenuItem as OxideMenuItem;
use engine::oxide::widgets::menu::PixelMenu;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent, Widget};

pub struct WelcomeScreenView {
    menu: PixelMenu,
    languages: Vec<String>,
    save_manager: SaveRef,
}

impl WelcomeScreenView {
    #[must_use]
    pub fn new(languages: Vec<String>, save_manager: SaveRef) -> Self {
        let count = languages.len();
        let items: Vec<OxideMenuItem> = (0..count)
            .map(|i| OxideMenuItem::new((i + 1) as u8, format!("{}", i)))
            .collect();
        Self {
            menu: PixelMenu::new(112, 64, 100, 8, items, FONT_BODY, FONT_BODY)
                .with_labels(false)
                .with_box(false),
            languages,
            save_manager,
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 51, 200), FILL_GRAY);
        cx.pattern_fill((52, 0, 216, 200), BG_PURPLE);
        cx.pattern_fill((269, 0, 51, 200), FILL_GRAY);
        cx.sprite(sprites::Sprite::Logo as u16, (80, 6));
        cx.right_text((240, 6), FONT_BODY, "WELCOME!");
        cx.right_text((240, 16), FONT_GOLD, "TERVETULOA!");
        cx.right_text((240, 26), FONT_TEAL, "WILLKOMMEN!");
        cx.right_text((240, 36), FONT_BODY, "VALKOMMEN!");
        cx.text((100, 50), FONT_BODY, "PLEASE CHOOSE A LANGUAGE:");

        for (i, name) in self.languages.iter().enumerate() {
            let y = ((i + 1) * 8 + 55) as i32;
            cx.center_text((155, y), FONT_GOLD, name);
        }

        let y = 61 + (self.menu.selected() as i32) * 8;
        cx.stroke((106, y, 101, 9), FONT_BODY);
    }
}

impl Screen<RouteTarget> for WelcomeScreenView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(0) => cx.navigate(RouteTarget::MainMenu),
            Some(n) => {
                self.save_manager.set_language(n - 1);
                cx.navigate(RouteTarget::MainMenu);
            }
            _ => {}
        }
        if ecx.is_consumed() {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

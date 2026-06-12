use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{BG_LEFT, BLACK, FILL_DIM, FONT_DEFAULT, FONT_GOLD, FONT_GREET};
use crate::gfx::sprites;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::text::lang::LangBase;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Component, Event, Key};
use std::rc::Rc;

pub struct WelcomeScreenView {
    menu: Menu,
    languages: Vec<String>,
    save_manager: SaveRef,
}

impl WelcomeScreenView {
    #[must_use]
    pub fn new(languages: Vec<String>, langbase: &Rc<LangBase>, save_manager: SaveRef) -> Self {
        let count = languages.len();
        let mut items = Vec::with_capacity(count);
        for (i, _) in languages.iter().enumerate() {
            items.push(MenuItem::new((i + 1) as u8, 0));
        }
        Self {
            menu: Menu::new(112, 64, 100, 8, items, langbase, FONT_DEFAULT, FONT_DEFAULT)
                .with_labels(false),
            languages,
            save_manager,
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.fill((0, 0, 51, 200), FILL_DIM);
        cx.fill((52, 0, 216, 200), BG_LEFT);
        cx.fill((269, 0, 51, 200), FILL_DIM);
        cx.dither_fill(63);
        cx.sprite(sprites::Sprite::Logo as u16, (80, 6));
        cx.right_text((240, 6), FONT_DEFAULT, "WELCOME!");
        cx.right_text((240, 16), FONT_GOLD, "TERVETULOA!");
        cx.right_text((240, 26), FONT_GREET, "WILLKOMMEN!");
        cx.right_text((240, 36), FONT_DEFAULT, "VALKOMMEN!");
        cx.text((100, 50), FONT_DEFAULT, "PLEASE CHOOSE A LANGUAGE:");

        for (i, name) in self.languages.iter().enumerate() {
            let y = ((i + 1) * 8 + 55) as i32;
            cx.center_text((155, y), FONT_GOLD, name);
        }

        let y = 61 + (self.menu.selected() as i32) * 8;
        cx.stroke((106, y, 101, 9), FONT_DEFAULT);
    }
}

impl Screen<RouteTarget> for WelcomeScreenView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_event(event) else {
            return;
        };
        match self.menu.handle_event(&event) {
            Some(0) => cx.navigate(RouteTarget::MainMenu),
            Some(n) => {
                self.save_manager.set_language(n - 1);
                cx.navigate(RouteTarget::MainMenu);
            }
            _ => {}
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

fn input_event(event: UiEvent) -> Option<Event> {
    match event {
        UiEvent::KeyDown(key) => Some(Event::Keyboard(key)),
        UiEvent::Text(ch) => Some(Event::Keyboard(Key::Char(ch))),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

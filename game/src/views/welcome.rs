use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{BG_LEFT, BLACK, FILL_DIM, FONT_DEFAULT, FONT_GOLD, FONT_GREET};
use crate::gfx::sprites;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::text::lang::LangBase;
use engine::oxide::legacy::{event_from_ui, paint_elements};
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Component, Element};
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

    fn legacy_elements(&self) -> Vec<Element> {
        let mut els = vec![
            Element::fillbox(0, 0, 320, 200, BLACK),
            Element::fillbox(0, 0, 51, 200, FILL_DIM),
            Element::fillbox(52, 0, 216, 200, BG_LEFT),
            Element::fillbox(269, 0, 51, 200, FILL_DIM),
            Element::fill_area(63),
            Element::sprite(sprites::Sprite::Logo as u16, 80, 6),
            Element::right_text("WELCOME!", 240, 6, FONT_DEFAULT),
            Element::right_text("TERVETULOA!", 240, 16, FONT_GOLD),
            Element::right_text("WILLKOMMEN!", 240, 26, FONT_GREET),
            Element::right_text("VALKOMMEN!", 240, 36, FONT_DEFAULT),
            Element::text("PLEASE CHOOSE A LANGUAGE:", 100, 50, FONT_DEFAULT, false),
        ];

        for (i, name) in self.languages.iter().enumerate() {
            let y = ((i + 1) * 8 + 55) as i32;
            els.push(Element::center_text(name.clone(), 155, y, FONT_GOLD));
        }

        els.extend(self.menu.elements());
        els
    }
}

impl Screen<RouteTarget> for WelcomeScreenView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = event_from_ui(event) else {
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
        paint_elements(cx, &self.legacy_elements());
    }
}

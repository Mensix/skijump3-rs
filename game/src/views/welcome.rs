use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{apply_logo_tint, FONT_DEFAULT, FONT_GOLD, FONT_GREET};
use crate::gfx::sprites;
use crate::parsers::langbase::LangBase;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use engine::ui::{Component, Element, Event, View};
use std::rc::Rc;

pub struct WelcomeScreenView {
    menu: Menu,
    languages: Vec<String>,
    save_manager: SaveRef,
}

impl WelcomeScreenView {
    #[must_use]
    pub fn new(languages: Vec<String>, langbase: Rc<LangBase>, save_manager: SaveRef) -> Self {
        let count = languages.len();
        let mut items = Vec::with_capacity(count);
        for (i, _) in languages.iter().enumerate() {
            items.push(MenuItem::new((i + 1) as u8, 0));
        }
        Self {
            menu: Menu::new(
                112,
                64,
                100,
                8,
                items,
                &langbase,
                FONT_DEFAULT,
                FONT_DEFAULT,
            )
            .with_labels(false),
            languages,
            save_manager,
        }
    }
}

impl View<RouteTarget> for WelcomeScreenView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![
            Element::fillbox(0, 0, 320, 200, 0),
            Element::fillbox(0, 0, 51, 200, 245),
            Element::fillbox(52, 0, 216, 200, 243),
            Element::fillbox(269, 0, 51, 200, 245),
            Element::fill_area(63),
            Element::sprite(sprites::Sprite::Logo as u16, 80, 6),
            Element::right_text("WELCOME!", 240, 6, FONT_DEFAULT),
            Element::right_text("TERVETULOA!", 240, 16, FONT_GOLD),
            Element::right_text("WILLKOMMEN!", 240, 26, FONT_GREET),
            Element::right_text("VALKOMMEN!", 240, 36, FONT_DEFAULT),
            Element::text("PLEASE CHOOSE A LANGUAGE:", 100, 50, FONT_DEFAULT, false),
        ];

        // language names centred at x=155, y=temp*8+55
        for (i, name) in self.languages.iter().enumerate() {
            let iy = ((i + 1) * 8 + 55) as i32;
            els.push(Element::center_text(name.clone(), 155, iy, FONT_GOLD));
        }

        // highlight box from Menu component (labels disabled)
        els.extend(self.menu.elements());

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(0) => Some(RouteTarget::MainMenu),
            Some(n) => {
                self.save_manager.set_language(n - 1);
                Some(RouteTarget::MainMenu)
            }
            _ => None,
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        apply_logo_tint(palette, 0);
    }
}

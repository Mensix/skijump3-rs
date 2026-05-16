use crate::components::menu::{Menu, MenuItem};
use crate::palette_consts::{FONT_DEFAULT, FONT_GOLD, FONT_GREET, apply_logo_tint};
use crate::parsers::langbase::LangBase;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::sprites;
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
            items.push(MenuItem {
                num: (i + 1) as u8,
                label: 0,
                y_off: 0,
            });
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
            Element::FillArea { thing: 63 },
            Element::sprite(sprites::LOGO_SPRITE, 80, 6),
            Element::text_color_right("WELCOME!", 240, 6, FONT_DEFAULT),
            Element::text_color_right("TERVETULOA!", 240, 16, FONT_GOLD),
            Element::text_color_right("WILLKOMMEN!", 240, 26, FONT_GREET),
            Element::text_color_right("VALKOMMEN!", 240, 36, FONT_DEFAULT),
            Element::text_color("PLEASE CHOOSE A LANGUAGE:", 100, 50, FONT_DEFAULT),
        ];

        // language names centred at x=155, y=temp*8+55
        for (i, name) in self.languages.iter().enumerate() {
            let iy = ((i + 1) * 8 + 55) as i32;
            els.push(Element::text_color_center(name, 155, iy, FONT_GOLD));
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

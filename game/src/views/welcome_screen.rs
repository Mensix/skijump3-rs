use std::sync::Arc;
use engine::ui::{Element, Event, View, Component};
use crate::parsers::langbase::LangBase;
use crate::components::menu::{Menu, MenuItem};
use crate::route::RouteTarget;

const FONT_DEFAULT: u8 = 240;
const FONT_GOLD: u8 = 246;
const FONT_GREET: u8 = 247;
const BOX_COLOR: u8 = 240;

pub struct WelcomeScreenView {
    menu: Menu,
    languages: Vec<String>,
}

impl WelcomeScreenView {
    pub fn new(languages: Vec<String>, langbase: Arc<LangBase>) -> Self {
        let count = languages.len();
        let mut items = Vec::with_capacity(count);
        for (i, _) in languages.iter().enumerate() {
            items.push(MenuItem { num: (i + 1) as u8, label: 0, y_off: 0 });
        }
        // phase=3: navigable = count-1, exit slot wraps to last language
        let navigable = if count > 1 { count - 1 } else { 1 };
        Self {
            menu: Menu::new(112, 64, 100, 8, items, &langbase, FONT_DEFAULT, BOX_COLOR)
                .with_navigable(navigable)
                .with_labels(false),
            languages,
        }
    }
}

impl View<RouteTarget> for WelcomeScreenView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        // NewScreen(6,0): coloured panels
        els.push(Element::fillbox(0, 0, 50, 200, 245));
        els.push(Element::fillbox(52, 0, 267, 200, 243));
        els.push(Element::fillbox(269, 0, 319, 200, 245));

        // logo sprite 61 at (80,6)
        els.push(Element::sprite(61, 80, 6));

        // welcome text, ewritefont (right-aligned) at x=240
        els.push(Element::text_color_right("WELCOME!", 240, 6, FONT_DEFAULT));
        els.push(Element::text_color_right("TERVETULOA!", 240, 16, FONT_GOLD));
        els.push(Element::text_color_right("WILLKOMMEN!", 240, 26, FONT_GREET));
        els.push(Element::text_color_right("VALKOMMEN!", 240, 36, FONT_DEFAULT));

        // instruction
        els.push(Element::text_color("PLEASE CHOOSE A LANGUAGE:", 100, 50, FONT_DEFAULT));

        // language names centred at x=155, y=temp*8+55
        for (i, name) in self.languages.iter().enumerate() {
            let iy = ((i + 1) * 8 + 55) as i32;
            els.push(Element::text_color(name, 155, iy, FONT_GOLD));
        }

        // highlight box from Menu component (labels disabled)
        els.extend(self.menu.elements());

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self.menu.handle_event(&event) {
            Some(0) => {
                // wrap: index=0 selects last language
                Some(RouteTarget::MainMenu)
            }
            Some(_n) => {
                // language n selected (1-indexed)
                Some(RouteTarget::MainMenu)
            }
            _ => None,
        }
    }
}

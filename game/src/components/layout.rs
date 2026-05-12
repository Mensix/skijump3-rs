use std::rc::Rc;
use engine::ui::Element;
use crate::parsers::langbase::LangBase;
use engine::consts::{WIDTH, HEIGHT};

const FONT_DEFAULT: u8 = 240;

pub fn header_elements(text: &str, x: i32, y: i32, color: u8, bg: u8) -> Vec<Element> {
    vec![
        Element::fillbox(x, y, 100, 6, bg),
        Element::text_color(text, x, y, color),
    ]
}

#[derive(Clone)]
pub struct MainLayout {
    pub langbase: Rc<LangBase>,
    version: String,
    background: Vec<u8>,
}

impl MainLayout {
    pub fn new(langbase: Rc<LangBase>, version: String, background: Vec<u8>) -> Self {
        Self { langbase, version, background }
    }

    pub fn wrap(&self, content: Vec<Element>) -> Vec<Element> {
        let mut els = vec![
            Element::image(self.background.clone(), WIDTH, HEIGHT),
            Element::text_color(self.langbase.lstr(34), 170, 51, FONT_DEFAULT),
        ];
        els.extend(content);
        els.push(Element::text_color_right("SKI JUMP", 308, 6, FONT_DEFAULT));
        els.push(Element::text_color_right("INTERNATIONAL", 308, 18, FONT_DEFAULT));
        els.push(Element::text_color(format!("v{}", self.version), 245, 30, FONT_DEFAULT));
        els
    }
}

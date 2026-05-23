use crate::gfx::palette::{FONT_DEFAULT, FONT_HELP, FONT_NEW};
use crate::parsers::langbase::LangBase;
use crate::store::StoreRef;
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::Element;
use std::rc::Rc;

#[must_use]
pub fn header_elements(text: &str, x: i32, y: i32, color: u8, bg: u8) -> Vec<Element> {
    vec![
        Element::fillbox(x, y, 100, 6, bg),
        Element::text(text, x, y, color, false),
    ]
}

#[derive(Clone)]
pub struct MainLayout {
    pub langbase: Rc<LangBase>,
    version: String,
    background: Rc<[u8]>,
    store: StoreRef,
}

impl MainLayout {
    pub fn new(
        langbase: Rc<LangBase>,
        version: String,
        background: Vec<u8>,
        store: StoreRef,
    ) -> Self {
        Self {
            langbase,
            version,
            background: background.into(),
            store,
        }
    }

    #[must_use]
    pub fn background(&self) -> Vec<Element> {
        vec![
            self.background_element(),
            Element::text(self.langbase.lstr(34), 170, 51, FONT_DEFAULT, false),
        ]
    }

    #[must_use]
    pub fn background_element(&self) -> Element {
        Element::image(Rc::clone(&self.background), WIDTH, HEIGHT)
    }

    #[must_use]
    pub fn jumpers(&self) -> Vec<Element> {
        let mut els = Vec::new();
        let pb = self.store.profiles.borrow();
        for (i, &profile_idx) in pb.active_order.iter().enumerate() {
            if profile_idx >= pb.profiles.len() {
                continue;
            }
            let profile = &pb.profiles[profile_idx];
            let y = (i as i32) * 9 + 64;
            els.push(Element::right_text(
                crate::text::format::ordinal_dot(i + 1),
                162,
                y,
                FONT_HELP,
            ));
            els.push(Element::text(&profile.name, 170, y, FONT_HELP, false));
        }
        els
    }

    #[must_use]
    pub fn registration(&self) -> Vec<Element> {
        vec![
            Element::fillbox(128, 155, 185, 1, 9),
            Element::text(
                format!("{} {}", self.langbase.lstr(35), self.langbase.lstr(36)),
                132,
                163,
                FONT_DEFAULT,
                false,
            ),
            Element::fillbox(132, 175, 177, 22, 248),
            Element::text(
                "EVERYONE - THANKS FOR THE SUPPORT!",
                140,
                177,
                FONT_NEW,
                false,
            ),
        ]
    }

    #[must_use]
    pub fn footer(&self) -> Vec<Element> {
        vec![
            Element::right_text("SKI JUMP", 308, 6, FONT_DEFAULT),
            Element::right_text("INTERNATIONAL", 308, 18, FONT_DEFAULT),
            Element::text(format!("v{}", self.version), 245, 30, FONT_DEFAULT, false),
        ]
    }
}

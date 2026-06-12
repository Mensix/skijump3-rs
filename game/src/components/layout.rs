use crate::gfx::palette::{FILL_BORDER, FILL_LINE, FONT_DEFAULT, FONT_HELP, FONT_NEW};
use crate::store::StoreRef;
use crate::text::format;
use crate::text::lang::LangBase;
use engine::ui::Element;
use std::rc::Rc;

#[derive(Clone)]
pub struct MainLayout {
    pub langbase: Rc<LangBase>,
    version: String,
    store: StoreRef,
}

impl MainLayout {
    pub fn new(langbase: Rc<LangBase>, version: String, store: StoreRef) -> Self {
        Self {
            langbase,
            version,
            store,
        }
    }

    #[must_use]
    pub fn background(&self) -> Vec<Element> {
        vec![Element::text(
            self.langbase.lstr(34),
            170,
            51,
            FONT_DEFAULT,
            false,
        )]
    }

    #[must_use]
    pub fn jumpers(&self) -> Vec<Element> {
        let mut els = Vec::new();
        let pb = self.store.profiles();
        for (i, &profile_idx) in pb.active_order.iter().enumerate() {
            if profile_idx >= pb.profiles.len() {
                continue;
            }
            let profile = &pb.profiles[profile_idx];
            let y = (i as i32) * 9 + 64;
            els.push(Element::right_text(
                format::ordinal_dot(i + 1),
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
            Element::fillbox(128, 155, 185, 1, FILL_LINE),
            Element::text(
                format!("{} {}", self.langbase.lstr(35), self.langbase.lstr(36)),
                132,
                163,
                FONT_DEFAULT,
                false,
            ),
            Element::fillbox(132, 175, 177, 22, FILL_BORDER),
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

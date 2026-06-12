use crate::gfx::palette::{FILL_BORDER, FILL_LINE, FONT_DEFAULT, FONT_HELP, FONT_NEW};
use crate::store::StoreRef;
use crate::text::format;
use crate::text::lang::LangBase;
use engine::oxide::PaintCx;
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

    pub fn background(&self, cx: &mut PaintCx<'_>) {
        cx.text((170, 51), FONT_DEFAULT, self.langbase.lstr(34));
    }

    pub fn jumpers(&self, cx: &mut PaintCx<'_>) {
        let pb = self.store.profiles();
        for (i, &profile_idx) in pb.active_order.iter().enumerate() {
            if profile_idx >= pb.profiles.len() {
                continue;
            }
            let profile = &pb.profiles[profile_idx];
            let y = (i as i32) * 9 + 64;
            cx.right_text((162, y), FONT_HELP, format::ordinal_dot(i + 1));
            cx.text((170, y), FONT_HELP, &profile.name);
        }
    }

    pub fn registration(&self, cx: &mut PaintCx<'_>) {
        cx.fill((128, 155, 185, 1), FILL_LINE);
        cx.text(
            (132, 163),
            FONT_DEFAULT,
            format!("{} {}", self.langbase.lstr(35), self.langbase.lstr(36)),
        );
        cx.fill((132, 175, 177, 22), FILL_BORDER);
        cx.text(
            (140, 177),
            FONT_NEW,
            "EVERYONE - THANKS FOR THE SUPPORT!",
        );
    }

    pub fn footer(&self, cx: &mut PaintCx<'_>) {
        cx.right_text((308, 6), FONT_DEFAULT, "SKI JUMP");
        cx.right_text((308, 18), FONT_DEFAULT, "INTERNATIONAL");
        cx.text((245, 30), FONT_DEFAULT, format!("v{}", self.version));
    }
}

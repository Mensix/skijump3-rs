use crate::gfx::theme::{FILL_PURPLE, FILL_DARK, FONT_BODY, FONT_GRAY, FONT_GOLD};
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
        cx.text((170, 51), FONT_BODY, self.langbase.lstr(34));
    }

    pub fn jumpers(&self, cx: &mut PaintCx<'_>) {
        let pb = self.store.profiles();
        for (i, &profile_idx) in pb.active_order.iter().enumerate() {
            if profile_idx >= pb.profiles.len() {
                continue;
            }
            let profile = &pb.profiles[profile_idx];
            let y = (i as i32) * 9 + 64;
            cx.right_text((162, y), FONT_GRAY, format::ordinal_dot(i + 1));
            cx.text((170, y), FONT_GRAY, &profile.name);
        }
    }

    pub fn registration(&self, cx: &mut PaintCx<'_>) {
        cx.fill((128, 155, 185, 1), FILL_DARK);
        cx.text(
            (132, 163),
            FONT_BODY,
            format!("{} {}", self.langbase.lstr(35), self.langbase.lstr(36)),
        );
        cx.fill((132, 175, 177, 22), FILL_PURPLE);
        cx.text((140, 177), FONT_GOLD, "EVERYONE - THANKS FOR THE SUPPORT!");
    }

    pub fn footer(&self, cx: &mut PaintCx<'_>) {
        cx.right_text((308, 6), FONT_BODY, "SKI JUMP");
        cx.right_text((308, 18), FONT_BODY, "INTERNATIONAL");
        cx.text((245, 30), FONT_BODY, format!("v{}", self.version));
    }
}

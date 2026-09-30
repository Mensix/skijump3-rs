pub mod browse;

pub use browse::{HallOfFameView, HillRecordsView};

use crate::data::records::Hiscore;
use crate::gfx::sprites::Sprite;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL};
use crate::store::ResourcesRef;
use crate::text::format::{format_decimal, ordinal_dot};
use crate::ui::UiCanvas;

#[derive(Debug, Clone)]
pub(crate) struct RecordNotification {
    title: String,
    entries: Vec<Hiscore>,
    old_record: Option<Hiscore>,
}

impl RecordNotification {
    pub(crate) fn new(
        title: impl Into<String>,
        entries: Vec<Hiscore>,
        old_record: Option<Hiscore>,
    ) -> Self {
        Self {
            title: title.into(),
            entries,
            old_record,
        }
    }

    pub(crate) fn paint(&self, cx: &mut dyn UiCanvas, resources: &ResourcesRef) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 320, 19), FILL_GRAY);
        cx.pattern_fill((0, 20, 320, 180), BG_PURPLE);
        cx.sprite(Sprite::Logo as u16, (5, 2));
        cx.text((30, 6), FONT_BODY, &self.title);

        if let Some(old) = &self.old_record {
            cx.text((30, 28), FONT_GRAY, resources.langbase.tr(129));
            paint_entry(cx, old, 40);
        }
        let start_y = if self.old_record.is_some() { 64 } else { 32 };
        for (idx, entry) in self.entries.iter().take(15).enumerate() {
            paint_entry(cx, entry, start_y + idx as i32 * 10);
        }
        cx.right_text(
            (319, 190),
            FONT_TEAL,
            &format!("{} ->", resources.langbase.tr(248)),
        );
    }
}

fn paint_entry(cx: &mut dyn UiCanvas, entry: &Hiscore, y: i32) {
    cx.text((30, y), FONT_BODY, &entry.name);
    cx.right_text((205, y), FONT_GOLD, &ordinal_dot(entry.pos));
    cx.right_text((275, y), FONT_BODY, &format_decimal(entry.score));
}

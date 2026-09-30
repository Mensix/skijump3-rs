use crate::gfx::theme::FONT_BODY;
use crate::text::lang::LangBase;
use crate::ui::UiCanvas;

pub(crate) fn paint_numbered_menu(
    cx: &mut dyn UiCanvas,
    lang: &LangBase,
    labels: [usize; 7],
    y_offsets: [i32; 7],
    selected: Option<usize>,
) {
    for (i, label) in labels.iter().enumerate() {
        let num = if i == 6 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + y_offsets[i];
        cx.text(
            (11, y),
            FONT_BODY,
            &format!("{} - {}", num, lang.tr(*label)),
        );
    }
    if let Some(selected) = selected {
        let selected = selected.min(y_offsets.len().saturating_sub(1));
        let y = 94 + (selected as i32) * 12 + y_offsets[selected];
        cx.stroke((5, y, 109, 13), FONT_BODY);
    }
}

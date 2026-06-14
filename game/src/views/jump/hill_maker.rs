use crate::gfx::theme::{BG_LEFT, BLACK, FONT_DEFAULT, FONT_GOLD, FONT_GREET, FONT_HELP};
use crate::route::RouteTarget;
use crate::store::ResourcesRef;
use crate::text::layout::lstr;
use engine::oxide::widgets::menu::{MenuItem, PixelMenu};
use engine::oxide::Widget;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};

pub struct HillMakerView {
    resources: ResourcesRef,
    menu: PixelMenu,
}

impl HillMakerView {
    pub fn new(resources: ResourcesRef) -> Self {
        let items = vec![MenuItem::new(1, lstr(&resources.langbase, 275, "*Add New Hill*"))];
        let menu = PixelMenu::new(99, 14, 221, 8, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false);
        Self { resources, menu }
    }
}

impl Screen<RouteTarget> for HillMakerView {
    fn event(&mut self, _cx: &mut ScreenEventCx<RouteTarget>, _event: UiEvent) {}

    fn paint(&self, cx: &mut PaintCx<'_>) {
        let lb = &*self.resources.langbase;

        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 320, 200), BG_LEFT);

        cx.text((5, 5), FONT_GOLD, lstr(lb, 270, "SJ3 Hill Maker"));

        cx.text((5, 21), FONT_HELP, lstr(lb, 271, "(use arrows, DEL,"));
        cx.text((5, 29), FONT_HELP, lstr(lb, 272, " ENTER or ESC)"));

        let col1 = 100i32;
        let col2 = 160i32;

        cx.text((col1, 5), FONT_GREET, lstr(lb, 273, "Filename"));
        cx.text((col2, 5), FONT_GREET, lstr(lb, 274, "Hillname"));

        cx.text(
            (5, 45),
            FONT_GREET,
            format!("{} 1 {} 1", lstr(lb, 157, "Page"), lstr(lb, 8, "of")),
        );

        cx.text((col1, 13), FONT_GOLD, lstr(lb, 275, "*Add New Hill*"));
        self.menu.paint(cx);

        cx.text((col1, 29), FONT_DEFAULT, lstr(lb, 276, "-Exit-"));
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::NoneBlack
    }
}

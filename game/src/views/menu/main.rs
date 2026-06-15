use crate::components::layout::MainLayout;
use crate::gfx::theme::{BG_DARK, FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::store::StoreRef;
use engine::oxide::widgets::menu::PixelMenu;
use engine::oxide::Widget;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};

pub struct MainMenuView {
    menu: PixelMenu,
    layout: MainLayout,
}

const MENU_ACTIONS: &[Option<RouteTarget>] = &[
    Some(RouteTarget::JumpMenu),
    Some(RouteTarget::ProfilesList),
    Some(RouteTarget::OptionsMenu),
    Some(RouteTarget::HallOfFame),
    Some(RouteTarget::HillRecords),
    Some(RouteTarget::Replays),
    Some(RouteTarget::Quit),
];

impl MainMenuView {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(layout: MainLayout, store: StoreRef) -> Self {
        use engine::oxide::widgets::menu::MenuItem as OxideMenuItem;

        let items = vec![
            OxideMenuItem::new(1, ""),
            OxideMenuItem::new(2, ""),
            OxideMenuItem::new(3, ""),
            OxideMenuItem::new(4, ""),
            OxideMenuItem::new(5, ""),
            OxideMenuItem::new(6, ""),
            OxideMenuItem::new(0, "").with_y(12),
        ];
        let selection = store
            .selected_main_menu()
            .min(items.len().saturating_sub(1));
        let mut menu = PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        menu.set_selected(selection);
        Self { menu, layout }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        self.layout.background(cx);
        self.layout.jumpers(cx);
        self.layout.registration(cx);
        cx.fill((11, 80, 100, 6), BG_DARK);
        cx.text((11, 80), FONT_GOLD, self.layout.langbase.lstr(17));
        paint_main_menu(cx, &self.menu, &self.layout);
        self.layout.footer(cx);
    }
}

impl Screen<RouteTarget> for MainMenuView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(0 | 7) => cx.quit(),
            Some(n) => {
                if let Some(route) = MENU_ACTIONS.get(n - 1).and_then(|&a| a) {
                    cx.navigate(route);
                }
            }
            _ => {}
        }
        if ecx.is_consumed() {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn paint_main_menu(cx: &mut PaintCx<'_>, menu: &PixelMenu, layout: &MainLayout) {
    let y_offsets = [0, 0, 0, 0, 0, 0, 12];
    for (i, label) in [20, 21, 22, 23, 24, 25, 26].iter().enumerate() {
        let num = if i == 6 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + y_offsets[i];
        cx.text(
            (11, y),
            FONT_BODY,
            format!("{} - {}", num, layout.langbase.lstr(*label)),
        );
    }
    let selected = menu.selected().min(y_offsets.len().saturating_sub(1));
    let y = 94 + (selected as i32) * 12 + y_offsets[selected];
    cx.stroke((5, y, 109, 13), FONT_BODY);
}

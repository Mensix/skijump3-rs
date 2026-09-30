use crate::gfx::sprites;
use crate::gfx::theme::{BG_PURPLE, BLACK, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_TEAL};
use crate::jump::replay::ReplayTrace;
use crate::route::{ReplayReturn, RouteTarget};
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use crate::ui::UiCanvas;
use crate::ui::{EventCx, MenuAction, MenuItem, PixelMenu, ScreenEventCx, UiEvent};

pub struct WelcomeScreenView {
    menu: PixelMenu,
    resources: ResourcesRef,
}

impl WelcomeScreenView {
    pub fn new(resources: ResourcesRef) -> Self {
        let count = resources.langbase.language_count();
        let items: Vec<MenuItem> = (0..count)
            .map(|i| MenuItem::new(i as u8, format!("{}", i)))
            .collect();
        Self {
            menu: PixelMenu::new(112, 64, 100, 8, items, FONT_BODY, FONT_BODY)
                .with_labels(false)
                .with_box(false),
            resources,
        }
    }

    fn paint_content(&self, cx: &mut dyn UiCanvas) {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 0, 51, 200), FILL_GRAY);
        cx.pattern_fill((52, 0, 216, 200), BG_PURPLE);
        cx.pattern_fill((269, 0, 51, 200), FILL_GRAY);
        cx.sprite(sprites::Sprite::Logo as u16, (80, 6));
        cx.right_text((240, 6), FONT_BODY, "WELCOME!");
        cx.right_text((240, 16), FONT_GOLD, "TERVETULOA!");
        cx.right_text((240, 26), FONT_TEAL, "WILLKOMMEN!");
        cx.right_text((240, 36), FONT_BODY, "VÄLKOMMEN!");
        cx.text((100, 50), FONT_BODY, "PLEASE CHOOSE A LANGUAGE:");

        for (i, lang) in self.resources.langbase.languages().iter().enumerate() {
            let y = ((i + 1) * 8 + 55) as i32;
            cx.center_text((155, y), FONT_GOLD, &lang.name);
        }

        let y = 61 + (self.menu.selected() as i32) * 8;
        cx.stroke((106, y, 101, 9), FONT_BODY);
    }
}

impl GameScreen for WelcomeScreenView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let mut ecx = EventCx::default();
        if let Some(MenuAction::Item(n)) = self.menu.event_action(&mut ecx, event) {
            self.resources.langbase.select(n);
            cx.state.config.language = self.resources.langbase.saved_language();
            cx.persistence().save_config(&cx.state.config);
            let intro = ReplayTrace::from_sjr_bytes(
                &self.resources.files.read_save_or_asset("INTRO.SJR"),
                true,
            );
            nav.navigate(intro.map_or(RouteTarget::MainMenu, |trace| {
                RouteTarget::ReplayPlayback {
                    trace: Box::new(trace),
                    return_to: ReplayReturn::MainMenu,
                }
            }));
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint);
    }
}

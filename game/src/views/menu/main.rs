use crate::components::layout::MainLayout;
use crate::components::modal::alert_prompt;
use crate::gfx::theme::{FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use engine::oxide::widgets::menu::PixelMenu;
use engine::oxide::Widget;
use engine::oxide::{Blinker, PaintCx, ScreenBackground, ScreenEventCx, UiEvent};

pub struct MainMenuView {
    menu: PixelMenu,
    confirming_quit: bool,
    quit_question: String,
    quit_prompt: String,
    quit_blinker: Blinker,
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
    pub fn new() -> Self {
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
        let menu = PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        Self {
            menu,
            confirming_quit: false,
            quit_question: String::new(),
            quit_prompt: String::new(),
            quit_blinker: Blinker::new(),
        }
    }
}

impl GameScreen for MainMenuView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if self.confirming_quit {
            match event {
                UiEvent::Text(c) if is_yes(c, &cx.layout) => nav.quit(),
                UiEvent::KeyDown(_) | UiEvent::Text(_) => {
                    self.confirming_quit = false;
                    nav.consume();
                }
                _ => {}
            }
            return;
        }

        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event(&mut ecx, event) {
            Some(0 | 7) => {
                self.confirming_quit = true;
                self.quit_blinker.reset();
                let qi = 251 + (cx.state.rng.random_i32(3) as usize).min(2);
                let pi = 256 + (cx.state.rng.random_i32(3) as usize).min(2);
                self.quit_question = cx.layout.langbase.lstr(qi).to_string();
                self.quit_prompt = cx.layout.langbase.lstr(pi).to_string();
                nav.consume();
            }
            Some(n) => {
                if let Some(route) = MENU_ACTIONS.get(n - 1).and_then(|&a| a) {
                    nav.navigate(route);
                }
            }
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        cx.layout.background(paint);
        cx.layout.jumpers(paint, &cx.state.profiles);
        cx.layout.registration(paint);
        paint.text((11, 80), FONT_GOLD, cx.layout.langbase.lstr(17));
        paint_main_menu(paint, &self.menu, &cx.layout);
        cx.layout.footer(paint);
        if self.confirming_quit {
            let cursor_on = self.quit_blinker.visible(11, 10);
            paint_quit_confirm(paint, &self.quit_question, &self.quit_prompt, cursor_on);
        }
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

fn paint_quit_confirm(cx: &mut PaintCx<'_>, question: &str, prompt: &str, cursor_on: bool) {
    alert_prompt(cx, question, format!("{prompt} (Y/N):"), cursor_on);
}

fn is_yes(c: char, layout: &MainLayout) -> bool {
    let localized = layout
        .langbase
        .lstr(6)
        .chars()
        .next()
        .unwrap_or('Y')
        .to_ascii_uppercase();
    c.to_ascii_uppercase() == localized || c.eq_ignore_ascii_case(&'Y')
}

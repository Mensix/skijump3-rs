use crate::components::layout::MainLayout;
use crate::components::modal::alert_prompt;
use crate::gfx::theme::{FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use engine::oxide::widgets::menu::{MenuAction, PixelMenu};
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx, UiEvent};

pub struct MainMenuView {
    menu: PixelMenu,
    confirming_quit: bool,
    quit_question: String,
    quit_prompt: String,
}

const MENU_ACTIONS: &[Option<RouteTarget>] = &[
    Some(RouteTarget::JumpMenu),
    Some(RouteTarget::ProfilesList),
    Some(RouteTarget::OptionsMenu),
    Some(RouteTarget::HallOfFame),
    Some(RouteTarget::HillRecords),
    Some(RouteTarget::MultiplayerMenu),
    Some(RouteTarget::Replays),
    Some(RouteTarget::Quit),
];

impl MainMenuView {
    pub fn new() -> Self {
        use engine::oxide::widgets::menu::MenuItem as OxideMenuItem;

        let items = vec![
            OxideMenuItem::new(0, ""),
            OxideMenuItem::new(1, ""),
            OxideMenuItem::new(2, ""),
            OxideMenuItem::new(3, ""),
            OxideMenuItem::new(4, ""),
            OxideMenuItem::new(5, ""),
            OxideMenuItem::new(6, ""),
            OxideMenuItem::new(7, "").with_y(12),
        ];
        let menu = PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        Self {
            menu,
            confirming_quit: false,
            quit_question: String::new(),
            quit_prompt: String::new(),
        }
    }
}

impl GameScreen for MainMenuView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let lang = &cx.layout.langbase;
        if self.confirming_quit {
            match event {
                UiEvent::Text(c) if is_yes(c, cx.layout) => nav.quit(),
                UiEvent::KeyDown(_) | UiEvent::Text(_) => {
                    self.confirming_quit = false;
                    nav.consume();
                }
                _ => {}
            }
            return;
        }

        let mut ecx = engine::oxide::widget::EventCx::default();
        match self.menu.event_action(&mut ecx, event) {
            Some(MenuAction::Item(6)) => {
                self.confirming_quit = true;
                let qi = 251 + (cx.state.rng.random_i32(3) as usize).min(2);
                let pi = 256 + (cx.state.rng.random_i32(3) as usize).min(2);
                self.quit_question = lang.tr(qi).to_string();
                self.quit_prompt = lang.tr(pi).to_string();
                nav.consume();
            }
            Some(MenuAction::Item(n)) => {
                if let Some(route) = MENU_ACTIONS.get(n).and_then(|&a| a) {
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
        let lang = &cx.layout.langbase;
        cx.layout.background(paint);
        cx.layout.jumpers(paint, &cx.state.profiles);
        cx.layout.registration(paint);
        paint.text((11, 80), FONT_GOLD, lang.tr(17));
        paint_main_menu(paint, &self.menu, cx.layout);
        cx.layout.footer(paint);
        if self.confirming_quit {
            paint_quit_confirm(paint, &self.quit_question, &self.quit_prompt);
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn paint_main_menu(cx: &mut PaintCx<'_>, menu: &PixelMenu, layout: &MainLayout) {
    let lang = &layout.langbase;
    let y_offsets = [0, 0, 0, 0, 0, 0, 0, 12];
    for (i, label) in [20, 21, 22, 23, 24, 524, 25, 26].iter().enumerate() {
        let num = if i == 7 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + y_offsets[i];
        cx.text((11, y), FONT_BODY, format!("{} - {}", num, lang.tr(*label)));
    }
    let selected = menu.selected().min(y_offsets.len().saturating_sub(1));
    let y = 94 + (selected as i32) * 12 + y_offsets[selected];
    cx.stroke((5, y, 109, 13), FONT_BODY);
}

fn paint_quit_confirm(cx: &mut PaintCx<'_>, question: &str, prompt: &str) {
    alert_prompt(cx, question, prompt, true);
}

fn is_yes(c: char, layout: &MainLayout) -> bool {
    let lang = &layout.langbase;
    let localized = lang
        .tr(6)
        .chars()
        .next()
        .unwrap_or('Y')
        .to_ascii_uppercase();
    c.to_ascii_uppercase() == localized || c.eq_ignore_ascii_case(&'Y')
}

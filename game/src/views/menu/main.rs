use crate::components::layout::MainLayout;
use crate::components::modal::alert_box;
use crate::gfx::theme::{BG_DARK, BG_PURPLE, BG_RED, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::route::RouteTarget;
use crate::store::GameStateRef;
use engine::oxide::widgets::menu::PixelMenu;
use engine::oxide::Widget;
use engine::oxide::{Blinker, PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};

pub struct MainMenuView {
    menu: PixelMenu,
    layout: MainLayout,
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
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(layout: MainLayout, store: GameStateRef) -> Self {
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
            .borrow()
            .selected_main_menu
            .min(items.len().saturating_sub(1));
        let mut menu = PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        menu.set_selected(selection);
        Self {
            menu,
            layout,
            confirming_quit: false,
            quit_question: String::new(),
            quit_prompt: String::new(),
            quit_blinker: Blinker::new(),
        }
    }

    fn paint_content(&mut self, cx: &mut PaintCx<'_>) {
        self.layout.background(cx);
        self.layout.jumpers(cx);
        self.layout.registration(cx);
        cx.fill((11, 80, 100, 6), BG_DARK);
        cx.text((11, 80), FONT_GOLD, self.layout.langbase.lstr(17));
        paint_main_menu(cx, &self.menu, &self.layout);
        self.layout.footer(cx);
        if self.confirming_quit {
            let cursor_on = self.quit_blinker.visible(11, 10);
            paint_quit_confirm(cx, &self.quit_question, &self.quit_prompt, cursor_on);
        }
    }
}

impl Screen<RouteTarget> for MainMenuView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if self.confirming_quit {
            match event {
                UiEvent::Text(c) if is_yes(c, &self.layout) => cx.quit(),
                UiEvent::KeyDown(_) | UiEvent::Text(_) => {
                    self.confirming_quit = false;
                    cx.consume();
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
                let mut state = self.layout.state.borrow_mut();
                let qi = 251 + (state.rng.random_i32(3) as usize).min(2);
                let pi = 256 + (state.rng.random_i32(3) as usize).min(2);
                self.quit_question = self.layout.langbase.lstr(qi).to_string();
                self.quit_prompt = self.layout.langbase.lstr(pi).to_string();
                cx.consume();
            }
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

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
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

fn paint_quit_confirm(cx: &mut PaintCx<'_>, question: &str, prompt: &str, cursor_on: bool) {
    let prompt_w = cx.string_width(prompt) as i32;
    alert_box(cx, (60, 80, 200, 51), BG_RED);
    cx.text((70, 90), FONT_GOLD, question);
    cx.text((70, 110), FONT_GOLD, prompt);
    let yn_x = 70 + prompt_w + 4;
    cx.text((yn_x, 110), FONT_GRAY, "(Y/N)");
    let cursor_x = yn_x + 25;
    let cursor_y = 110;
    cx.fill((cursor_x - 2, cursor_y - 2, 9, 11), BG_PURPLE);
    if cursor_on {
        cx.fill((cursor_x, cursor_y + 6, 5, 1), FONT_GOLD);
    }
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

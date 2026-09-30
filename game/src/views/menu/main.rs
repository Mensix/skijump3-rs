use super::paint_numbered_menu;
use crate::components::layout::MainLayout;
use crate::components::modal::{ConfirmationChoice, Modal, ModalResult};
use crate::gfx::theme::{FONT_BODY, FONT_GOLD};
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::ui::UiCanvas;
use crate::ui::{
    EventCx, Key, MenuAction, MenuItem, PixelMenu, ScreenBackground, ScreenEventCx, UiEvent,
};

pub struct MainMenuView {
    menu: PixelMenu,
    quit_modal: Option<Modal>,
}

impl MainMenuView {
    pub fn new() -> Self {
        let items = vec![
            MenuItem::new(0, ""),
            MenuItem::new(1, ""),
            MenuItem::new(2, ""),
            MenuItem::new(3, ""),
            MenuItem::new(4, ""),
            MenuItem::new(5, ""),
            MenuItem::new(6, "").with_y(12),
        ];
        let menu = PixelMenu::new(11, 97, 108, 12, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        Self {
            menu,
            quit_modal: None,
        }
    }

    fn begin_quit_confirmation(&mut self, cx: &mut GameCx<'_>) {
        let lang = &cx.layout.langbase;
        let question = 251 + (cx.state.rng.random_i32(3) as usize).min(2);
        let prompt = 256 + (cx.state.rng.random_i32(3) as usize).min(2);
        self.quit_modal = Some(Modal::confirm(lang.tr(question), lang.tr(prompt)));
    }
}

impl GameScreen for MainMenuView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let Some(modal) = &self.quit_modal {
            match modal.event(event, &cx.layout.langbase) {
                Some(ModalResult::Confirmed(ConfirmationChoice::Yes)) => {
                    self.quit_modal = None;
                    nav.quit();
                }
                Some(ModalResult::Confirmed(ConfirmationChoice::No))
                | Some(ModalResult::Dismissed) => {
                    self.quit_modal = None;
                    nav.consume();
                }
                None => nav.consume(),
            }
            return;
        }

        if matches!(event, UiEvent::KeyDown(Key::Escape | Key::F10)) {
            self.begin_quit_confirmation(cx);
            nav.consume();
            return;
        }

        let mut ecx = EventCx::default();
        match self.menu.event_action(&mut ecx, event) {
            Some(MenuAction::Item(6)) => {
                self.begin_quit_confirmation(cx);
                nav.consume();
            }
            Some(MenuAction::Item(n)) => nav.navigate(menu_route(n)),
            _ => {}
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        let lang = &cx.layout.langbase;
        cx.layout.background(paint);
        cx.layout.jumpers(paint, &cx.state.profiles);
        cx.layout.registration(paint);
        paint.text((11, 80), FONT_GOLD, lang.tr(17));
        paint_main_menu(paint, &self.menu, cx.layout);
        cx.layout.footer(paint);
        if let Some(modal) = &self.quit_modal {
            modal.paint(paint, lang);
        }
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }

    fn has_modal(&self) -> bool {
        self.quit_modal.is_some()
    }
}

fn menu_route(n: usize) -> RouteTarget {
    match n {
        0 => RouteTarget::JumpMenu,
        1 => RouteTarget::ProfilesList,
        2 => RouteTarget::OptionsMenu,
        3 => RouteTarget::HallOfFame,
        4 => RouteTarget::HillRecords,
        5 => RouteTarget::Replays,
        _ => RouteTarget::MainMenu,
    }
}

fn paint_main_menu(cx: &mut dyn UiCanvas, menu: &PixelMenu, layout: &MainLayout) {
    paint_numbered_menu(
        cx,
        &layout.langbase,
        [20, 21, 22, 23, 24, 25, 26],
        [0, 0, 0, 0, 0, 0, 12],
        Some(menu.selected()),
    );
}

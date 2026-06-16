use crate::gfx::theme::FONT_BODY;
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::save::SaveRef;
use crate::store::{GameStateRef, ResourcesRef};
use crate::text::lang::LangBase;
use engine::oxide::widgets::menu::{MenuItem as OxideMenuItem, PixelMenu};
use engine::oxide::{Blinker, PaintCx, Screen, ScreenEventCx, UiEvent};
use std::cell::Cell;

use super::state::SetupModal;

pub struct SetupView {
    pub(crate) resources: ResourcesRef,
    pub(crate) store: GameStateRef,
    pub(crate) save_manager: SaveRef,
    pub(crate) screen: Cell<usize>,
    pub(crate) selected_by_screen: [Cell<usize>; 4],
    pub(crate) menu: PixelMenu,
    pub(crate) modal: Cell<Option<SetupModal>>,
    pub(crate) cursor_blink: Blinker,
}

impl SetupView {
    pub fn new(resources: ResourcesRef, store: GameStateRef, save_manager: SaveRef) -> Self {
        let menu = Self::make_menu(0, 0);
        Self {
            resources,
            store,
            save_manager,
            screen: Cell::new(0),
            selected_by_screen: [Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0)],
            menu,
            modal: Cell::new(None),
            cursor_blink: Blinker::new(),
        }
    }

    pub(crate) fn make_menu(screen: usize, selected: usize) -> PixelMenu {
        let entries = match screen {
            0 => 6,
            1 => 4,
            2 => 11,
            3 => 5,
            _ => 0,
        };
        let items = (0..entries).map(|_| OxideMenuItem::new(0, "")).collect();
        let mut m = PixelMenu::new(35, 40, 221, 10, items, FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false)
            .trailing("", 0);
        m.set_selected(selected.min(entries));
        m
    }

    pub(crate) fn switch_screen(&mut self, new_screen: usize) {
        let old_screen = self.screen.get();
        if old_screen < self.selected_by_screen.len() {
            self.selected_by_screen[old_screen].set(self.menu.selected());
        }
        let selected = self
            .selected_by_screen
            .get(new_screen)
            .map(Cell::get)
            .unwrap_or_default();
        self.screen.set(new_screen);
        self.menu = Self::make_menu(new_screen, selected);
    }

    pub(crate) fn langbase(&self) -> &LangBase {
        &self.resources.langbase
    }

    pub(crate) fn config(&self) -> std::cell::Ref<'_, Config> {
        std::cell::Ref::map(self.store.borrow(), |s| &s.config)
    }

    pub(crate) fn save_manager(&self) -> &SaveRef {
        &self.save_manager
    }
}

impl Screen<RouteTarget> for SetupView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_from_ui(event) else {
            return;
        };
        if let Some(route) = super::actions::handle_event(self, event) {
            cx.navigate(route);
        } else {
            cx.consume();
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        super::render::paint_content(self, cx);
    }
}

fn input_from_ui(event: UiEvent) -> Option<UiEvent> {
    match event {
        UiEvent::KeyDown(_) | UiEvent::Text(_) => Some(event),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

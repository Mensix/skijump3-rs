use crate::gfx::palette::FONT_DEFAULT;
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::save::SaveManager;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::lang::LangBase;
use engine::oxide::widgets::menu::{PixelMenu, MenuItem as OxideMenuItem};
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use std::cell::Cell;
use std::rc::Rc;

use super::state::SetupModal;

pub struct SetupView {
    pub(crate) resources: ResourcesRef,
    pub(crate) store: StoreRef,
    pub(crate) screen: Cell<usize>,
    pub(crate) menu: PixelMenu,
    pub(crate) modal: Cell<Option<SetupModal>>,
}

impl SetupView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let menu = Self::make_menu(0, &resources.langbase, 0);
        Self {
            resources,
            store,
            screen: Cell::new(0),
            menu,
            modal: Cell::new(None),
        }
    }

    pub(crate) fn make_menu(screen: usize, _langbase: &Rc<LangBase>, selected: usize) -> PixelMenu {
        let entries = match screen {
            0 => 6,
            1 => 4,
            2 => 11,
            3 => 5,
            _ => 0,
        };
        let items = (0..entries).map(|_| OxideMenuItem::new(0, "")).collect();
        let mut m = PixelMenu::new(35, 40, 221, 10, items, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false)
            .with_exit("", 0);
        m.set_selected(selected.min(entries));
        m
    }

    pub(crate) fn switch_screen(&mut self, new_screen: usize) {
        let selected = self.menu.selected();
        self.screen.set(new_screen);
        self.menu = Self::make_menu(new_screen, &self.resources.langbase, selected);
    }

    pub(crate) fn langbase(&self) -> &LangBase {
        &self.resources.langbase
    }

    pub(crate) fn config(&self) -> std::cell::Ref<'_, Config> {
        self.resources.save_manager.config.borrow()
    }

    pub(crate) fn save_manager(&self) -> &SaveManager {
        &self.resources.save_manager
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

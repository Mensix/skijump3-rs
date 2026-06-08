use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::FONT_DEFAULT;
use crate::route::RouteTarget;
use crate::save::config::Config;
use crate::save::SaveManager;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::lang::LangBase;
use engine::ui::{Element, Event, View};
use std::cell::Cell;
use std::rc::Rc;

use super::state::SetupModal;

pub struct SetupView {
    pub(crate) resources: ResourcesRef,
    pub(crate) store: StoreRef,
    pub(crate) screen: Cell<usize>,
    pub(crate) menu: Menu,
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

    pub(crate) fn make_menu(screen: usize, langbase: &Rc<LangBase>, selected: usize) -> Menu {
        let entries = match screen {
            0 => 6,
            1 => 4,
            2 => 11,
            3 => 5,
            _ => 0,
        };
        let items = (0..entries).map(|_| MenuItem::new(0, 0)).collect();
        let mut m = Menu::new(35, 40, 221, 10, items, langbase, FONT_DEFAULT, FONT_DEFAULT)
            .with_labels(false)
            .with_box(false)
            .with_exit(154, 0);
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

impl View<RouteTarget> for SetupView {
    fn elements(&self) -> Vec<Element> {
        super::render::elements(self)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        super::actions::handle_event(self, event)
    }
}

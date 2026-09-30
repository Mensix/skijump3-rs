use super::state::{setup_page, SetupModal};
use crate::gfx::theme::FONT_BODY;
use crate::route::RouteTarget;
use crate::screen::Persistence;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use crate::ui::UiCanvas;
use crate::ui::{Blinker, PixelMenu, ScreenEventCx, UiEvent};

pub struct SetupView {
    pub(crate) resources: ResourcesRef,
    pub(crate) persistence: Persistence,
    pub(crate) screen: usize,
    pub(crate) selected_by_screen: [usize; 4],
    pub(crate) menu: PixelMenu,
    pub(crate) modal: Option<SetupModal>,
    pub(crate) cursor_blink: Blinker,
}

impl SetupView {
    pub fn new(resources: ResourcesRef, persistence: Persistence) -> Self {
        let menu = PixelMenu::new(35, 40, 221, 10, vec![], FONT_BODY, FONT_BODY)
            .with_labels(false)
            .with_box(false);
        let mut view = Self {
            resources,
            persistence,
            screen: 0,
            selected_by_screen: [0, 0, 0, 0],
            menu,
            modal: None,
            cursor_blink: Blinker::new(),
        };
        view.set_screen_items(0, 0);
        view
    }

    pub(crate) fn persistence(&self) -> &Persistence {
        &self.persistence
    }
    fn set_screen_items(&mut self, new_screen: usize, selected: usize) {
        let entries = setup_page(new_screen).map_or(0, |page| page.items.len());
        self.menu.set_index_items(entries + 1);
        self.menu.set_selected(selected.min(entries));
    }

    pub(crate) fn switch_screen(&mut self, new_screen: usize) {
        let old_screen = self.screen;
        if old_screen < self.selected_by_screen.len() {
            self.selected_by_screen[old_screen] = self.menu.selected();
        }
        let selected = self
            .selected_by_screen
            .get(new_screen)
            .copied()
            .unwrap_or_default();
        self.screen = new_screen;
        self.set_screen_items(new_screen, selected);
    }
}

impl GameScreen for SetupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_from_ui(event) else {
            return;
        };
        if let Some(route) = super::actions::handle_event(self, cx.state, event) {
            if route == RouteTarget::MainMenu {
                let place = cx.state.config.wind_position as u8;
                cx.state.wind.set_place(place);
                nav.back();
            } else {
                nav.navigate(route);
            }
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        super::render::paint_content(self, cx.state, paint);
    }

    fn has_modal(&self) -> bool {
        self.modal.is_some()
    }
}

fn input_from_ui(event: UiEvent) -> Option<UiEvent> {
    match event {
        UiEvent::KeyDown(_) | UiEvent::Text(_) | UiEvent::TextWithModifiers(..) => Some(event),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

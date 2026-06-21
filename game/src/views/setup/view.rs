use super::state::SetupModal;
use crate::gfx::theme::FONT_BODY;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use engine::oxide::widgets::menu::{MenuItem as OxideMenuItem, PixelMenu};
use engine::oxide::{Blinker, PaintCx, ScreenEventCx, UiEvent};

pub struct SetupView {
    pub(crate) resources: ResourcesRef,
    pub(crate) save_manager: SaveRef,
    pub(crate) screen: usize,
    pub(crate) selected_by_screen: [usize; 4],
    pub(crate) menu: PixelMenu,
    pub(crate) modal: Option<SetupModal>,
    pub(crate) cursor_blink: Blinker,
}

impl SetupView {
    pub fn new(resources: ResourcesRef, save_manager: SaveRef) -> Self {
        let menu = Self::make_menu(0, 0);
        Self {
            resources,
            save_manager,
            screen: 0,
            selected_by_screen: [0, 0, 0, 0],
            menu,
            modal: None,
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
            .trailing("", 0)
            .with_return_index(true);
        m.set_selected(selected.min(entries));
        m
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
        self.menu = Self::make_menu(new_screen, selected);
    }

    pub(crate) fn save_manager(&self) -> &SaveRef {
        &self.save_manager
    }
}

impl GameScreen for SetupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_from_ui(event) else {
            return;
        };
        if let Some(route) = super::actions::handle_event(self, cx.state, event) {
            nav.navigate(route);
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        super::render::paint_content(self, cx.state, paint);
    }
}

fn input_from_ui(event: UiEvent) -> Option<UiEvent> {
    match event {
        UiEvent::KeyDown(_) | UiEvent::Text(_) => Some(event),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

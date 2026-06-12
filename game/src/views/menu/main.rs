use crate::components::layout::{self, MainLayout};
use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{BG_ERASE, FONT_DEFAULT, FONT_HEADER};
use crate::route::RouteTarget;
use crate::store::StoreRef;
use engine::oxide::legacy::{commands_to_elements, event_from_ui, paint_elements};
use engine::oxide::{CommandBuffer, NavAction, PaintCx, Screen, ScreenEventCx, UiEvent};
use engine::ui::{Component, Element, Event, View};

pub struct MainMenuView {
    menu: Menu,
    layout: MainLayout,
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
    pub fn new(layout: MainLayout, store: StoreRef) -> Self {
        let items = vec![
            MenuItem::new(1, 20),
            MenuItem::new(2, 21),
            MenuItem::new(3, 22),
            MenuItem::new(4, 23),
            MenuItem::new(5, 24),
            MenuItem::new(6, 25),
            MenuItem::with_y(0, 26, 12),
        ];
        let selection = store
            .selected_main_menu()
            .min(items.len().saturating_sub(1));
        let mut menu = Menu::new(
            11,
            97,
            108,
            12,
            items,
            &layout.langbase,
            FONT_DEFAULT,
            FONT_DEFAULT,
        );
        menu.set_selected(selection);
        Self { menu, layout }
    }

    fn legacy_elements(&self) -> Vec<Element> {
        let mut els = self.layout.background();
        els.extend(self.layout.jumpers());
        els.extend(self.layout.registration());
        els.extend(layout::header_elements(
            self.layout.langbase.lstr(17),
            11,
            80,
            FONT_HEADER,
            BG_ERASE,
        ));
        els.extend(self.menu.elements());
        els.extend(self.layout.footer());
        els
    }
}

impl View<RouteTarget> for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut commands = CommandBuffer::new();
        let mut cx = PaintCx::new(&mut commands);
        Screen::paint(self, &mut cx);
        commands_to_elements(&commands)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let mut cx = ScreenEventCx::default();
        Screen::event(self, &mut cx, event.into());
        match cx.take_action() {
            NavAction::Navigate(route) => Some(route),
            NavAction::Back => Some(RouteTarget::Back),
            NavAction::Quit => Some(RouteTarget::Quit),
            NavAction::None => None,
        }
    }

    fn gpu_background(&self) -> engine::ui::BackgroundMode {
        engine::ui::BackgroundMode::MainPng
    }
}

impl Screen<RouteTarget> for MainMenuView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = event_from_ui(event) else {
            return;
        };
        match self.menu.handle_event(&event) {
            Some(0 | 7) => cx.quit(),
            Some(n) => {
                if let Some(route) = MENU_ACTIONS.get(n - 1).and_then(|&a| a) {
                    cx.navigate(route);
                }
            }
            _ => {}
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        paint_elements(cx, &self.legacy_elements());
    }
}

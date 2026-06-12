use crate::components::layout::MainLayout;
use crate::components::menu::{Menu, MenuItem};
use crate::gfx::palette::{BG_ERASE, FONT_DEFAULT, FONT_HEADER};
use crate::route::RouteTarget;
use crate::store::StoreRef;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};
use engine::ui::{Component, Element, Event, Key};

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

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        paint_component_output(cx, self.layout.background());
        paint_component_output(cx, self.layout.jumpers());
        paint_component_output(cx, self.layout.registration());
        cx.fill((11, 80, 100, 6), BG_ERASE);
        cx.text((11, 80), FONT_HEADER, self.layout.langbase.lstr(17));
        paint_main_menu(cx, &self.menu, &self.layout);
        paint_component_output(cx, self.layout.footer());
    }
}

impl Screen<RouteTarget> for MainMenuView {
    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        let Some(event) = input_event(event) else {
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
        self.paint_content(cx);
    }

    fn background(&self) -> ScreenBackground {
        ScreenBackground::MainPng
    }
}

fn input_event(event: UiEvent) -> Option<Event> {
    match event {
        UiEvent::KeyDown(key) => Some(Event::Keyboard(key)),
        UiEvent::Text(ch) => Some(Event::Keyboard(Key::Char(ch))),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

fn paint_main_menu(cx: &mut PaintCx<'_>, menu: &Menu, layout: &MainLayout) {
    let y_offsets = [0, 0, 0, 0, 0, 0, 12];
    for (i, label) in [20, 21, 22, 23, 24, 25, 26].iter().enumerate() {
        let num = if i == 6 { 0 } else { i + 1 };
        let y = 98 + (i as i32) * 12 + y_offsets[i];
        cx.text(
            (11, y),
            FONT_DEFAULT,
            format!("{} - {}", num, layout.langbase.lstr(*label)),
        );
    }
    let selected = menu.selected().min(y_offsets.len().saturating_sub(1));
    let y = 94 + (selected as i32) * 12 + y_offsets[selected];
    cx.stroke((5, y, 109, 13), FONT_DEFAULT);
}

fn paint_component_output(cx: &mut PaintCx<'_>, elements: Vec<Element>) {
    for element in elements {
        match element {
            Element::Text {
                text,
                x,
                y,
                color,
                right,
                center,
            } if center => cx.center_text((x, y), color, text),
            Element::Text {
                text,
                x,
                y,
                color,
                right,
                center: _,
            } if right => cx.right_text((x, y), color, text),
            Element::Text {
                text, x, y, color, ..
            } => cx.text((x, y), color, text),
            Element::Sprite(idx, x, y) => cx.sprite(idx, (x, y)),
            Element::Fillbox { x, y, w, h, color } => cx.fill((x, y, w, h), color),
            Element::FillArea { thing } => cx.dither_fill(thing),
            Element::Box { x, y, w, h, color } => cx.stroke((x, y, w, h), color),
            Element::Container(children) => paint_component_output(cx, children),
            Element::SpriteRemapped(_, _, _, _)
            | Element::Image(_, _, _)
            | Element::ImageRegion(_) => {}
        }
    }
}

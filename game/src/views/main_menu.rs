use engine::ui::{
    Element, Event, Key, View, RouteTarget, Font, Cmd,
    paint::PaintCtx,
};
use std::sync::Arc;

pub struct MainMenuView {
    pub selected: usize,
    items: Arc<Vec<(String, RouteTarget)>>,
}

impl MainMenuView {
    pub fn new() -> Self {
        Self {
            selected: 0,
            items: Arc::new(vec![
                ("SJ3 WORLD CUP".to_string(), RouteTarget::Play(1)),
                ("TEAM CUP".to_string(), RouteTarget::Play(1)),
                ("FOUR HILLS TOUR".to_string(), RouteTarget::Play(1)),
                ("KING OF THE HILL".to_string(), RouteTarget::Play(1)),
                ("PRACTISE".to_string(), RouteTarget::Play(1)),
                ("CUSTOM WORLD CUP".to_string(), RouteTarget::Play(1)),
                ("REPLAYS".to_string(), RouteTarget::Play(1)),
                ("PROFILES".to_string(), RouteTarget::Play(1)),
                ("SETTINGS".to_string(), RouteTarget::Play(1)),
                ("HILL RECORDS".to_string(), RouteTarget::Play(1)),
                ("EXIT".to_string(), RouteTarget::MainMenu),
            ]),
        }
    }

    pub fn with_items(items: Vec<(String, RouteTarget)>) -> Self {
        Self { selected: 0, items: Arc::new(items) }
    }
}

impl View for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = vec![];

        for (i, (label, route)) in self.items.iter().enumerate() {
            els.push(Element::Button {
                text: label.clone(),
                x: 20,
                y: 30 + (i as i32) * 12,
                selected: i == self.selected,
                cmd: Cmd::Navigate(route.clone()),
            });
        }

        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Up) => {
                self.selected = self.selected.saturating_sub(1);
                None
            }
            Event::Keyboard(Key::Down) => {
                if self.selected < self.items.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            Event::Keyboard(Key::Enter) | Event::Keyboard(Key::Char(' ')) => {
                let target = self.items[self.selected].1.clone();
                match self.items[self.selected].0.as_str() {
                    "EXIT" => {
                        std::process::exit(0);
                    }
                    _ => Some(target),
                }
            }
            Event::Keyboard(Key::Escape) => None,
            Event::Keyboard(Key::Char(c)) => {
                if let Some(idx) = c.to_digit(10) {
                    let idx = idx as usize - 1;
                    if idx < self.items.len() {
                        self.selected = idx;
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn route(&self) -> Option<RouteTarget> {
        Some(RouteTarget::MainMenu)
    }

    fn paint(&self, ctx: &mut PaintCtx, _sprites: &[Vec<u8>], font: &Font) {
        for (i, (label, _)) in self.items.iter().enumerate() {
            let y = 30 + (i as i32) * 12;

            if i == self.selected {
                ctx.fill_rect(10, y - 1, 150, 10, 246);
            }

            font.blit_string(ctx.pixels, ctx.width, label, 20, y);
        }
    }
}
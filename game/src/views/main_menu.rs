use engine::ui;
use engine::ui::{Cmd, Element, Event, Key, View, RouteTarget};
use std::sync::Arc;

pub struct MainMenuView {
    selected: usize,
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
                ("EXIT".to_string(), RouteTarget::Quit),
            ]),
        }
    }

    pub fn with_items(items: Vec<(String, RouteTarget)>) -> Self {
        Self { selected: 0, items: Arc::new(items) }
    }
}

impl View for MainMenuView {
    fn elements(&self) -> Vec<Element> {
        let mut els = ui![];
        for (i, (label, route)) in self.items.iter().enumerate() {
            let cmd = match route {
                RouteTarget::Quit => Cmd::Quit,
                _ => Cmd::Navigate(route.clone()),
            };
            els.push(ui::Element::button(
                label.clone(),
                20,
                30 + (i as i32) * 12,
                i == self.selected,
                cmd,
            ));
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
                match self.items[self.selected].1 {
                    RouteTarget::Quit => std::process::exit(0),
                    _ => Some(self.items[self.selected].1.clone()),
                }
            }
            Event::Keyboard(Key::Char(c)) => {
                if let Some(idx) = c.to_digit(10) {
                    let idx = idx as usize - 1;
                    if idx < self.items.len() {
                        self.selected = idx;
                    }
                }
                None
            }
            Event::Keyboard(Key::Escape) => None,
            _ => None,
        }
    }

    fn route(&self) -> Option<RouteTarget> {
        Some(RouteTarget::MainMenu)
    }
}
use engine::ui::{Element, Event, Key};
use crate::parsers::langbase::LangBase;

pub struct MenuItem {
    pub num: u8,
    pub label: usize,
}

pub struct Menu {
    selected: usize,
}

impl Menu {
    pub fn new() -> Self {
        Self { selected: 1 }
    }

    pub fn reset(&mut self) {
        self.selected = 1;
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn elements(
        &self,
        x: i32, y: i32,
        item_w: i32, item_h: i32,
        items: &[MenuItem],
        langbase: &LangBase,
        fontcolor: u8,
        boxcolor: u8,
    ) -> Vec<Element> {
        let mut els = Vec::with_capacity(items.len() + 1);

        for (i, item) in items.iter().enumerate() {
            let iy = y + (i as i32) * item_h;
            els.push(Element::text_color(
                format!("{} - {}", item.num, langbase.lstr(item.label)),
                x, iy, fontcolor,
            ));
        }

        let bx = x - 6;
        let by = y - 3 + ((self.selected - 1) as i32) * item_h;
        els.push(Element::box_(bx, by, item_w, item_h, boxcolor));

        els
    }

    pub fn handle_event(&mut self, event: &Event, num_items: usize) -> Option<usize> {
        match event {
            Event::Keyboard(Key::Up) if self.selected > 1 => {
                self.selected -= 1;
                None
            }
            Event::Keyboard(Key::Up) => Some(0),
            Event::Keyboard(Key::Down) if self.selected < num_items => {
                self.selected += 1;
                None
            }
            Event::Keyboard(Key::Down) => Some(0),
            Event::Keyboard(Key::Enter) | Event::Keyboard(Key::Char(' ')) => {
                Some(self.selected)
            }
            Event::Keyboard(Key::Char(c)) => {
                if let Some(d) = c.to_digit(10) {
                    let n = d as usize;
                    if n >= 1 && n <= num_items {
                        self.selected = n;
                        Some(n)
                    } else if n == 0 {
                        Some(0)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Event::Keyboard(Key::Escape) => Some(0),
            _ => None,
        }
    }
}

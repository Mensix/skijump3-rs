use engine::ui::{Element, Event, Key, Component};
use crate::parsers::langbase::LangBase;

pub struct MenuItem {
    pub num: u8,
    pub label: usize,
    pub y_off: i32,
}

pub struct Menu {
    selected: usize,
    navigable: usize,
    x: i32,
    y: i32,
    item_w: i32,
    item_h: i32,
    items: Vec<MenuItem>,
    langbase: LangBase,
    fontcolor: u8,
    boxcolor: u8,
}

impl Menu {
    pub fn new(
        x: i32, y: i32, item_w: i32, item_h: i32,
        items: Vec<MenuItem>, langbase: &LangBase,
        fontcolor: u8, boxcolor: u8,
    ) -> Self {
        let navigable = items.len();
        Self {
            selected: 1,
            navigable,
            x, y, item_w, item_h,
            items,
            langbase: langbase.clone(),
            fontcolor, boxcolor,
        }
    }

    pub fn with_navigable(mut self, n: usize) -> Self {
        self.navigable = n;
        self
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn reset(&mut self) {
        self.selected = 1;
    }

    pub fn num_items(&self) -> usize {
        self.items.len()
    }
}

impl Component for Menu {
    fn elements(&self) -> Vec<Element> {
        let mut els = Vec::with_capacity(self.items.len() + 1);

        for (i, item) in self.items.iter().enumerate() {
            let iy = self.y + 1 + (i as i32) * self.item_h + item.y_off;
            els.push(Element::text_color(
                format!("{} - {}", item.num, self.langbase.lstr(item.label)),
                self.x, iy, self.fontcolor,
            ));
        }

        let bx = self.x - 6;
        let idx = (self.selected - 1).min(self.items.len() - 1);
        let by = self.y - 3 + (idx as i32) * self.item_h + self.items[idx].y_off;
        els.push(Element::box_(bx, by, self.item_w + 1, self.item_h + 1, self.boxcolor));

        els
    }

    fn handle_event(&mut self, event: &Event) -> Option<usize> {
        match event {
            Event::Keyboard(Key::Up) if self.selected > 1 => {
                self.selected -= 1;
                None
            }
            Event::Keyboard(Key::Up) => {
                self.selected = self.navigable;
                None
            }
            Event::Keyboard(Key::Down) if self.selected < self.navigable => {
                self.selected += 1;
                None
            }
            Event::Keyboard(Key::Down) => {
                self.selected = 1;
                None
            }
            Event::Keyboard(Key::Enter) | Event::Keyboard(Key::Char(' ')) => {
                Some(self.selected)
            }
            Event::Keyboard(Key::Char(c)) => {
                if let Some(d) = c.to_digit(10) {
                    match d {
                        0 => Some(0),
                        n if (n as usize) <= self.navigable => {
                            self.selected = n as usize;
                            None
                        }
                        _ => None,
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

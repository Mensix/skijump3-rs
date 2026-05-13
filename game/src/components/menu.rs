use crate::parsers::langbase::LangBase;
use engine::ui::{Component, Element, Event, Key};
use std::rc::Rc;

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
    langbase: Rc<LangBase>,
    fontcolor: u8,
    boxcolor: u8,
    show_labels: bool,
    show_box: bool,
}

impl Menu {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: i32,
        y: i32,
        item_w: i32,
        item_h: i32,
        items: Vec<MenuItem>,
        langbase: &Rc<LangBase>,
        fontcolor: u8,
        boxcolor: u8,
    ) -> Self {
        let navigable = items.len();
        Self {
            selected: 0,
            navigable,
            x,
            y,
            item_w,
            item_h,
            items,
            langbase: Rc::clone(langbase),
            fontcolor,
            boxcolor,
            show_labels: true,
            show_box: true,
        }
    }

    pub fn with_navigable(mut self, n: usize) -> Self {
        self.navigable = n;
        self
    }

    pub fn with_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn with_box(mut self, show: bool) -> Self {
        self.show_box = show;
        self
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn reset(&mut self) {
        self.selected = 0;
    }

    pub fn set_selected(&mut self, idx: usize) {
        self.selected = idx.min(self.items.len().saturating_sub(1));
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl Component for Menu {
    type Action = usize;

    fn elements(&self) -> Vec<Element> {
        let mut els = Vec::with_capacity(self.items.len() + 1);

        if self.show_labels {
            for (i, item) in self.items.iter().enumerate() {
                let iy = self.y + 1 + (i as i32) * self.item_h + item.y_off;
                els.push(Element::text_color(
                    format!("{} - {}", item.num, self.langbase.lstr(item.label)),
                    self.x,
                    iy,
                    self.fontcolor,
                ));
            }
        }

        if self.show_box {
            let bx = self.x - 6;
            let idx = self.selected.min(self.items.len() - 1);
            let by = self.y - 3 + (idx as i32) * self.item_h + self.items[idx].y_off;
            els.push(Element::box_(
                bx,
                by,
                self.item_w + 1,
                self.item_h + 1,
                self.boxcolor,
            ));
        }

        els
    }

    fn handle_event(&mut self, event: &Event) -> Option<usize> {
        match event {
            Event::Keyboard(Key::Up) if self.selected > 0 => {
                self.selected -= 1;
                None
            }
            Event::Keyboard(Key::Up) => {
                self.selected = self.navigable - 1;
                None
            }
            Event::Keyboard(Key::Down) if self.selected + 1 < self.navigable => {
                self.selected += 1;
                None
            }
            Event::Keyboard(Key::Down) => {
                self.selected = 0;
                None
            }
            Event::Keyboard(Key::Enter) | Event::Keyboard(Key::Char(' ')) => {
                // return 1-shifted: Some(0) = back sentinel, Some(1..) = selected + 1
                Some(self.selected + 1)
            }
            Event::Keyboard(Key::Char(c)) => {
                if let Some(d) = c.to_digit(10) {
                    let n = d as usize;
                    if n >= 1 && n <= self.navigable {
                        self.selected = n - 1;
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

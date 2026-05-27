use crate::text::lang::LangBase;
use engine::color::Rgba;
use engine::ui::{Component, Element, Event, Key, SelectionState};
use std::rc::Rc;

pub struct MenuItem {
    pub num: u8,
    pub label: usize,
    pub y_off: i32,
}

impl MenuItem {
    #[must_use]
    pub const fn new(num: u8, label: usize) -> Self {
        Self {
            num,
            label,
            y_off: 0,
        }
    }

    #[must_use]
    pub const fn with_y(num: u8, label: usize, y_off: i32) -> Self {
        Self { num, label, y_off }
    }
}

pub struct Menu {
    selection: SelectionState,
    x: i32,
    y: i32,
    item_w: i32,
    item_h: i32,
    items: Vec<MenuItem>,
    langbase: Rc<LangBase>,
    fontcolor: Rgba,
    boxcolor: Rgba,
    show_labels: bool,
    show_box: bool,
    exit_item: bool,
    exit_label_idx: usize,
    exit_y_off: i32,
}

impl Menu {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        x: i32,
        y: i32,
        item_w: i32,
        item_h: i32,
        items: Vec<MenuItem>,
        langbase: &Rc<LangBase>,
        fontcolor: Rgba,
        boxcolor: Rgba,
    ) -> Self {
        Self {
            selection: SelectionState::new(items.len()),
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
            exit_item: false,
            exit_label_idx: 0,
            exit_y_off: 0,
        }
    }

    #[must_use]
    pub const fn with_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    #[must_use]
    pub const fn with_box(mut self, show: bool) -> Self {
        self.show_box = show;
        self
    }

    #[must_use]
    pub fn with_exit(mut self, label_idx: usize, y_off: i32) -> Self {
        self.exit_item = true;
        self.exit_label_idx = label_idx;
        self.exit_y_off = y_off;
        self.selection
            .resize(self.items.len() + usize::from(self.exit_item));
        self
    }

    #[must_use]
    pub fn selected(&self) -> usize {
        self.selection.selected()
    }

    pub fn reset(&mut self) {
        self.selection.set_selected(0);
    }

    pub fn set_selected(&mut self, idx: usize) {
        self.selection.set_selected(idx);
    }

    #[must_use]
    pub const fn item_count(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub const fn has_exit(&self) -> bool {
        self.exit_item
    }
}

impl Component for Menu {
    type Action = usize;

    fn elements(&self) -> Vec<Element> {
        let mut els = Vec::with_capacity(self.items.len() + 1);

        if self.show_labels {
            for (i, item) in self.items.iter().enumerate() {
                let iy = self.y + 1 + (i as i32) * self.item_h + item.y_off;
                els.push(Element::text(
                    format!("{} - {}", item.num, self.langbase.lstr(item.label)),
                    self.x,
                    iy,
                    self.fontcolor,
                    false,
                ));
            }
            if self.exit_item {
                let iy = self.y + 1 + (self.items.len() as i32) * self.item_h + self.exit_y_off;
                els.push(Element::text(
                    format!("0. {}", self.langbase.lstr(self.exit_label_idx)),
                    self.x,
                    iy,
                    self.fontcolor,
                    false,
                ));
            }
        }

        if self.show_box {
            let bx = self.x - 6;
            let sel = self.selection.selected();
            let (row, y_off) = if self.exit_item && sel == self.items.len() {
                (self.items.len() as i32, self.exit_y_off)
            } else {
                let idx = sel.min(self.items.len().saturating_sub(1));
                (idx as i32, self.items[idx].y_off)
            };
            let by = self.y - 3 + row * self.item_h + y_off;
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
        let total = self.items.len() + usize::from(self.exit_item);
        match event {
            Event::Keyboard(Key::Up) => {
                self.selection.up();
                None
            }
            Event::Keyboard(Key::Down) => {
                self.selection.down();
                None
            }
            Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                let sel = self.selection.selected();
                if self.exit_item && sel == self.items.len() {
                    Some(0)
                } else {
                    Some(sel + 1)
                }
            }
            Event::Keyboard(Key::Char(c)) => {
                if let Some(d) = c.to_digit(10) {
                    let n = d as usize;
                    if n >= 1 && n <= total {
                        self.selection.set_selected(n - 1);
                        Some(n)
                    } else if n == 0 {
                        if self.exit_item {
                            self.selection.set_selected(self.items.len());
                        }
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

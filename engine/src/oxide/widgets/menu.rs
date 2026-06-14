use crate::color::Rgba;
use crate::oxide::input::{Key, UiEvent};
use crate::oxide::paint::PaintCx;
use crate::oxide::widget::{EventCx, Widget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub number: u8,
    pub label: String,
    pub y_offset: i32,
}

impl MenuItem {
    #[must_use]
    pub fn new(number: u8, label: impl Into<String>) -> Self {
        Self {
            number,
            label: label.into(),
            y_offset: 0,
        }
    }

    #[must_use]
    pub fn with_y(mut self, y_offset: i32) -> Self {
        self.y_offset = y_offset;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PixelMenu {
    x: i32,
    y: i32,
    item_w: i32,
    item_h: i32,
    items: Vec<MenuItem>,
    selected: usize,
    font_color: Rgba,
    box_color: Rgba,
    show_labels: bool,
    show_box: bool,
    exit_label: Option<(String, i32)>,
}

impl PixelMenu {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        x: i32,
        y: i32,
        item_w: i32,
        item_h: i32,
        items: Vec<MenuItem>,
        font_color: Rgba,
        box_color: Rgba,
    ) -> Self {
        Self {
            x,
            y,
            item_w,
            item_h,
            items,
            selected: 0,
            font_color,
            box_color,
            show_labels: true,
            show_box: true,
            exit_label: None,
        }
    }

    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected.min(self.total_items().saturating_sub(1));
    }

    pub fn set_show_box(&mut self, show: bool) {
        self.show_box = show;
    }

    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }

    #[must_use]
    pub const fn item_count(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub const fn has_exit(&self) -> bool {
        self.exit_label.is_some()
    }

    #[must_use]
    pub const fn with_labels(mut self, show_labels: bool) -> Self {
        self.show_labels = show_labels;
        self
    }

    #[must_use]
    pub const fn with_box(mut self, show_box: bool) -> Self {
        self.show_box = show_box;
        self
    }

    #[must_use]
    pub fn with_exit(mut self, label: impl Into<String>, y_offset: i32) -> Self {
        self.exit_label = Some((label.into(), y_offset));
        self.set_selected(self.selected);
        self
    }

    #[must_use]
    fn total_items(&self) -> usize {
        self.items.len() + usize::from(self.exit_label.is_some())
    }

    fn move_up(&mut self) {
        let total = self.total_items();
        self.selected = if self.selected == 0 {
            total.saturating_sub(1)
        } else {
            self.selected - 1
        };
    }

    fn move_down(&mut self) {
        let total = self.total_items();
        self.selected = if self.selected + 1 >= total {
            0
        } else {
            self.selected + 1
        };
    }
}

impl Widget for PixelMenu {
    type Message = usize;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        let total = self.total_items();
        let msg = match event {
            UiEvent::KeyDown(Key::Up) => {
                self.move_up();
                None
            }
            UiEvent::KeyDown(Key::Down) => {
                self.move_down();
                None
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                if self.exit_label.is_some() && self.selected == self.items.len() {
                    Some(0)
                } else {
                    Some(self.selected + 1)
                }
            }
            UiEvent::Text(c) if c.is_ascii_digit() => {
                let d = c.to_digit(10).unwrap_or(0) as usize;
                let total = self.total_items();
                if d >= 1 && d <= total {
                    self.set_selected(d - 1);
                } else if d == 0 {
                    self.set_selected(total.saturating_sub(1));
                }
                cx.consume();
                None
            }
            UiEvent::Text(c) if c.is_ascii_digit() => {
                let n = c.to_digit(10).map_or(0, |d| d as usize);
                if n >= 1 && n <= total {
                    self.selected = n - 1;
                    Some(n)
                } else if n == 0 {
                    if self.exit_label.is_some() {
                        self.selected = self.items.len();
                    }
                    Some(0)
                } else {
                    None
                }
            }
            UiEvent::KeyDown(Key::Escape) => Some(0),
            _ => None,
        };
        if msg.is_some() || matches!(event, UiEvent::KeyDown(Key::Up | Key::Down)) {
            cx.consume();
        }
        msg
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        if self.show_labels {
            for (i, item) in self.items.iter().enumerate() {
                let y = self.y + 1 + (i as i32) * self.item_h + item.y_offset;
                cx.text(
                    (self.x, y),
                    self.font_color,
                    format!("{} - {}", item.number, item.label),
                );
            }
            if let Some((label, y_offset)) = &self.exit_label {
                let y = self.y + 1 + (self.items.len() as i32) * self.item_h + *y_offset;
                cx.text((self.x, y), self.font_color, format!("0. {label}"));
            }
        }

        if self.show_box && self.total_items() > 0 {
            let selected = self.selected.min(self.total_items().saturating_sub(1));
            let (row, y_offset) = if self.exit_label.is_some() && selected == self.items.len() {
                let y_offset = self.exit_label.as_ref().map_or(0, |(_, offset)| *offset);
                (self.items.len() as i32, y_offset)
            } else {
                let idx = selected.min(self.items.len().saturating_sub(1));
                (idx as i32, self.items[idx].y_offset)
            };
            cx.stroke(
                (
                    self.x - 6,
                    self.y - 3 + row * self.item_h + y_offset,
                    self.item_w + 1,
                    self.item_h + 1,
                ),
                self.box_color,
            );
        }
    }
}

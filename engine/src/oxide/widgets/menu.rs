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

#[cfg(test)]
mod tests {
    use super::{MenuItem, PixelMenu};
    use crate::color::Rgba;
    use crate::oxide::input::Key;
    use crate::oxide::input::UiEvent;
    use crate::oxide::widget::{EventCx, Widget};

    fn menu_with_trailing() -> PixelMenu {
        PixelMenu::new(
            0,
            0,
            10,
            10,
            vec![
                MenuItem::new(0, ""),
                MenuItem::new(1, ""),
                MenuItem::new(2, ""),
            ],
            Rgba::rgb(255, 255, 255),
            Rgba::rgb(255, 255, 255),
        )
        .trailing("", 0)
    }

    fn menu_return_index() -> PixelMenu {
        PixelMenu::new(
            0,
            0,
            10,
            10,
            vec![
                MenuItem::new(0, ""),
                MenuItem::new(0, ""),
                MenuItem::new(0, ""),
            ],
            Rgba::rgb(255, 255, 255),
            Rgba::rgb(255, 255, 255),
        )
        .trailing("", 0)
        .with_return_index(true)
    }

    #[test]
    fn digit_one_selects_first_visible_item() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::Text('1'));

        assert_eq!(menu.selected(), 0);
        assert_eq!(msg, Some(0));
        assert!(cx.is_consumed());
    }

    #[test]
    fn digit_zero_selects_trailing_item() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::Text('0'));

        assert_eq!(menu.selected(), menu.item_count());
        assert_eq!(msg, Some(0));
        assert!(cx.is_consumed());
    }

    #[test]
    fn up_from_first_selects_trailing_item() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Up));

        assert_eq!(menu.selected(), menu.item_count());
        assert_eq!(msg, None);
        assert!(cx.is_consumed());
    }

    #[test]
    fn down_from_trailing_goes_to_first() {
        let mut menu = menu_with_trailing();
        menu.set_selected(menu.item_count());
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Down));

        assert_eq!(menu.selected(), 0);
        assert_eq!(msg, None);
        assert!(cx.is_consumed());
    }

    #[test]
    fn home_selects_first() {
        let mut menu = menu_with_trailing();
        menu.set_selected(menu.item_count());
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Home));

        assert_eq!(menu.selected(), 0);
        assert_eq!(msg, None);
        assert!(cx.is_consumed());
    }

    #[test]
    fn end_selects_last_including_trailing() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::End));

        assert_eq!(menu.selected(), menu.item_count());
        assert_eq!(msg, None);
        assert!(cx.is_consumed());
    }

    #[test]
    fn enter_submits_selected_item_number() {
        let mut menu = menu_with_trailing();
        menu.set_selected(1);
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Enter));

        assert_eq!(msg, Some(1));
        assert!(cx.is_consumed());
    }

    #[test]
    fn enter_on_trailing_returns_zero() {
        let mut menu = menu_with_trailing();
        menu.set_selected(menu.item_count());
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Enter));

        assert_eq!(msg, Some(0));
        assert!(cx.is_consumed());
    }

    #[test]
    fn space_submits_like_enter() {
        let mut menu = menu_with_trailing();
        menu.set_selected(1);
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::Text(' '));

        assert_eq!(msg, Some(1));
        assert!(cx.is_consumed());
    }

    #[test]
    fn escape_returns_zero() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Escape));

        assert_eq!(msg, Some(0));
        assert!(cx.is_consumed());
    }

    #[test]
    fn return_index_mode_returns_selected_index() {
        let mut menu = menu_return_index();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::Text('1'));

        assert_eq!(menu.selected(), 0);
        assert_eq!(msg, Some(0));
        assert!(cx.is_consumed());
    }

    #[test]
    fn return_index_enter_second_item() {
        let mut menu = menu_return_index();
        menu.set_selected(1);
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Enter));

        assert_eq!(msg, Some(1));
        assert!(cx.is_consumed());
    }

    #[test]
    fn return_index_trailing_returns_item_count() {
        let mut menu = menu_return_index();
        menu.set_selected(menu.item_count());
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Enter));

        assert_eq!(msg, Some(menu.item_count()));
        assert!(cx.is_consumed());
    }

    #[test]
    fn return_index_digit_zero_selects_trailing() {
        let mut menu = menu_return_index();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::Text('0'));

        assert_eq!(menu.selected(), menu.item_count());
        assert_eq!(msg, Some(menu.item_count()));
        assert!(cx.is_consumed());
    }

    #[test]
    fn return_index_f10_selects_trailing_and_returns_item_count() {
        let mut menu = menu_return_index();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::F10));

        assert_eq!(menu.selected(), menu.item_count());
        assert_eq!(msg, Some(menu.item_count()));
        assert!(cx.is_consumed());
    }

    #[test]
    fn f1_selects_first_item() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::F1));

        assert_eq!(menu.selected(), 0);
        assert_eq!(msg, Some(0));
        assert!(cx.is_consumed());
    }

    #[test]
    fn f10_selects_trailing_if_exists() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::F10));

        assert_eq!(menu.selected(), menu.item_count());
        assert_eq!(msg, Some(0));
        assert!(cx.is_consumed());
    }

    #[test]
    fn f10_without_trailing_does_nothing() {
        let mut menu = PixelMenu::new(
            0,
            0,
            10,
            10,
            vec![MenuItem::new(0, ""), MenuItem::new(1, "")],
            Rgba::rgb(255, 255, 255),
            Rgba::rgb(255, 255, 255),
        );
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::F10));

        assert_eq!(msg, None);
        assert!(!cx.is_consumed());
    }

    #[test]
    fn digit_greater_than_item_count_does_nothing() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::Text('9'));

        assert_eq!(msg, None);
        assert!(cx.is_consumed());
    }

    #[test]
    fn left_arrow_moves_up() {
        let mut menu = menu_with_trailing();
        menu.set_selected(2);
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Left));

        assert_eq!(menu.selected(), 1);
        assert_eq!(msg, None);
        assert!(cx.is_consumed());
    }

    #[test]
    fn right_arrow_moves_down() {
        let mut menu = menu_with_trailing();
        let mut cx = EventCx::default();

        let msg = menu.event(&mut cx, UiEvent::KeyDown(Key::Right));

        assert_eq!(menu.selected(), 1);
        assert_eq!(msg, None);
        assert!(cx.is_consumed());
    }
}

impl MenuItem {
    pub fn new(number: u8, label: impl Into<String>) -> Self {
        Self {
            number,
            label: label.into(),
            y_offset: 0,
        }
    }

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
    trailing: Option<(String, i32)>,
    return_index: bool,
}

impl PixelMenu {
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
            trailing: None,
            return_index: false,
        }
    }

    pub fn with_return_index(mut self, val: bool) -> Self {
        self.return_index = val;
        self
    }

    pub const fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_action(&self) -> usize {
        self.submit_selected()
    }

    pub fn is_trailing_selected(&self) -> bool {
        self.trailing.is_some() && self.selected == self.items.len()
    }

    pub const fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected.min(self.total_items().saturating_sub(1));
    }

    pub fn set_show_box(&mut self, show: bool) {
        self.show_box = show;
    }

    pub const fn item_count(&self) -> usize {
        self.items.len()
    }

    pub const fn has_trailing(&self) -> bool {
        self.trailing.is_some()
    }

    pub fn total_items(&self) -> usize {
        self.items.len() + usize::from(self.trailing.is_some())
    }

    pub fn with_labels(mut self, show_labels: bool) -> Self {
        self.show_labels = show_labels;
        self
    }

    pub const fn with_box(mut self, show_box: bool) -> Self {
        self.show_box = show_box;
        self
    }

    pub fn trailing(mut self, label: impl Into<String>, gap: i32) -> Self {
        self.trailing = Some((label.into(), gap));
        self.set_selected(self.selected);
        self
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

    fn select_last(&mut self) {
        self.selected = self.total_items().saturating_sub(1);
    }

    fn submit_selected(&self) -> usize {
        if self.return_index {
            self.selected
        } else if self.trailing.is_some() && self.selected == self.items.len() {
            0
        } else {
            self.items[self.selected].number as usize
        }
    }

    pub fn function_key_index(key: Key) -> Option<usize> {
        match key {
            Key::F1 => Some(1),
            Key::F2 => Some(2),
            Key::F3 => Some(3),
            Key::F4 => Some(4),
            Key::F5 => Some(5),
            Key::F6 => Some(6),
            Key::F7 => Some(7),
            Key::F8 => Some(8),
            Key::F9 => Some(9),
            Key::F10 => Some(10),
            _ => None,
        }
    }

    fn trailing_y_offset(&self) -> i32 {
        self.trailing
            .as_ref()
            .map_or(0, |(_, gap)| self.items.len() as i32 * self.item_h + *gap)
    }

    pub fn item_y(&self, sel: usize) -> i32 {
        if self.trailing.is_some() && sel == self.items.len() {
            self.y + self.trailing_y_offset()
        } else {
            let idx = sel.min(self.items.len().saturating_sub(1));
            self.y + idx as i32 * self.item_h + self.items[idx].y_offset
        }
    }
}

impl Widget for PixelMenu {
    type Message = usize;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        let msg = match event {
            UiEvent::KeyDown(Key::Up | Key::Left) => {
                self.move_up();
                None
            }
            UiEvent::KeyDown(Key::Down | Key::Right) => {
                self.move_down();
                None
            }
            UiEvent::KeyDown(Key::Home) => {
                self.set_selected(0);
                None
            }
            UiEvent::KeyDown(Key::End) => {
                self.select_last();
                None
            }
            UiEvent::KeyDown(key) if Self::function_key_index(key).is_some() => {
                let index = Self::function_key_index(key).unwrap();
                if index == 10 && self.trailing.is_some() {
                    self.select_last();
                    Some(self.submit_selected())
                } else if index >= 1 && index <= self.items.len() {
                    self.set_selected(index - 1);
                    Some(self.submit_selected())
                } else {
                    None
                }
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => Some(self.submit_selected()),
            UiEvent::Text(c) if c.is_ascii_digit() => {
                let d = c.to_digit(10).unwrap_or(0) as usize;
                let total = self.total_items();
                if d >= 1 && d <= total {
                    self.set_selected(d - 1);
                    cx.consume();
                    Some(self.submit_selected())
                } else if d == 0 {
                    self.set_selected(total.saturating_sub(1));
                    cx.consume();
                    Some(self.submit_selected())
                } else {
                    cx.consume();
                    None
                }
            }
            UiEvent::Text(c) if matches!(c, 'A'..='L' | 'a'..='l') => {
                let index = c.to_ascii_uppercase() as usize - 'A' as usize + 10;
                if index >= 1 && index <= self.total_items() {
                    self.set_selected(index - 1);
                    cx.consume();
                    Some(self.submit_selected())
                } else {
                    cx.consume();
                    None
                }
            }
            UiEvent::KeyDown(Key::Escape) => Some(0),
            _ => None,
        };
        if msg.is_some()
            || matches!(
                event,
                UiEvent::KeyDown(
                    Key::Up | Key::Down | Key::Left | Key::Right | Key::Home | Key::End
                )
            )
        {
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
        }

        if let Some((label, _gap)) = &self.trailing {
            let y = self.y + 1 + self.trailing_y_offset();
            cx.text((self.x, y), self.font_color, format!("0. {label}"));
        }

        if self.show_box && self.total_items() > 0 {
            let selected = self.selected.min(self.total_items().saturating_sub(1));
            let item_y = self.item_y(selected);
            cx.stroke(
                (self.x - 6, item_y - 3, self.item_w + 1, self.item_h + 1),
                self.box_color,
            );
        }
    }
}

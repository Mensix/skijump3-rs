use engine::color::Rgba;
pub(crate) use engine::oxide::{
    Font, Glyph, Modifiers, PointBatches, Rect as EngineRect, StaticImage,
};

#[derive(Default)]
pub(crate) struct EventCx {
    consumed: bool,
}

impl EventCx {
    pub(crate) fn consume(&mut self) {
        self.consumed = true;
    }

    pub(crate) const fn is_consumed(&self) -> bool {
        self.consumed
    }
}

pub(crate) trait Widget {
    type Message;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message>;
}

#[derive(Debug)]
pub(crate) struct Blinker {
    counter: std::cell::Cell<u32>,
}

impl Blinker {
    pub(crate) fn new() -> Self {
        Self {
            counter: std::cell::Cell::new(0),
        }
    }

    pub(crate) fn reset(&self) {
        self.counter.set(0);
    }

    pub(crate) fn visible(&self, on: u32, off: u32) -> bool {
        let counter = self.counter.get();
        self.counter.set(counter + 1);
        counter % (on + off) < on
    }
}

impl Default for Blinker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextEditState {
    buffer: String,
    cursor: usize,
    max_chars: usize,
}

impl TextEditState {
    pub(crate) fn new(initial: String, max_chars: usize) -> Self {
        let cursor = initial.chars().count();
        Self {
            buffer: initial,
            cursor,
            max_chars,
        }
    }

    pub(crate) fn buffer(&self) -> &str {
        &self.buffer
    }

    pub(crate) fn cursor_byte(&self) -> usize {
        self.buffer
            .char_indices()
            .nth(self.cursor)
            .map_or(self.buffer.len(), |(index, _)| index)
    }

    pub(crate) fn insert(&mut self, c: char) -> bool {
        if self.buffer.chars().count() >= self.max_chars {
            return false;
        }
        let byte_index = self.cursor_byte();
        self.buffer.insert(byte_index, c);
        self.cursor += 1;
        true
    }

    pub(crate) fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let byte_index = self
            .buffer
            .char_indices()
            .nth(self.cursor - 1)
            .map(|(index, _)| index);
        let Some(byte_index) = byte_index else {
            return false;
        };
        self.buffer.remove(byte_index);
        self.cursor -= 1;
        true
    }

    pub(crate) fn set_buffer(&mut self, text: String) {
        self.buffer = text;
        if self.buffer.chars().count() > self.max_chars {
            let end = self
                .buffer
                .char_indices()
                .nth(self.max_chars)
                .map_or(self.buffer.len(), |(index, _)| index);
            self.buffer.truncate(end);
        }
        self.cursor = self.cursor.min(self.buffer.chars().count());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MenuItem {
    pub(crate) number: u8,
    pub(crate) label: String,
    pub(crate) y_offset: i32,
    pub(crate) gap_before: i32,
}

impl MenuItem {
    pub(crate) fn new(number: u8, label: impl Into<String>) -> Self {
        Self {
            number,
            label: label.into(),
            y_offset: 0,
            gap_before: 0,
        }
    }

    pub(crate) fn with_y(mut self, y_offset: i32) -> Self {
        self.y_offset = y_offset;
        self
    }

    pub(crate) fn with_gap_before(mut self, gap_before: i32) -> Self {
        self.gap_before = gap_before;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuAction {
    Item(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct PixelMenu {
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
}

impl PixelMenu {
    pub(crate) fn new(
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
        }
    }

    pub(crate) const fn selected(&self) -> usize {
        self.selected
    }

    pub(crate) const fn x(&self) -> i32 {
        self.x
    }

    pub(crate) const fn y(&self) -> i32 {
        self.y
    }

    pub(crate) const fn item_width(&self) -> i32 {
        self.item_w
    }

    pub(crate) const fn item_height(&self) -> i32 {
        self.item_h
    }

    pub(crate) fn items(&self) -> &[MenuItem] {
        &self.items
    }

    pub(crate) const fn font_color(&self) -> Rgba {
        self.font_color
    }

    pub(crate) const fn box_color(&self) -> Rgba {
        self.box_color
    }

    pub(crate) const fn show_labels(&self) -> bool {
        self.show_labels
    }

    pub(crate) const fn show_box(&self) -> bool {
        self.show_box
    }

    pub(crate) fn set_selected(&mut self, selected: usize) {
        self.selected = selected.min(self.item_count().saturating_sub(1));
    }

    pub(crate) fn set_items(&mut self, items: Vec<MenuItem>) {
        self.items = items;
        self.set_selected(self.selected);
    }

    pub(crate) fn set_index_items(&mut self, count: usize) {
        self.set_items((0..count).map(|i| MenuItem::new(i as u8, "")).collect());
    }

    pub(crate) fn selected_action(&self) -> MenuAction {
        MenuAction::Item(self.selected)
    }

    pub(crate) fn event_action(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<MenuAction> {
        self.event(cx, event)?;
        Some(self.selected_action())
    }

    pub(crate) fn set_show_box(&mut self, show: bool) {
        self.show_box = show;
    }

    pub(crate) const fn item_count(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn with_labels(mut self, show_labels: bool) -> Self {
        self.show_labels = show_labels;
        self
    }

    pub(crate) const fn with_box(mut self, show_box: bool) -> Self {
        self.show_box = show_box;
        self
    }

    pub(crate) fn function_key_index(key: Key) -> Option<usize> {
        match key {
            Key::F1 => Some(0),
            Key::F2 => Some(1),
            Key::F3 => Some(2),
            Key::F4 => Some(3),
            Key::F5 => Some(4),
            Key::F6 => Some(5),
            Key::F7 => Some(6),
            Key::F8 => Some(7),
            Key::F9 => Some(8),
            Key::F10 => Some(9),
            _ => None,
        }
    }

    fn move_up(&mut self) {
        let total = self.item_count();
        self.selected = if self.selected == 0 {
            total.saturating_sub(1)
        } else {
            self.selected - 1
        };
    }

    fn move_down(&mut self) {
        let total = self.item_count();
        self.selected = if self.selected + 1 >= total {
            0
        } else {
            self.selected + 1
        };
    }

    fn submit_selected(&self) -> usize {
        self.items
            .get(self.selected)
            .map_or(0, |item| item.number as usize)
    }
}

impl Widget for PixelMenu {
    type Message = usize;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        if self.item_count() == 0 {
            if matches!(event, UiEvent::KeyDown(_) | UiEvent::Text(_)) {
                cx.consume();
            }
            return None;
        }
        let message = match event {
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
                self.set_selected(self.item_count().saturating_sub(1));
                None
            }
            UiEvent::KeyDown(key) if Self::function_key_index(key).is_some() => {
                let index = Self::function_key_index(key)?;
                if index < self.items.len() {
                    self.set_selected(index);
                    Some(self.submit_selected())
                } else {
                    None
                }
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => Some(self.submit_selected()),
            UiEvent::Text(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).map_or(0, |value| value as usize);
                let total = self.item_count();
                if digit >= 1 && digit <= total {
                    self.set_selected(digit - 1);
                    Some(self.submit_selected())
                } else if digit == 0 {
                    self.set_selected(total.saturating_sub(1));
                    Some(self.submit_selected())
                } else {
                    None
                }
            }
            UiEvent::Text(c) if matches!(c, 'A'..='L' | 'a'..='l') => {
                let index = c.to_ascii_uppercase() as usize - 'A' as usize + 10;
                if index >= 1 && index <= self.item_count() {
                    self.set_selected(index - 1);
                    Some(self.submit_selected())
                } else {
                    None
                }
            }
            UiEvent::KeyDown(Key::Escape) => None,
            _ => None,
        };
        if message.is_some()
            || matches!(
                event,
                UiEvent::KeyDown(
                    Key::Up | Key::Down | Key::Left | Key::Right | Key::Home | Key::End
                )
            )
            || matches!(event, UiEvent::Text(c) if c.is_ascii_digit() || matches!(c, 'A'..='L' | 'a'..='l'))
        {
            cx.consume();
        }
        message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TextInputMessage {
    Commit(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub(crate) struct TextInputParams {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) max_width: i32,
    pub(crate) initial: String,
    pub(crate) max_chars: usize,
    pub(crate) bg: Rgba,
    pub(crate) fg: Rgba,
    pub(crate) cursor: Rgba,
    pub(crate) font: Font,
}

#[derive(Debug)]
pub(crate) struct TextInput {
    x: i32,
    y: i32,
    max_width: i32,
    bg: Rgba,
    fg: Rgba,
    cursor: Rgba,
    font: Font,
    editor: TextEditState,
    blinker: Blinker,
}

impl TextInput {
    pub(crate) fn new(params: TextInputParams) -> Self {
        Self {
            x: params.x,
            y: params.y,
            max_width: params.max_width,
            bg: params.bg,
            fg: params.fg,
            cursor: params.cursor,
            font: params.font,
            editor: TextEditState::new(params.initial, params.max_chars),
            blinker: Blinker::new(),
        }
    }

    pub(crate) fn value(&self) -> &str {
        self.editor.buffer()
    }

    pub(crate) const fn x(&self) -> i32 {
        self.x
    }

    pub(crate) const fn y(&self) -> i32 {
        self.y
    }

    pub(crate) const fn max_width(&self) -> i32 {
        self.max_width
    }

    pub(crate) const fn bg(&self) -> Rgba {
        self.bg
    }

    pub(crate) const fn fg(&self) -> Rgba {
        self.fg
    }

    pub(crate) const fn cursor(&self) -> Rgba {
        self.cursor
    }

    pub(crate) fn cursor_visible(&self) -> bool {
        self.blinker.visible(11, 10)
    }
}

impl Widget for TextInput {
    type Message = TextInputMessage;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        self.blinker.reset();
        let message = match event {
            UiEvent::KeyDown(Key::Backspace) => {
                self.editor.backspace();
                None
            }
            UiEvent::KeyDown(Key::Delete) => {
                self.editor.set_buffer(String::new());
                None
            }
            UiEvent::KeyDown(Key::Enter) => {
                Some(TextInputMessage::Commit(self.value().to_string()))
            }
            UiEvent::KeyDown(Key::Escape) => Some(TextInputMessage::Cancel),
            UiEvent::Text(c) if c >= ' ' => {
                if self.font.string_width(self.value()) as i32 + 7 < self.max_width {
                    self.editor.insert(c);
                }
                None
            }
            _ => return None,
        };
        cx.consume();
        message
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorMessage {
    Commit(usize),
    Cancel,
}

#[derive(Debug, Clone)]
pub(crate) struct NumericSelectorParams {
    pub(crate) max: usize,
    pub(crate) value: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct NumericSelector {
    value: usize,
    max: usize,
    wrap: bool,
}

impl NumericSelector {
    pub(crate) fn new(params: NumericSelectorParams) -> Self {
        Self {
            value: params.value.min(params.max),
            max: params.max,
            wrap: true,
        }
    }

    pub(crate) const fn value(&self) -> usize {
        self.value
    }

    pub(crate) fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    fn decrement(&mut self, step: usize) {
        self.value = if self.wrap && self.value == 0 {
            self.max
        } else {
            self.value.saturating_sub(step)
        };
    }

    fn increment(&mut self, step: usize) {
        self.value = if self.wrap && self.value == self.max {
            0
        } else {
            (self.value + step).min(self.max)
        };
    }
}

impl Widget for NumericSelector {
    type Message = SelectorMessage;

    fn event(&mut self, cx: &mut EventCx, event: UiEvent) -> Option<Self::Message> {
        let message = match event {
            UiEvent::KeyDown(Key::Up | Key::Left) => {
                self.decrement(1);
                None
            }
            UiEvent::KeyDown(Key::Down | Key::Right) => {
                self.increment(1);
                None
            }
            UiEvent::KeyDown(Key::Home) => {
                self.value = 0;
                None
            }
            UiEvent::KeyDown(Key::End) => {
                self.value = self.max;
                None
            }
            UiEvent::KeyDown(Key::PageUp) => {
                self.decrement(10);
                None
            }
            UiEvent::KeyDown(Key::PageDown) => {
                self.increment(10);
                None
            }
            UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ') => {
                Some(SelectorMessage::Commit(self.value))
            }
            UiEvent::KeyDown(Key::Escape) => Some(SelectorMessage::Cancel),
            _ => return None,
        };
        cx.consume();
        message
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Insert,
    PageUp,
    PageDown,
    Kp5,
    Enter,
    Escape,
    Backspace,
    Delete,
    Tab,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UiEvent {
    KeyDown(Key),
    Text(char),
    TextWithModifiers(char, Modifiers),
    Quit,
    Tick,
}

impl From<engine::oxide::Key> for Key {
    fn from(key: engine::oxide::Key) -> Self {
        match key {
            engine::oxide::Key::Up => Self::Up,
            engine::oxide::Key::Down => Self::Down,
            engine::oxide::Key::Left => Self::Left,
            engine::oxide::Key::Right => Self::Right,
            engine::oxide::Key::Home => Self::Home,
            engine::oxide::Key::End => Self::End,
            engine::oxide::Key::Insert => Self::Insert,
            engine::oxide::Key::PageUp => Self::PageUp,
            engine::oxide::Key::PageDown => Self::PageDown,
            engine::oxide::Key::Kp5 => Self::Kp5,
            engine::oxide::Key::Enter => Self::Enter,
            engine::oxide::Key::Escape => Self::Escape,
            engine::oxide::Key::Backspace => Self::Backspace,
            engine::oxide::Key::Delete => Self::Delete,
            engine::oxide::Key::Tab => Self::Tab,
            engine::oxide::Key::F1 => Self::F1,
            engine::oxide::Key::F2 => Self::F2,
            engine::oxide::Key::F3 => Self::F3,
            engine::oxide::Key::F4 => Self::F4,
            engine::oxide::Key::F5 => Self::F5,
            engine::oxide::Key::F6 => Self::F6,
            engine::oxide::Key::F7 => Self::F7,
            engine::oxide::Key::F8 => Self::F8,
            engine::oxide::Key::F9 => Self::F9,
            engine::oxide::Key::F10 => Self::F10,
        }
    }
}

impl From<engine::oxide::UiEvent> for UiEvent {
    fn from(event: engine::oxide::UiEvent) -> Self {
        match event {
            engine::oxide::UiEvent::KeyDown(key) => Self::KeyDown(key.into()),
            engine::oxide::UiEvent::Text(ch) => Self::Text(ch),
            engine::oxide::UiEvent::TextWithModifiers(ch, modifiers) => {
                Self::TextWithModifiers(ch, modifiers)
            }
            engine::oxide::UiEvent::Quit => Self::Quit,
            engine::oxide::UiEvent::Tick => Self::Tick,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NavAction<R> {
    None,
    Navigate(R),
    Back,
    Quit,
}

impl<R> Default for NavAction<R> {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScreenBackground {
    NoneBlack,
    MainPng,
}

pub(crate) struct ScreenEventCx<R> {
    action: NavAction<R>,
    consumed: bool,
}

impl<R> Default for ScreenEventCx<R> {
    fn default() -> Self {
        Self {
            action: NavAction::None,
            consumed: false,
        }
    }
}

impl<R> ScreenEventCx<R> {
    pub(crate) fn consume(&mut self) {
        self.consumed = true;
    }

    pub(crate) fn navigate(&mut self, route: R) {
        self.action = NavAction::Navigate(route);
        self.consume();
    }

    pub(crate) fn back(&mut self) {
        self.action = NavAction::Back;
        self.consume();
    }

    pub(crate) fn quit(&mut self) {
        self.action = NavAction::Quit;
        self.consume();
    }

    pub(crate) const fn is_consumed(&self) -> bool {
        self.consumed
    }

    pub(crate) fn take_action(&mut self) -> NavAction<R> {
        std::mem::take(&mut self.action)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpriteMaterialData {
    pub(crate) id: u64,
    pub(crate) overrides: Vec<(u8, Rgba)>,
}

impl SpriteMaterialData {
    #[cfg(test)]
    pub(crate) fn get(&self, source: u8) -> Option<Rgba> {
        self.overrides
            .iter()
            .find(|(index, _)| *index == source)
            .map(|(_, color)| *color)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MenuItemRenderSnapshot {
    pub(crate) number: u8,
    pub(crate) label: String,
    pub(crate) y_offset: i32,
    pub(crate) gap_before: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MenuRenderSnapshot {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) item_w: i32,
    pub(crate) item_h: i32,
    pub(crate) items: Vec<MenuItemRenderSnapshot>,
    pub(crate) selected: usize,
    pub(crate) font_color: Rgba,
    pub(crate) box_color: Rgba,
    pub(crate) show_labels: bool,
    pub(crate) show_box: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextInputRenderSnapshot {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) max_width: i32,
    pub(crate) text: String,
    pub(crate) bg: Rgba,
    pub(crate) fg: Rgba,
    pub(crate) cursor: Rgba,
    pub(crate) cursor_visible: bool,
}

impl MenuRenderSnapshot {
    pub(crate) fn from_menu(menu: &PixelMenu) -> Self {
        Self {
            x: menu.x(),
            y: menu.y(),
            item_w: menu.item_width(),
            item_h: menu.item_height(),
            items: menu
                .items()
                .iter()
                .map(|item| MenuItemRenderSnapshot {
                    number: item.number,
                    label: item.label.clone(),
                    y_offset: item.y_offset,
                    gap_before: item.gap_before,
                })
                .collect(),
            selected: menu.selected(),
            font_color: menu.font_color(),
            box_color: menu.box_color(),
            show_labels: menu.show_labels(),
            show_box: menu.show_box(),
        }
    }
}

impl TextInputRenderSnapshot {
    pub(crate) fn from_input(input: &TextInput) -> Self {
        Self {
            x: input.x(),
            y: input.y(),
            max_width: input.max_width(),
            text: input.value().to_string(),
            bg: input.bg(),
            fg: input.fg(),
            cursor: input.cursor(),
            cursor_visible: input.cursor_visible(),
        }
    }
}

pub(crate) trait UiCanvas {
    fn string_width(&self, text: &str) -> u32;
    fn fill(&mut self, rect: (i32, i32, i32, i32), color: Rgba);
    fn stroke(&mut self, rect: (i32, i32, i32, i32), color: Rgba);
    fn pattern_fill(&mut self, rect: (i32, i32, i32, i32), color: Rgba);
    fn text(&mut self, position: (i32, i32), color: Rgba, text: &str);
    fn right_text(&mut self, position: (i32, i32), color: Rgba, text: &str);
    fn center_text(&mut self, position: (i32, i32), color: Rgba, text: &str);
    fn sprite(&mut self, idx: u16, position: (i32, i32));
    fn sprite_with_material(
        &mut self,
        idx: u16,
        position: (i32, i32),
        material: SpriteMaterialData,
    );
    fn static_image_region(
        &mut self,
        image: StaticImage,
        source: EngineRect,
        destination: EngineRect,
        modulation: Option<Rgba>,
        destination_offset: (i32, i32),
    );
    fn pixels(&mut self, batches: PointBatches);
    fn paint_pixel_menu(&mut self, menu: &MenuRenderSnapshot);
    fn paint_text_input(&mut self, input: &TextInputRenderSnapshot);
}

#[cfg(test)]
mod tests {
    use super::{Key, PixelMenu};

    #[test]
    fn function_keys_return_zero_based_indices() {
        assert_eq!(PixelMenu::function_key_index(Key::F1), Some(0));
        assert_eq!(PixelMenu::function_key_index(Key::F10), Some(9));
    }
}

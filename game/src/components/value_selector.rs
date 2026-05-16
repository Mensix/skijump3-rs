use engine::ui::{Component, Element, Event, Key};

#[derive(Debug)]
pub enum ValueSelectorAction {
    Commit(usize),
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueSelectorKind {
    ColorBars,
    Numeric,
}

#[derive(Debug)]
pub struct ValueSelector {
    x: i32,
    y: i32,
    width: i32,
    max: usize,
    value: usize,
    kind: ValueSelectorKind,
    bg: u8,
    border: u8,
    fg: u8,
    suit_boxes: bool,
    display: String,
    right_text: Option<String>,
    wrap: bool,
}

impl ValueSelector {
    #[must_use] 
    pub fn color_bars(
        x: i32,
        y: i32,
        max: usize,
        value: usize,
        bg: u8,
        border: u8,
        suit_boxes: bool,
    ) -> Self {
        Self {
            x,
            y,
            width: 31,
            max,
            value,
            kind: ValueSelectorKind::ColorBars,
            bg,
            border,
            fg: 240,
            suit_boxes,
            display: String::new(),
            right_text: None,
            wrap: true,
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[must_use] 
    pub fn numeric(
        x: i32,
        y: i32,
        width: i32,
        max: usize,
        value: usize,
        bg: u8,
        fg: u8,
        display: String,
    ) -> Self {
        Self {
            x,
            y,
            width,
            max,
            value,
            kind: ValueSelectorKind::Numeric,
            bg,
            border: fg,
            fg,
            suit_boxes: false,
            display,
            right_text: None,
            wrap: true,
        }
    }

    #[must_use] 
    pub fn value(&self) -> usize {
        self.value
    }

    pub fn set_display(&mut self, text: &str) {
        self.display = text.to_string();
    }

    pub fn set_right_text(&mut self, text: &str) {
        self.right_text = Some(text.to_string());
    }

    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }
}

impl Component for ValueSelector {
    type Action = ValueSelectorAction;

    fn elements(&self) -> Vec<Element> {
        match self.kind {
            ValueSelectorKind::ColorBars => self.color_elements(),
            ValueSelectorKind::Numeric => self.numeric_elements(),
        }
    }

    fn handle_event(&mut self, event: &Event) -> Option<Self::Action> {
        match event {
            Event::Keyboard(Key::Up | Key::Left) => {
                if self.wrap && self.value == 0 {
                    self.value = self.max;
                } else {
                    self.value = self.value.saturating_sub(1);
                }
                None
            }
            Event::Keyboard(Key::Down | Key::Right) => {
                if self.wrap && self.value == self.max {
                    self.value = 0;
                } else {
                    self.value = (self.value + 1).min(self.max);
                }
                None
            }
            Event::Keyboard(Key::Home) => {
                self.value = 0;
                None
            }
            Event::Keyboard(Key::End) => {
                self.value = self.max;
                None
            }
            Event::Keyboard(Key::PageUp) => {
                self.value = self.value.saturating_sub(10);
                None
            }
            Event::Keyboard(Key::PageDown) => {
                self.value = (self.value + 10).min(self.max);
                None
            }
            Event::Keyboard(Key::Enter | Key::Char(' ')) => {
                Some(ValueSelectorAction::Commit(self.value))
            }
            Event::Keyboard(Key::Escape) => Some(ValueSelectorAction::Cancel),
            _ => None,
        }
    }
}

impl ValueSelector {
    fn color_elements(&self) -> Vec<Element> {
        let mut els = vec![
            Element::fillbox(
                self.x,
                self.y,
                self.width,
                5 + ((self.max + 1) as i32 * 8),
                self.bg,
            ),
            Element::box_(
                self.x,
                self.y,
                self.width,
                5 + ((self.max + 1) as i32 * 8),
                self.border,
            ),
        ];
        for temp in 0..=self.max {
            let y = self.y + 4 + temp as i32 * 8;
            els.push(Element::fillbox(self.x + 6, y, 19, 5, (temp as u8 + 1) * 5));
            if self.suit_boxes {
                els.push(Element::box_(
                    self.x + 6,
                    y,
                    19,
                    5,
                    (temp as u8 + 1) * 5 + 2,
                ));
            }
        }
        els.push(Element::box_(
            self.x + 3,
            self.y + 2 + self.value as i32 * 8,
            25,
            9,
            self.fg,
        ));
        els
    }

    fn numeric_elements(&self) -> Vec<Element> {
        let text = if self.value == 0 {
            "None".to_string()
        } else if self.display.is_empty() {
            format!("Player #{}", self.value)
        } else {
            self.display.clone()
        };
        let mut els = vec![
            Element::fillbox(self.x - 2, self.y - 1, self.width, 8, self.bg),
            Element::text_color(text, self.x, self.y, self.fg),
        ];
        if let Some(rt) = &self.right_text {
            els.push(Element::text_color_right(rt, 316, self.y, self.fg));
        }
        els
    }
}

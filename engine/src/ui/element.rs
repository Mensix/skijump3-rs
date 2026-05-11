use crate::ui::cmd::Cmd;

pub enum Element {
    Text { text: String, x: i32, y: i32, color: u8 },
    Sprite(u8, i32, i32),
    Fillbox { x: i32, y: i32, w: i32, h: i32, color: u8 },
    Box { x: i32, y: i32, w: i32, h: i32, color: u8 },
    Button {
        text: String,
        x: i32,
        y: i32,
        selected: bool,
        cmd: Cmd,
    },
    Container(Vec<Element>),
}

impl Element {
    pub fn text(text: impl Into<String>, x: i32, y: i32) -> Self {
        Self::Text { text: text.into(), x, y, color: 15 }
    }

    pub fn text_color(text: impl Into<String>, x: i32, y: i32, color: u8) -> Self {
        Self::Text { text: text.into(), x, y, color }
    }

    pub fn sprite(idx: u8, x: i32, y: i32) -> Self {
        Self::Sprite(idx, x, y)
    }

    pub fn fillbox(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self::Fillbox { x, y, w, h, color }
    }

    pub fn box_(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self::Box { x, y, w, h, color }
    }

    pub fn button(text: impl Into<String>, x: i32, y: i32, selected: bool, cmd: Cmd) -> Self {
        Self::Button { text: text.into(), x, y, selected, cmd }
    }

    pub fn container(children: Vec<Element>) -> Self {
        Self::Container(children)
    }
}
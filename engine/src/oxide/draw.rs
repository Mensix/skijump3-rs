use crate::color::Rgba;
use crate::sprite::SpriteColorRecolor;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    #[must_use]
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

impl From<(i32, i32, i32, i32)> for Rect {
    fn from((x, y, w, h): (i32, i32, i32, i32)) -> Self {
        Self::new(x, y, w, h)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    pub position: Point,
    pub color: Rgba,
    pub align: TextAlign,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteDraw {
    pub idx: u16,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRegionDraw {
    pub pixels: Rc<[u8]>,
    pub src_w: u32,
    pub src_h: u32,
    pub src_x: i32,
    pub src_y: i32,
    pub dst_x: i32,
    pub dst_y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommand {
    Image {
        pixels: Rc<[u8]>,
        w: u32,
        h: u32,
    },
    ImageRegion(ImageRegionDraw),
    Text(TextRun),
    Sprite(SpriteDraw),
    SpriteRemapped {
        sprite: SpriteDraw,
        recolor: SpriteColorRecolor,
    },
    Fill(Rect, Rgba),
    Stroke(Rect, Rgba),
    DitherFill(u8),
}

#[derive(Debug, Default, Clone)]
pub struct CommandBuffer {
    commands: Vec<DrawCommand>,
}

impl CommandBuffer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn extend(&mut self, commands: impl IntoIterator<Item = DrawCommand>) {
        self.commands.extend(commands);
    }

    #[must_use]
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

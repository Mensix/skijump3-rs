use crate::color::Rgba;
use crate::sprite::SpriteMaterial;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STATIC_IMAGE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
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

#[derive(Debug, Clone)]
pub struct StaticImage {
    pixels: Rc<[u8]>,
    width: u32,
    height: u32,
    id: u64,
}

impl StaticImage {
    pub fn new(pixels: impl Into<Rc<[u8]>>, width: u32, height: u32) -> Self {
        Self {
            pixels: pixels.into(),
            width,
            height,
            id: NEXT_STATIC_IMAGE_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub(crate) const fn id(&self) -> u64 {
        self.id
    }
}

impl PartialEq for StaticImage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StaticImage {}

impl Hash for StaticImage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Default for StaticImage {
    fn default() -> Self {
        Self::new(Vec::<u8>::new(), 0, 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticImageRegionDraw {
    pub image: StaticImage,
    pub source: Rect,
    pub destination: Rect,
    pub modulation: Option<Rgba>,
    pub destination_offset: (i32, i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointBatches {
    pub colors: [Rgba; 4],
    pub points: Vec<(i32, i32)>,
    pub ranges: [std::ops::Range<usize>; 4],
}

impl PointBatches {
    pub fn empty() -> Self {
        Self {
            colors: [Rgba::transparent(); 4],
            points: Vec::new(),
            ranges: std::array::from_fn(|_| 0..0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Default for PointBatches {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommand {
    StaticImageRegion(StaticImageRegionDraw),
    Pixels(PointBatches),
    Text(TextRun),
    Sprite(SpriteDraw),
    SpriteWithMaterial {
        sprite: SpriteDraw,
        material: SpriteMaterial,
    },
    Fill(Rect, Rgba),
    Stroke(Rect, Rgba),
    PatternFill(Rect, Rgba),
    PatternStroke(Rect, Rgba),
}

#[derive(Debug, Default, Clone)]
pub struct CommandBuffer {
    commands: Vec<DrawCommand>,
}

impl CommandBuffer {
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

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}

/// A bitmap of pre-computed RGBA pixels.
#[derive(Debug, Clone)]
pub struct RgbaBitmap {
    pub pixels: Vec<u8>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// A bitmap of indexed-color pixels for GPU overlay drawing.
/// Index 0 is treated as transparent; other indices use the current palette.
#[derive(Debug, Clone)]
pub struct IndexedBitmap {
    pub pixels: Vec<u8>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

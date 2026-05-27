use crate::video::TextureId;

/// A region within an RGBA atlas texture.
/// Coordinates define the sub-rect in the atlas image.
/// `center_x`/`center_y` mirror `SpriteData` offsets for positioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub center_x: i8,
    pub center_y: i8,
}

/// A collection of sprite regions packed into a single RGBA texture.
pub struct Atlas {
    pub texture_id: TextureId,
    pub regions: Vec<AtlasRegion>,
}

impl Atlas {
    pub fn region(&self, index: usize) -> Option<&AtlasRegion> {
        self.regions.get(index)
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }
}

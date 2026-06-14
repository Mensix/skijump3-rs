use crate::bitmap::IndexedBitmap;
use crate::color::Rgba;
use crate::consts::{HEIGHT, WIDTH};
use crate::palette::{Palette, PaletteIndex, PALETTE_SIZE};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub pixels: Box<[u8]>,
    pub width: u16,
    pub height: u16,
    pub center_x: i8,
    pub center_y: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteMaterialId(u64);

impl SpriteMaterialId {
    pub const DEFAULT: Self = Self(0);

    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub struct SpriteMaterial {
    id: SpriteMaterialId,
    overrides: Box<[(PaletteIndex, Rgba)]>,
}

impl PartialEq for SpriteMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SpriteMaterial {}

impl Hash for SpriteMaterial {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl SpriteMaterial {
    #[must_use]
    pub fn new(overrides: &[(u8, Rgba)]) -> Self {
        let mut overrides: Vec<_> = overrides
            .iter()
            .map(|&(from, to)| (PaletteIndex(from), to))
            .collect();
        overrides.sort_by_key(|(from, _)| from.value());
        let id = Self::id_for_overrides(&overrides);
        Self {
            id,
            overrides: overrides.into_boxed_slice(),
        }
    }

    #[must_use]
    pub fn with_id(id: SpriteMaterialId, overrides: &[(u8, Rgba)]) -> Self {
        let mut overrides: Vec<_> = overrides
            .iter()
            .map(|&(from, to)| (PaletteIndex(from), to))
            .collect();
        overrides.sort_by_key(|(from, _)| from.value());
        Self {
            id,
            overrides: overrides.into_boxed_slice(),
        }
    }

    fn id_for_overrides(overrides: &[(PaletteIndex, Rgba)]) -> SpriteMaterialId {
        let mut hasher = DefaultHasher::new();
        overrides.hash(&mut hasher);
        SpriteMaterialId(hasher.finish().max(1))
    }

    #[must_use]
    pub fn color_override(&self, source: PaletteIndex) -> Option<Rgba> {
        for &(from, to) in &self.overrides {
            if from == source {
                return Some(to);
            }
        }
        None
    }

    #[must_use]
    pub fn get(&self, source: u8) -> Option<Rgba> {
        self.color_override(PaletteIndex(source))
    }

}

impl Default for SpriteMaterial {
    fn default() -> Self {
        Self {
            id: SpriteMaterialId::DEFAULT,
            overrides: Box::new([]),
        }
    }
}

impl SpriteMaterial {
    #[must_use]
    pub fn resolved_palette(&self, palette: &Palette) -> [Rgba; PALETTE_SIZE] {
        let mut resolved = [Rgba::transparent(); PALETTE_SIZE];
        for (idx, color) in resolved.iter_mut().enumerate() {
            *color = palette.color(PaletteIndex(idx as u8));
        }
        for &(index, color) in &self.overrides {
            resolved[index.value() as usize] = color;
        }
        resolved
    }
}

impl SpriteData {
    /// Render sprite to a minimal indexed bitmap suitable for GPU upload.
    /// Returns `None` when the sprite is fully off-screen or has no
    /// non-zero visible pixels.
    #[must_use]
    pub fn render_bitmap(&self, dst_x: i32, dst_y: i32) -> Option<IndexedBitmap> {
        let start_x = dst_x - i32::from(self.center_x);
        let start_y = dst_y - i32::from(self.center_y);

        let vis_left = start_x.max(0);
        let vis_top = start_y.max(0);
        let vis_right = (start_x + i32::from(self.width)).min(WIDTH as i32);
        let vis_bottom = (start_y + i32::from(self.height)).min(HEIGHT as i32);
        let vis_w = (vis_right - vis_left).max(0) as u32;
        let vis_h = (vis_bottom - vis_top).max(0) as u32;

        if vis_w == 0 || vis_h == 0 {
            return None;
        }

        let mut pixels = vec![0u8; (vis_w * vis_h) as usize];
        let mut has_opaque_pixel = false;

        for src_y in 0..i32::from(self.height) {
            let screen_y = start_y + src_y;
            if screen_y < vis_top || screen_y >= vis_bottom {
                continue;
            }
            for src_x in 0..i32::from(self.width) {
                let screen_x = start_x + src_x;
                if screen_x < vis_left || screen_x >= vis_right {
                    continue;
                }
                let src_idx = src_y as usize * self.width as usize + src_x as usize;
                if src_idx >= self.pixels.len() {
                    continue;
                }
                let pixel = self.pixels[src_idx];
                if pixel == 0 {
                    continue;
                }
                has_opaque_pixel = true;
                let dx = (screen_x - vis_left) as u32;
                let dy = (screen_y - vis_top) as u32;
                pixels[(dy * vis_w + dx) as usize] = pixel;
            }
        }

        if !has_opaque_pixel {
            return None;
        }

        Some(IndexedBitmap {
            pixels,
            x: vis_left,
            y: vis_top,
            width: vis_w,
            height: vis_h,
        })
    }
}

impl SpriteData {
    /// Render the full sprite to an RGBA buffer with material overrides.
    /// Source index 0 is transparent; overridden indices use explicit RGBA;
    /// all others use the source palette.
    pub fn render_material_rgba(
        &self,
        palette: &Palette,
        material: &SpriteMaterial,
        out: &mut Vec<u8>,
    ) {
        let count = self.width as usize * self.height as usize;
        out.clear();
        out.reserve(count * 4);
        let resolved_palette = material.resolved_palette(palette);
        for &pixel in self.pixels.iter().take(count) {
            if pixel == 0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let rgba = resolved_palette[pixel as usize];
                out.push(rgba.r);
                out.push(rgba.g);
                out.push(rgba.b);
                out.push(rgba.a);
            }
        }
    }

    #[must_use]
    pub fn has_palette_index(&self, index: PaletteIndex) -> bool {
        self.pixels.iter().any(|&pixel| pixel == index.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::WIDTH;

    // ---------------------------------------------------------------------------
    // IndexedBitmap rendering tests
    // ---------------------------------------------------------------------------

    #[test]
    fn render_bitmap_basic() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let bm = sprite.render_bitmap(0, 0).unwrap();
        assert_eq!(bm.width, 2);
        assert_eq!(bm.height, 2);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.y, 0);
        assert_eq!(bm.pixels, vec![1, 2, 3, 4]);
    }

    #[test]
    fn render_bitmap_skips_transparent() {
        let sprite = SpriteData {
            pixels: vec![0, 2, 3, 0].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let bm = sprite.render_bitmap(0, 0).unwrap();
        // transparent pixels should remain 0
        assert_eq!(bm.pixels[0], 0, "top-left pixel was 0");
        assert_eq!(bm.pixels[3], 0, "bottom-right pixel was 0");
        // opaque pixels preserved
        assert_eq!(bm.pixels[1], 2);
        assert_eq!(bm.pixels[2], 3);
    }

    #[test]
    fn render_bitmap_clips_right_edge() {
        let sprite = SpriteData {
            pixels: vec![1, 2].into_boxed_slice(),
            width: 2,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        // Place so only 1 pixel visible at the right edge.
        // First sprite pixel at x=WIDTH-1 is visible, second at x=WIDTH is clipped.
        let bm = sprite.render_bitmap((WIDTH - 1) as i32, 0).unwrap();
        assert_eq!(bm.width, 1, "only leftmost pixel visible");
        assert_eq!(bm.pixels[0], 1);
    }

    #[test]
    fn render_bitmap_clips_bottom_edge() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        // Place at y=HEIGHT-1 so only top row visible (bottom row clipped)
        let bm = sprite.render_bitmap(0, (HEIGHT - 1) as i32).unwrap();
        assert_eq!(bm.height, 1, "only top row visible");
        assert_eq!(bm.pixels[0], 1);
        assert_eq!(bm.pixels[1], 2);
    }

    #[test]
    fn render_bitmap_clips_left_negative() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        // Place at x=-1 so only right column visible
        let bm = sprite.render_bitmap(-1, 0).unwrap();
        assert_eq!(bm.width, 1, "only right column visible");
        assert_eq!(bm.y, 0);
        assert_eq!(bm.pixels[0], 2, "(1,0) was pixel index 2");
        assert_eq!(bm.pixels[1], 4, "(1,1) was pixel index 4");
    }

    #[test]
    fn render_bitmap_clips_top_negative() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        // Place at y=-1 so only bottom row visible
        let bm = sprite.render_bitmap(0, -1).unwrap();
        assert_eq!(bm.height, 1, "only bottom row visible");
        assert_eq!(bm.pixels[0], 3, "(0,1) was pixel index 3");
        assert_eq!(bm.pixels[1], 4, "(1,1) was pixel index 4");
    }

    #[test]
    fn render_bitmap_fully_offscreen_right_returns_none() {
        let sprite = SpriteData {
            pixels: vec![1].into_boxed_slice(),
            width: 1,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        assert!(sprite.render_bitmap(WIDTH as i32 + 100, 0).is_none());
    }

    #[test]
    fn render_bitmap_center_offset() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 1,
            center_y: 1,
        };
        // With center=(1,1), the anchor (dst_x, dst_y) is at the center.
        // start_x = dst_x - center_x = 10 - 1 = 9
        // start_y = dst_y - center_y = 20 - 1 = 19
        let bm = sprite.render_bitmap(10, 20).unwrap();
        assert_eq!(bm.x, 9);
        assert_eq!(bm.y, 19);
        assert_eq!(bm.width, 2);
        assert_eq!(bm.height, 2);
    }

    #[test]
    fn render_bitmap_all_transparent_returns_none() {
        let sprite = SpriteData {
            pixels: vec![0, 0, 0, 0].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        assert!(sprite.render_bitmap(0, 0).is_none());
    }

    #[test]
    fn render_bitmap_preserves_original_values_no_remap() {
        let sprite = SpriteData {
            pixels: vec![5, 10, 15, 20].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let bm = sprite.render_bitmap(0, 0).unwrap();
        assert_eq!(bm.pixels, vec![5, 10, 15, 20]);
    }

    #[test]
    fn render_bitmap_partially_offscreen_left() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 3,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        // Place at x=-2 so only 1 pixel visible
        let bm = sprite.render_bitmap(-2, 0).unwrap();
        assert_eq!(bm.width, 1);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.pixels[0], 3, "third pixel (index 2) is visible");
    }
}

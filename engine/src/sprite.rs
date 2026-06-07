use crate::bitmap::{IndexedBitmap, RgbaBitmap};
use crate::color::Rgba;
use crate::consts::{HEIGHT, WIDTH};

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub data: Vec<u8>,
    pub rgba_data: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub center_x: i8,
    pub center_y: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpriteColorRecolor {
    pairs: Vec<(u8, Rgba)>,
}

impl SpriteColorRecolor {
    #[must_use]
    pub fn new(pairs: Vec<(u8, Rgba)>) -> Self {
        Self { pairs }
    }

    #[must_use]
    pub fn get(&self, source: u8) -> Option<Rgba> {
        for &(from, to) in &self.pairs {
            if from == source {
                return Some(to);
            }
        }
        None
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
                if src_idx >= self.data.len() {
                    continue;
                }
                let pixel = self.data[src_idx];
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

// ---------------------------------------------------------------------------
// RGBA rendering methods (palette-independent; uses precomputed rgba_data)
// ---------------------------------------------------------------------------

impl SpriteData {
    /// Render sprite to a minimal RGBA bitmap, clipping to screen bounds.
    /// Returns `None` when the sprite is fully off-screen or has no
    /// non-zero visible pixels.
    #[must_use]
    pub fn render_rgba_bitmap(&self, dst_x: i32, dst_y: i32) -> Option<RgbaBitmap> {
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

        let mut pixels = vec![0u8; (vis_w * vis_h * 4) as usize];
        let mut has_opaque = false;

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
                let src_idx = (src_y as usize * self.width as usize + src_x as usize) * 4;
                if src_idx + 4 > self.rgba_data.len() {
                    continue;
                }
                let pixel_val = self.data[src_idx / 4];
                if pixel_val == 0 {
                    continue;
                }
                has_opaque = true;
                let dx = (screen_x - vis_left) as u32;
                let dy = (screen_y - vis_top) as u32;
                let dst_off = (dy * vis_w + dx) as usize * 4;
                pixels[dst_off..dst_off + 4].copy_from_slice(&self.rgba_data[src_idx..src_idx + 4]);
            }
        }

        if !has_opaque {
            return None;
        }

        Some(RgbaBitmap {
            pixels,
            x: vis_left,
            y: vis_top,
            width: vis_w,
            height: vis_h,
        })
    }

    /// Render the full sprite to an RGBA buffer with colour recolor.
    /// Source index 0 → transparent; recolored indices → explicit RGBA;
    /// all others → precomputed rgba_data.
    pub fn render_recolored_rgba(&self, recolor: &SpriteColorRecolor, out: &mut Vec<u8>) {
        let count = self.width as usize * self.height as usize;
        out.clear();
        out.reserve(count * 4);
        for (i, &pixel) in self.data.iter().enumerate().take(count) {
            if pixel == 0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
            } else if let Some(rgba) = recolor.get(pixel) {
                out.push(rgba.r);
                out.push(rgba.g);
                out.push(rgba.b);
                out.push(rgba.a);
            } else {
                let src_off = i * 4;
                if src_off + 4 <= self.rgba_data.len() {
                    out.extend_from_slice(&self.rgba_data[src_off..src_off + 4]);
                } else {
                    out.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
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
            data: vec![1, 2, 3, 4],
            rgba_data: Vec::new(),
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
            data: vec![0, 2, 3, 0],
            rgba_data: Vec::new(),
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
            data: vec![1, 2],
            rgba_data: Vec::new(),
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
            data: vec![1, 2, 3, 4],
            rgba_data: Vec::new(),
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
            data: vec![1, 2, 3, 4],
            rgba_data: Vec::new(),
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
            data: vec![1, 2, 3, 4],
            rgba_data: Vec::new(),
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
            data: vec![1],
            rgba_data: Vec::new(),
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
            data: vec![1, 2, 3, 4],
            rgba_data: Vec::new(),
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
            data: vec![0, 0, 0, 0],
            rgba_data: Vec::new(),
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
            data: vec![5, 10, 15, 20],
            rgba_data: Vec::new(),
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
            data: vec![1, 2, 3, 4],
            rgba_data: Vec::new(),
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

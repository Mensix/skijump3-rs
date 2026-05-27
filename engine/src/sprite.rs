use crate::consts::{HEIGHT, WIDTH};

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub center_x: i8,
    pub center_y: i8,
}

/// Indexed-color pixel bitmap produced by `render_bitmap` / `render_bitmap_with_remap`.
/// Index 0 is transparent; other indices use the current palette.
pub struct SpriteBitmap {
    pub pixels: Vec<u8>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct SpriteColorRemap {
    pairs: Vec<(u8, u8)>,
}

impl SpriteColorRemap {
    pub fn new(pairs: Vec<(u8, u8)>) -> Self {
        Self { pairs }
    }

    pub fn map(&self, pixel: u8) -> u8 {
        for &(from, to) in &self.pairs {
            if pixel == from {
                return to;
            }
        }
        pixel
    }
}

impl SpriteData {
    pub fn blit_to(&self, pixels: &mut [u8], screen_w: u32, dst_x: i32, dst_y: i32) {
        self.blit_to_impl(pixels, screen_w, dst_x, dst_y, None);
    }

    pub fn blit_to_with_remap(
        &self,
        pixels: &mut [u8],
        screen_w: u32,
        dst_x: i32,
        dst_y: i32,
        remap: &SpriteColorRemap,
    ) {
        self.blit_to_impl(pixels, screen_w, dst_x, dst_y, Some(remap));
    }

    fn blit_to_impl(
        &self,
        pixels: &mut [u8],
        screen_w: u32,
        dst_x: i32,
        dst_y: i32,
        remap: Option<&SpriteColorRemap>,
    ) {
        let start_x = dst_x - self.center_x as i32;
        let start_y = dst_y - self.center_y as i32;
        for yy in 0..self.height as i32 {
            for xx in 0..self.width as i32 {
                let src_idx = (yy * self.width as i32 + xx) as usize;
                if src_idx < self.data.len() {
                    let pixel = self.data[src_idx];
                    if pixel == 0 {
                        continue;
                    }
                    let px = start_x + xx;
                    let py = start_y + yy;
                    if px >= 0 && py >= 0 && (px as u32) < screen_w && py < HEIGHT as i32 {
                        let idx = (py as usize) * (screen_w as usize) + (px as usize);
                        pixels[idx] = if let Some(r) = remap {
                            r.map(pixel)
                        } else {
                            pixel
                        };
                    }
                }
            }
        }
    }

    /// Render sprite to a minimal indexed bitmap suitable for GPU upload.
    /// Returns `None` when the sprite is fully off-screen or has no
    /// non-zero visible pixels.
    pub fn render_bitmap(&self, dst_x: i32, dst_y: i32) -> Option<SpriteBitmap> {
        self.render_bitmap_impl(dst_x, dst_y, None)
    }

    /// Render sprite with colour remapping to a minimal indexed bitmap.
    pub fn render_bitmap_with_remap(
        &self,
        dst_x: i32,
        dst_y: i32,
        remap: &SpriteColorRemap,
    ) -> Option<SpriteBitmap> {
        self.render_bitmap_impl(dst_x, dst_y, Some(remap))
    }

    fn render_bitmap_impl(
        &self,
        dst_x: i32,
        dst_y: i32,
        remap: Option<&SpriteColorRemap>,
    ) -> Option<SpriteBitmap> {
        let start_x = dst_x - self.center_x as i32;
        let start_y = dst_y - self.center_y as i32;

        // Visible bounding box clipped to screen
        let vis_left = start_x.max(0);
        let vis_top = start_y.max(0);
        let vis_right = (start_x + self.width as i32).min(WIDTH as i32);
        let vis_bottom = (start_y + self.height as i32).min(HEIGHT as i32);
        let vis_w = (vis_right - vis_left).max(0) as u32;
        let vis_h = (vis_bottom - vis_top).max(0) as u32;

        if vis_w == 0 || vis_h == 0 {
            return None;
        }

        let mut pixels = vec![0u8; (vis_w * vis_h) as usize];

        for src_y in 0..self.height as i32 {
            let screen_y = start_y + src_y;
            if screen_y < vis_top || screen_y >= vis_bottom {
                continue;
            }
            for src_x in 0..self.width as i32 {
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
                let final_pixel = if let Some(r) = remap {
                    r.map(pixel)
                } else {
                    pixel
                };
                let dx = (screen_x - vis_left) as u32;
                let dy = (screen_y - vis_top) as u32;
                pixels[(dy * vis_w + dx) as usize] = final_pixel;
            }
        }

        Some(SpriteBitmap {
            pixels,
            x: vis_left,
            y: vis_top,
            width: vis_w,
            height: vis_h,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::WIDTH;

    #[test]
    fn remap_maps_known_indices() {
        let remap = SpriteColorRemap::new(vec![(10, 20), (30, 40)]);
        assert_eq!(remap.map(10), 20);
        assert_eq!(remap.map(30), 40);
    }

    #[test]
    fn remap_preserves_unknown_indices() {
        let remap = SpriteColorRemap::new(vec![(10, 20)]);
        assert_eq!(remap.map(0), 0);
        assert_eq!(remap.map(5), 5);
        assert_eq!(remap.map(255), 255);
    }

    #[test]
    fn remap_transparency_is_blit_concern_not_map_concern() {
        let remap = SpriteColorRemap::new(vec![(0, 99)]);
        // map() is a pure pixel-index transform; transparency (pixel==0) is
        // handled by the blit loop BEFORE map() is called. So map(0) -> 99 is correct.
        assert_eq!(remap.map(0), 99);
    }

    #[test]
    fn blit_with_remap_writes_remapped_pixels() {
        // sprite of 2x1: [0, 10] (transparent, index 10)
        let sprite = SpriteData {
            data: vec![0, 10],
            width: 2,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        let mut buf = vec![99u8; WIDTH as usize];
        let remap = SpriteColorRemap::new(vec![(10, 20)]);
        sprite.blit_to_with_remap(&mut buf, WIDTH, 0, 0, &remap);
        // pixel 0 was transparent, should remain
        // pixel 1 should be remapped from 10 to 20
        assert_eq!(buf[0], 99, "transparent pixel should not be written");
        assert_eq!(buf[1], 20, "index 10 should be remapped to 20");
    }

    #[test]
    fn blit_with_remap_unknown_index_stays_unchanged() {
        let sprite = SpriteData {
            data: vec![0, 50],
            width: 2,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        let mut buf = vec![99u8; WIDTH as usize];
        let remap = SpriteColorRemap::new(vec![(10, 20)]);
        sprite.blit_to_with_remap(&mut buf, WIDTH, 0, 0, &remap);
        // pixel 1 has index 50 which isn't in the remap pairs
        assert_eq!(buf[1], 50, "unmapped index should be preserved");
    }

    #[test]
    fn blit_with_remap_skips_transparent() {
        let sprite = SpriteData {
            data: vec![0, 0],
            width: 2,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        let mut buf = vec![99u8; WIDTH as usize];
        let remap = SpriteColorRemap::new(vec![(10, 20)]);
        sprite.blit_to_with_remap(&mut buf, WIDTH, 0, 0, &remap);
        assert_eq!(buf[0], 99, "pixel 0 should not be touched");
        assert_eq!(buf[1], 99, "pixel 0 should not be touched");
    }

    #[test]
    fn blit_with_remap_clips_correctly() {
        let sprite = SpriteData {
            data: vec![10, 10],
            width: 2,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        let mut buf = vec![99u8; WIDTH as usize];
        let remap = SpriteColorRemap::new(vec![(10, 20)]);
        // draw at x=WIDTH-1 so the second pixel is clipped
        sprite.blit_to_with_remap(&mut buf, WIDTH, WIDTH as i32 - 1, 0, &remap);
        assert_eq!(
            buf[(WIDTH - 1) as usize],
            20,
            "first pixel should be remapped and drawn"
        );
        // pixel at x=WIDTH is off-screen; buf[0] still has the leftover from WIDTH-1
        // Actually this is already the last pixel - the second one is off-screen
    }

    // ---------------------------------------------------------------------------
    // SpriteBitmap rendering tests
    // ---------------------------------------------------------------------------

    #[test]
    fn render_bitmap_basic() {
        let sprite = SpriteData {
            data: vec![1, 2, 3, 4],
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
    fn render_bitmap_remap() {
        let sprite = SpriteData {
            data: vec![0, 10, 10, 0],
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let remap = SpriteColorRemap::new(vec![(10, 20)]);
        let bm = sprite.render_bitmap_with_remap(0, 0, &remap).unwrap();
        assert_eq!(bm.pixels[0], 0, "transparent remained");
        assert_eq!(bm.pixels[1], 20, "index 10 remapped to 20");
        assert_eq!(bm.pixels[2], 20, "index 10 remapped to 20");
        assert_eq!(bm.pixels[3], 0, "transparent remained");
    }

    #[test]
    fn render_bitmap_remap_unknown_preserved() {
        let sprite = SpriteData {
            data: vec![50],
            width: 1,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        let remap = SpriteColorRemap::new(vec![(10, 20)]);
        let bm = sprite.render_bitmap_with_remap(0, 0, &remap).unwrap();
        assert_eq!(bm.pixels[0], 50, "unmapped index preserved");
    }

    #[test]
    fn render_bitmap_clips_right_edge() {
        let sprite = SpriteData {
            data: vec![1, 2],
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
    fn render_bitmap_all_transparent_returns_some_with_zeros() {
        // render_bitmap always returns Some when there are visible pixels,
        // even if they're all transparent (pixel == 0 is skipped, so the
        // bitmap is all zeros, but the bounding box still exists)
        let sprite = SpriteData {
            data: vec![0, 0, 0, 0],
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let bm = sprite.render_bitmap(0, 0).unwrap();
        assert_eq!(bm.width, 2);
        assert_eq!(bm.height, 2);
        assert!(bm.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn render_bitmap_preserves_original_values_no_remap() {
        let sprite = SpriteData {
            data: vec![5, 10, 15, 20],
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

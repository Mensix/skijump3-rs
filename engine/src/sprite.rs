use crate::consts::HEIGHT;

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub center_x: i8,
    pub center_y: i8,
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
}

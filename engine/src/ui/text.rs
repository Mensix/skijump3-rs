use crate::bitmap::IndexedBitmap;
use crate::consts::{FONT_GLYPH_COUNT, HEIGHT, SHADOW_PIXEL, WIDTH};
use crate::sprite::SpriteData;

fn blit_glyph_color(
    g: &SpriteData,
    pixels: &mut [u8],
    screen_w: u32,
    dst_x: i32,
    dst_y: i32,
    color: u8,
) {
    let start_x = dst_x - g.center_x as i32;
    let start_y = dst_y - g.center_y as i32;
    for yy in 0..g.height as i32 {
        for xx in 0..g.width as i32 {
            let src_idx = (yy * g.width as i32 + xx) as usize;
            if src_idx >= g.data.len() {
                continue;
            }
            let pixel = g.data[src_idx];
            if pixel == 0 {
                continue;
            }
            let px = start_x + xx;
            let py = start_y + yy;
            if px < 0 || py < 0 || px >= screen_w as i32 || py >= HEIGHT as i32 {
                continue;
            }
            let idx = py as usize * screen_w as usize + px as usize;
            pixels[idx] = if pixel == SHADOW_PIXEL {
                SHADOW_PIXEL
            } else {
                color
            };
        }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    glyphs: Vec<Option<SpriteData>>,
}

impl Font {
    pub fn new() -> Self {
        Self {
            glyphs: (0..FONT_GLYPH_COUNT).map(|_| None).collect(),
        }
    }

    pub fn from_sprites(sprites: &[SpriteData]) -> Self {
        let mut font = Self::new();
        for (i, sprite) in sprites.iter().enumerate() {
            if i < FONT_GLYPH_COUNT {
                font.glyphs[i] = Some(sprite.clone());
            }
        }
        font
    }

    pub fn blit_string(&self, pixels: &mut [u8], screen_w: u32, text: &str, x: i32, y: i32) {
        self.blit_string_color(pixels, screen_w, text, x, y, 0);
    }

    pub fn string_width(&self, text: &str) -> u32 {
        let mut w = 0u32;
        for ch in text.chars() {
            match ch {
                ' ' => w += 4,
                '$' => w += 5,
                _ => {
                    if let Some(idx) = Self::char_to_index(ch) {
                        if let Some(ref g) = self.glyphs[idx] {
                            w += g.width as u32;
                        }
                    }
                }
            }
        }
        w
    }

    pub fn blit_string_color(
        &self,
        pixels: &mut [u8],
        screen_w: u32,
        text: &str,
        x: i32,
        y: i32,
        color: u8,
    ) {
        let mut px = x;
        for ch in text.chars() {
            match ch {
                ' ' => px += 4,
                '$' => px += 5,
                _ => {
                    if let Some(idx) = Self::char_to_index(ch) {
                        if let Some(ref g) = self.glyphs[idx] {
                            if color != 0 {
                                blit_glyph_color(g, pixels, screen_w, px, y, color);
                            } else {
                                g.blit_to(pixels, screen_w, px, y);
                            }
                            px += g.width as i32;
                        }
                    }
                }
            }
        }
    }

    /// Render text into a minimal indexed bitmap suitable for GPU upload.
    /// Returns `None` when there are no drawable glyphs.
    /// Index 0 in the result is transparent; other indices are the final
    /// mapped color (either `color` or the original glyph pixel when
    /// `color == 0`). The shadow pixel (`SHADOW_PIXEL = 242`) is preserved.
    ///
    /// The bitmap's (`x`, `y`) is the screen-space top-left corner.
    pub fn render_string_bitmap(
        &self,
        text: &str,
        x: i32,
        y: i32,
        color: u8,
    ) -> Option<IndexedBitmap> {
        // First pass: compute bounding box of all glyphs
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut px = x;
        let mut defined = false;

        // We need to store glyph references for second pass.
        // Collect (glyph_idx, screen_px, screen_y) for each drawable glyph.
        // The screen_y is always `y` for all glyphs (monospace line).
        struct GlyphPos {
            idx: usize,
            screen_px: i32,
            width: u16,
            height: u16,
            center_x: i8,
            center_y: i8,
        }
        let mut positions: Vec<GlyphPos> = Vec::new();

        for ch in text.chars() {
            match ch {
                ' ' => px += 4,
                '$' => px += 5,
                _ => {
                    if let Some(glyph_idx) = Self::char_to_index(ch) {
                        if let Some(ref g) = self.glyphs[glyph_idx] {
                            let left = px - g.center_x as i32;
                            let top = y - g.center_y as i32;
                            let right = left + g.width as i32;
                            let bottom = top + g.height as i32;
                            min_x = min_x.min(left);
                            min_y = min_y.min(top);
                            max_x = max_x.max(right);
                            max_y = max_y.max(bottom);
                            positions.push(GlyphPos {
                                idx: glyph_idx,
                                screen_px: px,
                                width: g.width,
                                height: g.height,
                                center_x: g.center_x,
                                center_y: g.center_y,
                            });
                            px += g.width as i32;
                            defined = true;
                        }
                    }
                }
            }
        }

        if !defined {
            return None;
        }

        // Clip bounding box to screen
        let bitmap_x = min_x.max(0);
        let bitmap_y = min_y.max(0);
        let bitmap_w = (max_x.min(WIDTH as i32) - bitmap_x).max(0) as u32;
        let bitmap_h = (max_y.min(HEIGHT as i32) - bitmap_y).max(0) as u32;

        if bitmap_w == 0 || bitmap_h == 0 {
            return None;
        }

        let mut pixels = vec![0u8; (bitmap_w * bitmap_h) as usize];
        let bitmap_w_i32 = bitmap_w as i32;
        let bitmap_h_i32 = bitmap_h as i32;

        // Second pass: render each glyph into the bitmap
        for gp in &positions {
            if let Some(ref g) = self.glyphs[gp.idx] {
                let start_x = gp.screen_px - gp.center_x as i32;
                let start_y = y - gp.center_y as i32;
                for yy in 0..gp.height as i32 {
                    for xx in 0..gp.width as i32 {
                        let src_idx = (yy * gp.width as i32 + xx) as usize;
                        if src_idx >= g.data.len() {
                            continue;
                        }
                        let glyph_pixel = g.data[src_idx];
                        if glyph_pixel == 0 {
                            continue;
                        }
                        let sx = start_x + xx;
                        let sy = start_y + yy;
                        // Clip to bitmap
                        if sx < bitmap_x
                            || sy < bitmap_y
                            || sx >= bitmap_x + bitmap_w_i32
                            || sy >= bitmap_y + bitmap_h_i32
                        {
                            continue;
                        }
                        let dx = (sx - bitmap_x) as u32;
                        let dy = (sy - bitmap_y) as u32;
                        let final_pixel = if glyph_pixel == SHADOW_PIXEL {
                            SHADOW_PIXEL
                        } else if color != 0 {
                            color
                        } else {
                            glyph_pixel
                        };
                        pixels[(dy * bitmap_w + dx) as usize] = final_pixel;
                    }
                }
            }
        }

        Some(IndexedBitmap {
            pixels,
            x: bitmap_x,
            y: bitmap_y,
            width: bitmap_w,
            height: bitmap_h,
        })
    }

    fn char_to_index(c: char) -> Option<usize> {
        match c {
            'A'..='Z' => Some((c as usize) - ('A' as usize)),
            'a'..='z' => Some((c as usize) - ('a' as usize)),
            'Å' | 'å' => Some(26),
            'Ä' | 'ä' => Some(27),
            'Ö' | 'ö' => Some(28),
            '0' => Some(29),
            '1'..='9' => Some((c as usize) - ('1' as usize) + 30),
            ':' => Some(39),
            '.' => Some(40),
            '?' => Some(41),
            '!' => Some(42),
            '*' => Some(43),
            '-' => Some(44),
            '+' => Some(45),
            ',' => Some(46),
            '(' => Some(47),
            ')' => Some(48),
            'µ' => Some(49),
            '"' => Some(50),
            '\'' => Some(51),
            '#' => Some(52),
            'Ø' | 'ø' => Some(53),
            'Ý' | 'ý' => Some(54),
            'ß' => Some(55),
            '/' => Some(56),
            'Æ' | 'æ' => Some(57),
            '%' => Some(58),
            _ => None,
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit tests for IndexedBitmap rendering
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprite::SpriteData;

    /// Build a minimal font with two glyphs:
    ///   'A' (idx 0): 3×5 solid block of value 1, center=(0,0)
    ///   'B' (idx 1): 2×4 with a SHADOW_PIXEL, center=(0,0)
    fn make_font() -> Font {
        let mut sprites = Vec::with_capacity(FONT_GLYPH_COUNT);
        sprites.resize_with(FONT_GLYPH_COUNT, || SpriteData {
            data: vec![],
            width: 0,
            height: 0,
            center_x: 0,
            center_y: 0,
        });
        sprites[0] = SpriteData {
            data: vec![1; 15], // 3×5 block
            width: 3,
            height: 5,
            center_x: 0,
            center_y: 0,
        };
        sprites[1] = SpriteData {
            data: vec![
                2,      2,          // row 0
                2,      2,          // row 1
                2,      SHADOW_PIXEL, // row 2
                2,      2,          // row 3
            ],
            width: 2,
            height: 4,
            center_x: 0,
            center_y: 0,
        };
        Font::from_sprites(&sprites)
    }

    #[test]
    fn empty_string_returns_none() {
        let font = make_font();
        assert!(font.render_string_bitmap("", 0, 0, 1).is_none());
    }

    #[test]
    fn only_spaces_returns_none() {
        let font = make_font();
        assert!(font.render_string_bitmap("   ", 0, 0, 1).is_none());
    }

    #[test]
    fn no_matching_glyphs_returns_none() {
        let font = make_font();
        assert!(font.render_string_bitmap("@", 0, 0, 1).is_none());
    }

    #[test]
    fn single_glyph_with_color_maps_non_shadow_to_color() {
        let font = make_font();
        // 'A' is 3×5 at (0,0) when center=(0,0). With color=7 all set pixels → 7.
        let bm = font.render_string_bitmap("A", 0, 0, 7).unwrap();
        assert_eq!(bm.width, 3);
        assert_eq!(bm.height, 5);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.y, 0);
        // All non-zero pixels were 1, now should be 7
        assert!(bm.pixels.iter().all(|&p| p == 7));
    }

    #[test]
    fn single_glyph_with_color_zero_preserves_original() {
        let font = make_font();
        // 'A' at (0,0), color=0 → original glyph value 1
        let bm = font.render_string_bitmap("A", 0, 0, 0).unwrap();
        assert!(bm.pixels.iter().all(|&p| p == 1));
    }

    #[test]
    fn shadow_pixel_is_preserved() {
        let font = make_font();
        // 'B' at (0,0), color=7. Pixel at x=1, y=2 is SHADOW_PIXEL.
        // With center=(0,0), the anchor is at (0,0), so glyph pixels start at (0,0).
        let bm = font.render_string_bitmap("B", 0, 0, 7).unwrap();
        assert_eq!(bm.width, 2);
        assert_eq!(bm.height, 4);
        // Bitmap pixel at (1, 2) = y*2 + x = 2*2 + 1 = 5 → SHADOW_PIXEL
        assert_eq!(bm.pixels[5], SHADOW_PIXEL, "shadow at (1,2)");
        // All other non-zero pixels should be 7
        for (i, &p) in bm.pixels.iter().enumerate() {
            if i != 5 && p != 0 {
                assert_eq!(p, 7, "pixel at index {i}");
            }
        }
    }

    #[test]
    fn multiple_glyphs_combined_bounding_box() {
        let font = make_font();
        // "AB": 'A'(3×5) + 'B'(2×4) = 5 wide, 5 tall
        let bm = font.render_string_bitmap("AB", 0, 0, 3).unwrap();
        assert_eq!(bm.width, 5, "combined width = 3 + 2");
        assert_eq!(bm.height, 5, "max height = max(5,4)");
    }

    #[test]
    fn space_advances_cursor() {
        let font = make_font();
        // "A B": 3 + 4 + 2 = 9 wide; height = 5
        let bm = font.render_string_bitmap("A B", 0, 0, 3).unwrap();
        assert_eq!(bm.width, 9);
    }

    #[test]
    fn dollar_advances_cursor() {
        let font = make_font();
        // "A$B": 3 + 5 + 2 = 10 wide
        let bm = font.render_string_bitmap("A$B", 0, 0, 3).unwrap();
        assert_eq!(bm.width, 10);
    }

    #[test]
    fn positive_x_offset_moves_bitmap() {
        let font = make_font();
        let bm = font.render_string_bitmap("A", 50, 30, 1).unwrap();
        assert_eq!(bm.x, 50);
        assert_eq!(bm.y, 30);
    }

    #[test]
    fn bitmap_clips_to_screen_right_edge() {
        let font = make_font();
        // 'A' is 3 wide, place it at WIDTH-1 → only 1 pixel visible
        let bm = font.render_string_bitmap("A", (WIDTH - 1) as i32, 0, 1).unwrap();
        assert_eq!(bm.width, 1);
    }

    #[test]
    fn bitmap_clips_to_screen_top_edge() {
        let font = make_font();
        // 'A' is 5 tall, place at y=-2 → only 3 pixels visible (rows 0..2 in glyph = screen y -2..0 → clipped to 0..0?)
        // Actually start_y = y - center_y = -2 - 0 = -2. Rows 0..4 give screen y -2, -1, 0, 1, 2.
        // Clipped to y>=0: visible rows are 2, 3, 4 (screen y 0, 1, 2). Height = 3.
        let bm = font.render_string_bitmap("A", 0, -2, 1).unwrap();
        assert_eq!(bm.y, 0);
        assert_eq!(bm.height, 3);
    }

    #[test]
    fn completely_offscreen_returns_none() {
        let font = make_font();
        let bm = font.render_string_bitmap("A", WIDTH as i32 + 100, 0, 1);
        assert!(bm.is_none());
    }
}

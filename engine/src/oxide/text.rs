use crate::bitmap::RgbaBitmap;
use crate::color::Rgba;
use crate::consts::{FONT_GLYPH_COUNT, HEIGHT, SHADOW_PIXEL, WIDTH};

#[derive(Debug, Clone)]
pub struct Glyph {
    pub pixels: Box<[u8]>,
    pub width: u16,
    pub height: u16,
    pub center_x: i8,
    pub center_y: i8,
}

#[derive(Debug, Clone)]
pub struct Font {
    glyphs: Vec<Option<Glyph>>,
}

struct GlyphPos {
    idx: usize,
    screen_px: i32,
    width: u16,
    height: u16,
    center_x: i8,
    center_y: i8,
}

impl Font {
    pub fn new() -> Self {
        Self {
            glyphs: (0..FONT_GLYPH_COUNT).map(|_| None).collect(),
        }
    }

    pub fn from_sprites(sprites: &[Glyph]) -> Self {
        let mut font = Self::new();
        for (i, sprite) in sprites.iter().enumerate() {
            if i < FONT_GLYPH_COUNT {
                font.glyphs[i] = Some(sprite.clone());
            }
        }
        font
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
                            w += u32::from(g.width);
                        }
                    }
                }
            }
        }
        w
    }

    pub fn render_string_rgba(
        &self,
        text: &str,
        x: i32,
        y: i32,
        color: Rgba,
        shadow: Rgba,
        out: &mut Vec<u8>,
    ) -> Option<RgbaBitmap> {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut px = x;
        let mut defined = false;

        let mut positions: Vec<GlyphPos> = Vec::new();

        for ch in text.chars() {
            match ch {
                ' ' => px += 4,
                '$' => px += 5,
                _ => {
                    if let Some(glyph_idx) = Self::char_to_index(ch) {
                        if let Some(ref g) = self.glyphs[glyph_idx] {
                            let left = px - i32::from(g.center_x);
                            let top = y - i32::from(g.center_y);
                            let right = left + i32::from(g.width);
                            let bottom = top + i32::from(g.height);
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
                            px += i32::from(g.width);
                            defined = true;
                        }
                    }
                }
            }
        }

        if !defined {
            return None;
        }

        let bitmap_x = min_x.max(0);
        let bitmap_y = min_y.max(0);
        let bitmap_w = (max_x.min(WIDTH as i32) - bitmap_x).max(0) as u32;
        let bitmap_h = (max_y.min(HEIGHT as i32) - bitmap_y).max(0) as u32;

        if bitmap_w == 0 || bitmap_h == 0 {
            return None;
        }

        let total = bitmap_w as usize * bitmap_h as usize * 4;
        out.clear();
        out.reserve(total);
        out.resize(total, 0);

        let bitmap_w_i32 = bitmap_w as i32;
        let bitmap_h_i32 = bitmap_h as i32;

        let sr = shadow.r;
        let sg = shadow.g;
        let sb = shadow.b;
        let sa = shadow.a;

        let tr = color.r;
        let tg = color.g;
        let tb = color.b;
        let ta = color.a;

        for gp in &positions {
            if let Some(ref g) = self.glyphs[gp.idx] {
                let start_x = gp.screen_px - i32::from(gp.center_x);
                let start_y = y - i32::from(gp.center_y);
                for yy in 0..i32::from(gp.height) {
                    for xx in 0..i32::from(gp.width) {
                        let src_idx = (yy * i32::from(gp.width) + xx) as usize;
                        if src_idx >= g.pixels.len() {
                            continue;
                        }
                        let glyph_pixel = g.pixels[src_idx];
                        if glyph_pixel == 0 {
                            continue;
                        }
                        let sx = start_x + xx;
                        let sy = start_y + yy;

                        if sx < bitmap_x
                            || sy < bitmap_y
                            || sx >= bitmap_x + bitmap_w_i32
                            || sy >= bitmap_y + bitmap_h_i32
                        {
                            continue;
                        }
                        let dx = (sx - bitmap_x) as usize;
                        let dy = (sy - bitmap_y) as usize;
                        let pixel_offset = (dy * bitmap_w as usize + dx) * 4;
                        if glyph_pixel == SHADOW_PIXEL {
                            out[pixel_offset] = sr;
                            out[pixel_offset + 1] = sg;
                            out[pixel_offset + 2] = sb;
                            out[pixel_offset + 3] = sa;
                        } else {
                            out[pixel_offset] = tr;
                            out[pixel_offset + 1] = tg;
                            out[pixel_offset + 2] = tb;
                            out[pixel_offset + 3] = ta;
                        }
                    }
                }
            }
        }

        Some(RgbaBitmap {
            pixels: std::mem::take(out),
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

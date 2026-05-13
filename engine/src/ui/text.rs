#[derive(Debug, Clone)]
struct Glyph {
    data: Vec<u8>,
    width: u16,
    height: u16,
    center_x: i16,
    center_y: i16,
}

impl Glyph {
    fn blit_to(&self, pixels: &mut [u8], screen_w: u32, dst_x: i32, dst_y: i32) {
        let start_x = dst_x - self.center_x as i32;
        let start_y = dst_y - self.center_y as i32;
        for yy in 0..self.height as i32 {
            for xx in 0..self.width as i32 {
                let src_idx = (yy * self.width as i32 + xx) as usize;
                if src_idx >= self.data.len() {
                    continue;
                }
                let pixel = self.data[src_idx];
                if pixel == 0 {
                    continue;
                }
                let px = start_x + xx;
                let py = start_y + yy;
                if px < 0 || py < 0 || px >= (screen_w as i32) || py >= crate::consts::HEIGHT as i32
                {
                    continue;
                }
                let idx = (py as usize) * (screen_w as usize) + (px as usize);
                pixels[idx] = pixel;
            }
        }
    }

    fn blit_color(&self, pixels: &mut [u8], screen_w: u32, dst_x: i32, dst_y: i32, color: u8) {
        let start_x = dst_x - self.center_x as i32;
        let start_y = dst_y - self.center_y as i32;
        for yy in 0..self.height as i32 {
            for xx in 0..self.width as i32 {
                let src_idx = (yy * self.width as i32 + xx) as usize;
                if src_idx >= self.data.len() {
                    continue;
                }
                let pixel = self.data[src_idx];
                if pixel == 0 {
                    continue;
                }
                let px = start_x + xx;
                let py = start_y + yy;
                if px < 0 || py < 0 || px >= (screen_w as i32) || py >= crate::consts::HEIGHT as i32
                {
                    continue;
                }
                let idx = (py as usize) * (screen_w as usize) + (px as usize);
                pixels[idx] = if pixel == crate::consts::SHADOW_PIXEL {
                    crate::consts::SHADOW_PIXEL
                } else {
                    color
                };
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    glyphs: Vec<Option<Glyph>>,
}

impl Font {
    pub fn new() -> Self {
        Self {
            glyphs: (0..crate::consts::FONT_GLYPH_COUNT).map(|_| None).collect(),
        }
    }

    pub fn set_glyph(
        &mut self,
        index: usize,
        data: Vec<u8>,
        width: u16,
        height: u16,
        center_x: i8,
        center_y: i8,
    ) {
        if index < self.glyphs.len() {
            self.glyphs[index] = Some(Glyph {
                data,
                width,
                height,
                center_x: center_x as i16,
                center_y: center_y as i16,
            });
        }
    }

    pub fn blit_string(&self, pixels: &mut [u8], screen_w: u32, text: &str, x: i32, y: i32) {
        self.blit_string_color(pixels, screen_w, text, x, y, 0);
    }

    pub fn string_width(&self, text: &str) -> u32 {
        let mut w = 0u32;
        for ch in text.chars().map(|c| c.to_ascii_uppercase()) {
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
        for ch in text.chars().map(|c| c.to_ascii_uppercase()) {
            match ch {
                ' ' => px += 4,
                '$' => px += 5,
                _ => {
                    if let Some(idx) = Self::char_to_index(ch) {
                        if let Some(ref g) = self.glyphs[idx] {
                            if color != 0 {
                                g.blit_color(pixels, screen_w, px, y, color);
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

    fn char_to_index(c: char) -> Option<usize> {
        match c {
            'A'..='Z' => Some((c as usize) - ('A' as usize)),
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

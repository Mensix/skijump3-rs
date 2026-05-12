#[derive(Clone)]
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
                if px < 0 || py < 0 || px >= (screen_w as i32) || py >= crate::consts::HEIGHT as i32 {
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
                if px < 0 || py < 0 || px >= (screen_w as i32) || py >= crate::consts::HEIGHT as i32 {
                    continue;
                }
                let idx = (py as usize) * (screen_w as usize) + (px as usize);
                pixels[idx] = if pixel == crate::consts::SHADOW_PIXEL { crate::consts::SHADOW_PIXEL } else { color };
            }
        }
    }
}

#[derive(Clone)]
pub struct Font {
    glyphs: Vec<Option<Glyph>>,
}

impl Font {
    pub fn new() -> Self {
        Self { glyphs: (0..67).map(|_| None).collect() }
    }

    pub fn set_glyph(&mut self, index: usize, data: Vec<u8>, width: u16, height: u16, center_x: i8, center_y: i8) {
        if index < self.glyphs.len() {
            self.glyphs[index] = Some(Glyph { data, width, height, center_x: center_x as i16, center_y: center_y as i16 });
        }
    }

    pub fn blit_string(&self, pixels: &mut [u8], screen_w: u32, text: &str, x: i32, y: i32) {
        self.blit_string_color(pixels, screen_w, text, x, y, 0);
    }

    pub fn string_width(&self, text: &str) -> u32 {
        let mut w = 0u32;
        for byte in text.bytes().map(|b| b.to_ascii_uppercase()) {
            match byte {
                b' ' => w += 4,
                b'$' => w += 5,
                _ => {
                    if let Some(idx) = Self::char_to_index(byte) {
                        if let Some(ref g) = self.glyphs[idx] {
                            w += g.width as u32;
                        }
                    }
                }
            }
        }
        w
    }

    pub fn blit_string_color(&self, pixels: &mut [u8], screen_w: u32, text: &str, x: i32, y: i32, color: u8) {
        let mut px = x;
        for byte in text.bytes().map(|b| b.to_ascii_uppercase()) {
            match byte {
                b' ' => px += 4,
                b'$' => px += 5,
                _ => {
                    if let Some(idx) = Self::char_to_index(byte) {
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

    fn char_to_index(c: u8) -> Option<usize> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as usize),
            b'\xC4' | b'\xE4' => Some(26), // Ä
            b'\xD6' | b'\xF6' => Some(27), // Ö
            b'\xC5' | b'\xE5' => Some(28), // Å
            b'0' => Some(29),
            b'1'..=b'9' => Some((c - b'1' + 30) as usize),
            b':' => Some(39),
            b'.' => Some(40),
            b'?' => Some(41),
            b'!' => Some(42),
            b'*' => Some(43),
            b'-' => Some(44),
            b',' => Some(46),
            b'(' => Some(47),
            b')' => Some(48),
            b'\xB5' => Some(49),  // µ
            b'"' => Some(50),
            b'\'' => Some(51),
            b'#' => Some(52),
            b'\xD8' | b'\xF8' => Some(53), // Ø
            b'\xDD' | b'\xFD' => Some(54), // Ý
            b'\xDF' => Some(55), // ß
            b'/' => Some(56),
            b'\xC6' | b'\xE6' => Some(57), // Æ
            b'%' => Some(58),
            _ => None,
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}
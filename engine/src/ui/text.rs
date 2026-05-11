use crate::ui::paint::PaintCtx;
use crate::ui::widget::{Widget, Props, WidgetId};

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
                if px < 0 || py < 0 || px >= (screen_w as i32) || py >= 200 {
                    continue;
                }
                let idx = (py as usize) * (screen_w as usize) + (px as usize);
                pixels[idx] = pixel;
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
        let mut px = x;
        for byte in text.bytes().map(|b| b.to_ascii_uppercase()) {
            let idx = Self::char_to_index(byte);
            if let Some(ref g) = self.glyphs[idx] {
                g.blit_to(pixels, screen_w, px, y);
                px += g.width as i32;
            }
        }
    }

    fn char_to_index(c: u8) -> usize {
        match c {
            b'A'..=b'Z' => (c - b'A' + 1) as usize,
            b'0'..=b'9' => (c - b'0' + 53) as usize,
            _ => 0,
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TextProps {
    pub content: String,
    pub x: i32,
    pub y: i32,
}

impl Props for TextProps {}

pub struct Text {
    pub props: TextProps,
    pub font: Font,
}

impl Text {
    pub fn new(content: &str, x: i32, y: i32, font: Font) -> Self {
        Self { props: TextProps { content: content.to_string(), x, y }, font }
    }
}

impl Widget for Text {
    fn paint(&self, ctx: &mut PaintCtx, _id: WidgetId) {
        self.font.blit_string(ctx.pixels, ctx.width, &self.props.content, self.props.x, self.props.y);
    }
}
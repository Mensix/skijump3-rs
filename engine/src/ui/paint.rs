use crate::palette::Palette;

pub struct PaintCtx<'a> {
    pub pixels: &'a mut [u8],
    pub palette: &'a Palette,
    pub width: u32,
    pub height: u32,
}

impl<'a> PaintCtx<'a> {
    pub fn new(pixels: &'a mut [u8], palette: &'a Palette, width: u32, height: u32) -> Self {
        Self { pixels, palette, width, height }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        let x = x.max(0);
        let y = y.max(0);
        let w = w.min(self.width as i32 - x).max(0);
        let h = h.min(self.height as i32 - y).max(0);
        if w <= 0 || h <= 0 {
            return;
        }
        for dy in 0..h {
            let row_start = ((y + dy) as usize) * (self.width as usize);
            self.pixels[row_start..row_start + (w as usize)].fill(color);
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) {
        if x < 0 || y < 0 || x >= (self.width as i32) || y >= (self.height as i32) {
            return;
        }
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.pixels[idx] = color;
    }
}
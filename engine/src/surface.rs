use crate::palette::Dim;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    pixels: Vec<u8>,
    pub dim: Dim,
}

impl Surface {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self { pixels: vec![0; size], dim: Dim { width, height } }
    }

    pub fn with_pixels(dim: Dim, pixels: Vec<u8>) -> Self {
        Self { pixels, dim }
    }

    pub fn width(&self) -> u16 { self.dim.width }
    pub fn height(&self) -> u16 { self.dim.height }
    pub fn pixels(&self) -> &[u8] { &self.pixels }
    pub fn pixels_mut(&mut self) -> &mut [u8] { &mut self.pixels }

    pub fn fill(&mut self, color: u8) {
        self.pixels.fill(color);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        let idx = y * (self.dim.width as usize) + x;
        if idx < self.pixels.len() {
            self.pixels[idx] = color;
        }
    }

    pub fn set_pixel_clipped(&mut self, x: i32, y: i32, color: u8) {
        if x < 0 || y < 0 || x >= (self.dim.width as i32) || y >= (self.dim.height as i32) {
            return;
        }
        let idx = (y as usize) * (self.dim.width as usize) + (x as usize);
        self.pixels[idx] = color;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let idx = y * (self.dim.width as usize) + x;
        self.pixels.get(idx).copied().unwrap_or(0)
    }

    pub fn fill_rect_clipped(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        let x = x.max(0);
        let y = y.max(0);
        let w = w.min(self.dim.width as i32 - x).max(0);
        let h = h.min(self.dim.height as i32 - y).max(0);
        if w <= 0 || h <= 0 { return; }
        for dy in 0..h {
            let row_start = ((y + dy) as usize) * (self.dim.width as usize);
            self.pixels[row_start..row_start + (w as usize)].fill(color);
        }
    }

    pub fn blit(&mut self, src: &Surface, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, w: u16, h: u16) {
        for dy in 0..h as usize {
            for dx in 0..w as usize {
                let src_idx = (src_y + dy) * (src.dim.width as usize) + (src_x + dx);
                let dst_idx = (dst_y + dy) * (self.dim.width as usize) + (dst_x + dx);
                if src_idx < src.pixels.len() && dst_idx < self.pixels.len() {
                    self.pixels[dst_idx] = src.pixels[src_idx];
                }
            }
        }
    }

    pub fn blit_indexed(&mut self, src: &[u8], src_w: u32, dst_x: i32, dst_y: i32) {
        for i in 0..src.len() {
            let rel_x = (i as u32) % src_w;
            let rel_y = (i as u32) / src_w;
            self.set_pixel_clipped(dst_x + rel_x as i32, dst_y + rel_y as i32, src[i]);
        }
    }
}
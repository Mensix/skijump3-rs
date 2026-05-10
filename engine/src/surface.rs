use crate::palette::Dim;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    pixels: Vec<u8>,
    pub dim: Dim,
}

impl Surface {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            pixels: vec![0; size],
            dim: Dim { width, height },
        }
    }

    pub fn with_pixels(dim: Dim, pixels: Vec<u8>) -> Self {
        Self { pixels, dim }
    }

    pub fn width(&self) -> u16 {
        self.dim.width
    }

    pub fn height(&self) -> u16 {
        self.dim.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn fill(&mut self, color: u8) {
        self.pixels.fill(color);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        let idx = y * (self.dim.width as usize) + x;
        if idx < self.pixels.len() {
            self.pixels[idx] = color;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let idx = y * (self.dim.width as usize) + x;
        if idx < self.pixels.len() {
            self.pixels[idx]
        } else {
            0
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
}
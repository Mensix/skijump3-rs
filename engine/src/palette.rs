use std::ops::Index;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    colors: [[u8; 3]; 256],
}

impl Palette {
    #[must_use]
    pub fn new() -> Self {
        Self {
            colors: [[0; 3]; 256],
        }
    }

    pub fn from_pcx_bytes(data: &[u8]) -> Result<Self, PaletteError> {
        if data.len() < 256 * 3 {
            return Err(PaletteError::InvalidSize(data.len()));
        }
        let mut palette = Self::new();
        for i in 0..256 {
            palette.colors[i] = [data[i * 3] >> 2, data[i * 3 + 1] >> 2, data[i * 3 + 2] >> 2];
        }
        Ok(palette)
    }

    pub fn set(&mut self, index: usize, rgb: [u8; 3]) {
        self.colors[index] = rgb;
    }

    #[must_use]
    pub fn color(&self, index: usize) -> [u8; 3] {
        self.colors[index]
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for Palette {
    type Output = [u8; 3];
    fn index(&self, index: usize) -> &[u8; 3] {
        &self.colors[index]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteError {
    InvalidSize(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dim {
    pub width: u16,
    pub height: u16,
}

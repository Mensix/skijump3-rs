#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color([u8; 3]);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }
    pub fn r(&self) -> u8 { self.0[0] }
    pub fn g(&self) -> u8 { self.0[1] }
    pub fn b(&self) -> u8 { self.0[2] }
}

impl From<[u8; 3]> for Color {
    fn from(arr: [u8; 3]) -> Self {
        Self(arr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    colors: [[u8; 3]; 256],
}

impl Palette {
    pub fn new() -> Self {
        Self { colors: [[0; 3]; 256] }
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

    pub fn get(&self, index: usize) -> Color {
        Color(self.colors[index])
    }

    pub fn color(&self, index: usize) -> [u8; 3] {
        self.colors[index]
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for Palette {
    type Output = [u8; 3];
    fn index(&self, index: usize) -> &[u8; 3] {
        &self.colors[index]
    }
}

impl std::ops::IndexMut<usize> for Palette {
    fn index_mut(&mut self, index: usize) -> &mut [u8; 3] {
        &mut self.colors[index]
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
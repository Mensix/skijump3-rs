use sdl3::pixels::Color;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn transparent() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    pub const fn from_rgb6(r6: u8, g6: u8, b6: u8) -> Self {
        Self {
            r: (r6 as u32 * 255 / 63) as u8,
            g: (g6 as u32 * 255 / 63) as u8,
            b: (b6 as u32 * 255 / 63) as u8,
            a: 255,
        }
    }

    pub fn to_sdl(&self) -> Color {
        Color::RGBA(self.r, self.g, self.b, self.a)
    }
}

impl Hash for Rgba {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(u32::from_le_bytes([self.r, self.g, self.b, self.a]));
    }
}

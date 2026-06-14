use crate::color::Rgba;

pub const PALETTE_SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaletteIndex(pub u8);

impl PaletteIndex {
    pub const TRANSPARENT: Self = Self(0);

    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    rgba: [Rgba; PALETTE_SIZE],
}

impl Palette {
    #[must_use]
    pub fn from_rgb6_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != PALETTE_SIZE * 3 {
            return None;
        }

        let mut rgba = [Rgba::transparent(); PALETTE_SIZE];
        for (idx, color) in rgba.iter_mut().enumerate() {
            if idx == PaletteIndex::TRANSPARENT.value() as usize {
                continue;
            }
            let off = idx * 3;
            *color = Rgba::from_rgb6(bytes[off], bytes[off + 1], bytes[off + 2]);
        }

        Some(Self { rgba })
    }

    #[must_use]
    pub fn color(&self, index: PaletteIndex) -> Rgba {
        self.rgba[index.value() as usize]
    }

    #[must_use]
    pub fn rgba_bytes(&self, index: PaletteIndex) -> [u8; 4] {
        let color = self.color(index);
        [color.r, color.g, color.b, color.a]
    }
}

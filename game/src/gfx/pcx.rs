/// Minimal 256-colour palette for PCX file parsing.
/// Each RGB channel is stored as 6-bit (0-63).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcxPalette {
    data: [u8; 768],
}

impl PcxPalette {
    /// Create a PcxPalette from pre-converted 6-bit values (e.g. palette.toml).
    pub fn from_6bit_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 768 {
            return Err(format!(
                "Palette: expected 768 bytes, got {}",
                bytes.len()
            ));
        }
        let mut data = [0u8; 768];
        data.copy_from_slice(bytes);
        Ok(Self { data })
    }

    /// Return the 6-bit RGB triple for palette entry `idx`.
    pub fn color(&self, idx: usize) -> [u8; 3] {
        let off = idx * 3;
        [self.data[off], self.data[off + 1], self.data[off + 2]]
    }

    /// Overwrite palette entry `idx` with a 6-bit RGB triple.
    pub fn set(&mut self, idx: usize, rgb: [u8; 3]) {
        let off = idx * 3;
        self.data[off] = rgb[0];
        self.data[off + 1] = rgb[1];
        self.data[off + 2] = rgb[2];
    }
}

// PCX parser (from_pcx_bytes, PcxParser, DecodedPcx) kept for test fixtures only.
#[cfg(test)]
mod tests {
    use super::*;

    const PCX_HEADER_SIZE: usize = 128;

    fn scale_6bit(v: u8) -> u8 {
        (u32::from(v) * 255 / 63) as u8
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DecodedPcx {
        pub pixels: Vec<u8>,
        pub rgba_pixels: Vec<u8>,
        pub palette: PcxPalette,
        pub width: u16,
        pub height: u16,
    }

    pub struct PcxParser;

    impl PcxParser {
        fn rle_decode(data: &[u8], total_pixels: usize) -> Vec<u8> {
            let mut pixels = Vec::with_capacity(total_pixels);
            let mut i = 0;

            while pixels.len() < total_pixels && i < data.len() {
                let b1 = data[i];
                i += 1;

                if b1 >= 192 {
                    if i >= data.len() {
                        break;
                    }
                    let count = (b1 - 192) as usize;
                    let b2 = data[i];
                    i += 1;

                    for _ in 0..count {
                        if pixels.len() < total_pixels {
                            pixels.push(b2);
                        }
                    }
                } else if pixels.len() < total_pixels {
                    pixels.push(b1);
                }
            }

            pixels
        }

        fn from_pcx_bytes(bytes: &[u8]) -> Result<PcxPalette, String> {
            if bytes.len() != 768 {
                return Err(format!(
                    "PCX palette: expected 768 bytes, got {}",
                    bytes.len()
                ));
            }
            let mut data = [0u8; 768];
            for (i, &b) in bytes.iter().enumerate() {
                data[i] = b >> 2;
            }
            Ok(PcxPalette { data })
        }

        fn indexed_to_rgba(pixels: &[u8], palette: &PcxPalette) -> Vec<u8> {
            let mut rgba = Vec::with_capacity(pixels.len() * 4);
            for &idx in pixels {
                if idx == 0 {
                    rgba.extend_from_slice(&[0, 0, 0, 0]);
                } else {
                    let [r6, g6, b6] = palette.color(idx as usize);
                    rgba.push(scale_6bit(r6));
                    rgba.push(scale_6bit(g6));
                    rgba.push(scale_6bit(b6));
                    rgba.push(255);
                }
            }
            rgba
        }

        pub fn parse(data: &[u8]) -> Result<DecodedPcx, String> {
            if data.len() <= PCX_HEADER_SIZE + 768 {
                return Err(format!(
                    "PCX file too small: {} bytes (expected > {})",
                    data.len(),
                    PCX_HEADER_SIZE + 769
                ));
            }

            let width = u16::from_le_bytes([data[8], data[9]]).wrapping_add(1);
            let height = u16::from_le_bytes([data[10], data[11]]).wrapping_add(1);
            let total_pixels = (width as usize) * (height as usize);

            let image_data = &data[PCX_HEADER_SIZE..];
            let pixels = Self::rle_decode(image_data, total_pixels);

            if pixels.len() != total_pixels {
                return Err(format!(
                    "PCX truncated: expected {total_pixels} pixels, decoded {}",
                    pixels.len()
                ));
            }

            let palette_data = &data[data.len() - 768..];
            let palette = Self::from_pcx_bytes(palette_data)
                .map_err(|e| format!("PCX palette: {e}"))?;

            let rgba_pixels = Self::indexed_to_rgba(&pixels, &palette);

            Ok(DecodedPcx {
                pixels,
                rgba_pixels,
                palette,
                width,
                height,
            })
        }
    }

    #[test]
    fn from_pcx_bytes_converts_8bit_to_6bit() {
        let mut raw = [0u8; 768];
        raw[0] = 252;
        raw[1] = 128;
        raw[2] = 0;
        raw[3] = 255;
        raw[4] = 255;
        raw[5] = 255;

        let pal = PcxParser::from_pcx_bytes(&raw).unwrap();
        assert_eq!(pal.color(0), [63, 32, 0]);
        assert_eq!(pal.color(1), [63, 63, 63]);
    }

    #[test]
    fn from_pcx_bytes_rejects_wrong_size() {
        assert!(PcxParser::from_pcx_bytes(&[0; 767]).is_err());
        assert!(PcxParser::from_pcx_bytes(&[0; 769]).is_err());
    }

    #[test]
    fn from_6bit_bytes_preserves_values() {
        let mut raw = [0u8; 768];
        raw[0] = 63;
        raw[1] = 32;
        raw[2] = 0;
        let pal = PcxPalette::from_6bit_bytes(&raw).unwrap();
        assert_eq!(pal.color(0), [63, 32, 0]);
    }
}

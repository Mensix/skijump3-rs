use engine::palette::Palette;

const PCX_HEADER_SIZE: usize = 128;

/// Convert a 6-bit palette value (0-63) to an 8-bit value (0-255).
fn scale_6bit(v: u8) -> u8 {
    (u32::from(v) * 255 / 63) as u8
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPcx {
    pub pixels: Vec<u8>,
    pub rgba_pixels: Vec<u8>,
    pub palette: Palette,
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

    fn indexed_to_rgba(pixels: &[u8], palette: &Palette) -> Vec<u8> {
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
        let palette =
            Palette::from_pcx_bytes(palette_data).map_err(|e| format!("PCX palette: {e:?}"))?;

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

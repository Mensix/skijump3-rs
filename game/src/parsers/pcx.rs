use engine::palette::Palette;
use crate::parsers::{AssetParser, ParseError};

const PCX_HEADER_SIZE: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPcx {
    pub pixels: Vec<u8>,
    pub palette: Palette,
    pub width: u16,
    pub height: u16,
}

pub struct PcxParser;

impl PcxParser {
    fn rle_decode(data: &[u8], total_pixels: usize) -> Result<Vec<u8>, ParseError> {
        let mut pixels = Vec::with_capacity(total_pixels);
        let mut i = 0;

        while pixels.len() < total_pixels && i < data.len() {
            let b1 = data[i];
            i += 1;

            if b1 >= 192 {
                if i >= data.len() {
                    break;
                }
                let count = (b1 - 192 + 1) as usize;
                let b2 = data[i];
                i += 1;

                for _ in 0..count {
                    if pixels.len() < total_pixels {
                        pixels.push(b2);
                    }
                }
            } else {
                if pixels.len() < total_pixels {
                    pixels.push(b1);
                }
            }
        }

        Ok(pixels)
    }
}

impl AssetParser<DecodedPcx> for PcxParser {
    fn parse(data: &[u8]) -> Result<DecodedPcx, ParseError> {
        if data.len() <= PCX_HEADER_SIZE + 768 {
            return Err(ParseError {
                message: format!(
                    "PCX file too small: {} bytes (expected > {})",
                    data.len(),
                    PCX_HEADER_SIZE + 769
                ),
                byte_offset: None,
            });
        }

        let width = u16::from_le_bytes([data[8], data[9]]).wrapping_add(1);
        let height = u16::from_le_bytes([data[10], data[11]]).wrapping_add(1);
        let total_pixels = (width as usize) * (height as usize);

        let image_data = &data[PCX_HEADER_SIZE..];
        let pixels = Self::rle_decode(image_data, total_pixels)?;

        if pixels.len() != total_pixels {
            return Err(ParseError {
                message: format!(
                    "PCX pixel count mismatch: got {}, expected {}",
                    pixels.len(),
                    total_pixels
                ),
                byte_offset: Some(PCX_HEADER_SIZE),
            });
        }

        let palette_data = &data[data.len() - 768..];
        let palette = Palette::from_pcx_bytes(palette_data).map_err(|_| ParseError {
            message: "Failed to parse PCX palette".to_string(),
            byte_offset: Some(data.len() - 768),
        })?;

        Ok(DecodedPcx {
            pixels,
            palette,
            width,
            height,
        })
    }

    fn validate(data: &[u8]) -> bool {
        if data.len() < PCX_HEADER_SIZE + 769 {
            return false;
        }
        data[0] == 10 && data[2] == 1 && (data[3] & 0x80) != 0
    }
}
use crate::parsers::{AssetParser, ParseError};
use engine::sprite::SpriteData;

pub struct AnimParser;

impl AssetParser<Vec<SpriteData>> for AnimParser {
    fn parse(data: &[u8]) -> Result<Vec<SpriteData>, ParseError> {
        let mut sprites = Vec::new();

        if data.len() < 2 {
            return Err(ParseError { message: "ANIM file too short".to_string(), byte_offset: None });
        }

        let mut pos = 0;

        loop {
            if pos >= data.len() {
                break;
            }

            let x = data[pos] as u16;
            let y = data[pos + 1] as u16;
            pos += 2;

            if x == 255 && y == 255 {
                break;
            }

            if pos + (x as usize) * (y as usize) + 2 >= data.len() {
                break;
            }

            let mut pixel_data = Vec::with_capacity((x as usize) * (y as usize));
            for yy in 0..y {
                for xx in 0..x {
                    let idx = pos + (yy as usize) * (x as usize) + (xx as usize);
                    if idx >= data.len() {
                        break;
                    }
                    let mut byte = data[idx];
                    if byte == 9 {
                        byte = 15;
                    }
                    pixel_data.push(byte);
                }
            }
            pos += (x as usize) * (y as usize);

            let center_x = data[pos] as i8;
            let center_y = data[pos + 1] as i8;
            pos += 2;

            sprites.push(SpriteData { data: pixel_data, width: x, height: y, center_x, center_y });
        }

        let temp_count = sprites.len();
        if temp_count >= 83 {
            let mut extra = Vec::new();
            for i in 72..84.min(sprites.len()) {
                let src = &sprites[i];
                let mut flipped = vec![0u8; src.data.len()];
                let w = src.width as usize;
                let h = src.height as usize;
                for yy in 0..h {
                    for xx in 0..w {
                        let src_idx = yy * w + xx;
                        let dst_idx = (h - 1 - yy) * w + xx;
                        flipped[dst_idx] = src.data[src_idx];
                    }
                }
                extra.push(SpriteData {
                    data: flipped,
                    width: src.width,
                    height: src.height,
                    center_x: src.center_x,
                    center_y: (h as i8 - 1) - src.center_y as i8,
                });
            }
            sprites.extend(extra);
        }

        Ok(sprites)
    }

    fn validate(data: &[u8]) -> bool {
        data.len() > 4
    }
}
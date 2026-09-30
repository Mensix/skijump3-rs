use std::io::Cursor;

use png::ColorType;

pub struct RgbaImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn load_png(data: &[u8]) -> Option<RgbaImage> {
    let decoder = png::Decoder::new(Cursor::new(data));
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info();
    if info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let (width, height) = (info.width, info.height);
    if width == 0 || height == 0 || width > u32::from(u16::MAX) || height > u32::from(u16::MAX) {
        return None;
    }
    let color_type = info.color_type;
    let palette: Vec<u8> = info
        .palette
        .clone()
        .map(|c| c.into_owned())
        .unwrap_or_default();
    let trns: Vec<u8> = info
        .trns
        .clone()
        .map(|c| c.into_owned())
        .unwrap_or_default();
    let buffer_len = (width as usize) * (height as usize);
    let mut raw = vec![0u8; reader.output_buffer_size()?];
    reader.next_frame(&mut raw).ok()?;
    let pixels = match color_type {
        ColorType::Rgba => {
            if raw.len() != buffer_len * 4 {
                return None;
            }
            raw
        }
        ColorType::Rgb => {
            if raw.len() != buffer_len * 3 {
                return None;
            }
            let mut rgba = Vec::with_capacity(buffer_len * 4);
            for chunk in raw.chunks_exact(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255);
            }
            rgba
        }
        ColorType::Grayscale => {
            if raw.len() != buffer_len {
                return None;
            }
            let mut rgba = Vec::with_capacity(buffer_len * 4);
            for &v in &raw {
                rgba.extend_from_slice(&[v, v, v, 255]);
            }
            rgba
        }
        ColorType::GrayscaleAlpha => {
            if raw.len() != buffer_len * 2 {
                return None;
            }
            let mut rgba = Vec::with_capacity(buffer_len * 4);
            for chunk in raw.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
            rgba
        }
        ColorType::Indexed => {
            if raw.len() != buffer_len {
                return None;
            }
            let mut rgba = Vec::with_capacity(buffer_len * 4);
            for &i in &raw {
                let o = (i as usize) * 3;
                let (r, g, b) = if o + 2 < palette.len() {
                    (palette[o], palette[o + 1], palette[o + 2])
                } else {
                    (0, 0, 0)
                };
                rgba.extend_from_slice(&[r, g, b, trns.get(i as usize).copied().unwrap_or(255)]);
            }
            rgba
        }
    };
    Some(RgbaImage {
        pixels,
        width,
        height,
    })
}

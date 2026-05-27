use crate::palette::Palette;

/// Convert a slice of indexed pixels to ABGR8888 RGBA bytes.
/// Index 0 → transparent `[0,0,0,0]`; other indices are opaque palette colours
/// scaled from 6‑bit to 8‑bit.
pub fn indexed_pixels_to_rgba(pixels: &[u8], palette: &Palette, out: &mut Vec<u8>) {
    out.reserve(pixels.len() * 4);
    for &idx in pixels {
        if idx == 0 {
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let [r6, g6, b6] = palette.color(idx as usize);
            out.push((r6 as u32 * 255 / 63) as u8);
            out.push((g6 as u32 * 255 / 63) as u8);
            out.push((b6 as u32 * 255 / 63) as u8);
            out.push(255);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_to_rgba_zero_transparent() {
        let mut palette = Palette::new();
        palette.set(0, [10, 20, 30]);
        let mut out = Vec::new();
        indexed_pixels_to_rgba(&[0, 0, 0], &palette, &mut out);
        assert_eq!(out, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn indexed_to_rgba_nonzero_opaque() {
        let mut palette = Palette::new();
        palette.set(7, [10, 20, 30]); // 6-bit values
        let mut out = Vec::new();
        indexed_pixels_to_rgba(&[7], &palette, &mut out);
        // 10*255/63 ≈ 40, 20*255/63 ≈ 80, 30*255/63 ≈ 121
        assert_eq!(out, vec![40, 80, 121, 255]);
    }

    #[test]
    fn indexed_to_rgba_mixed() {
        let mut palette = Palette::new();
        palette.set(1, [63, 0, 0]); // max red 6-bit
        palette.set(2, [0, 63, 0]); // max green
        let mut out = Vec::new();
        indexed_pixels_to_rgba(&[0, 1, 2], &palette, &mut out);
        // 0 → transparent, 1 → red-ish, 2 → green-ish
        assert_eq!(out.len(), 12);
        assert_eq!(&out[0..4], &[0, 0, 0, 0]); // idx 0
        assert_eq!(&out[4..8], &[255, 0, 0, 255]); // idx 1: 63*255/63 = 255
        assert_eq!(&out[8..12], &[0, 255, 0, 255]); // idx 2
    }

    #[test]
    fn indexed_to_rgba_empty_input() {
        let palette = Palette::new();
        let mut out = Vec::new();
        indexed_pixels_to_rgba(&[], &palette, &mut out);
        assert!(out.is_empty());
    }
}

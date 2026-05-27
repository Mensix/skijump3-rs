use crate::palette::Palette;
use crate::sprite::SpriteColorRemap;

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

/// Convert a full sprite's indexed pixels to RGBA, applying colour remapping.
/// The output is a complete RGBA buffer of size `width * height * 4` suitable
/// for use as a static texture.
///
/// *   Source index `0` → fully transparent.
/// *   Non-zero source indices are first run through `remap.map()`, then if the
///     remapped index is `0` the pixel is transparent, otherwise it is opaque
///     using the current palette.
pub fn remapped_sprite_to_rgba(
    pixels: &[u8],
    width: u16,
    height: u16,
    palette: &Palette,
    remap: &SpriteColorRemap,
    out: &mut Vec<u8>,
) {
    let count = width as usize * height as usize;
    out.reserve(count * 4);
    for &pixel in pixels.iter().take(count) {
        if pixel == 0 {
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let final_pixel = remap.map(pixel);
            if final_pixel == 0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let [r6, g6, b6] = palette.color(final_pixel as usize);
                out.push((r6 as u32 * 255 / 63) as u8);
                out.push((g6 as u32 * 255 / 63) as u8);
                out.push((b6 as u32 * 255 / 63) as u8);
                out.push(255);
            }
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

    // ---------------------------------------------------------------------------
    // remapped_sprite_to_rgba tests
    // ---------------------------------------------------------------------------

    fn make_remap_palette() -> Palette {
        let mut p = Palette::new();
        p.set(5, [10, 20, 30]); // index 5 → non-zero test color
        p.set(7, [40, 50, 60]);
        p
    }

    #[test]
    fn remapped_sprite_zero_src_is_transparent() {
        let palette = make_remap_palette();
        let remap = SpriteColorRemap::new(vec![(3, 5)]);
        let mut out = Vec::new();
        remapped_sprite_to_rgba(&[0, 0, 0], 1, 3, &palette, &remap, &mut out);
        assert_eq!(out.len(), 12);
        assert_eq!(&out, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn remapped_sprite_remaps_to_opaque() {
        let palette = make_remap_palette();
        let remap = SpriteColorRemap::new(vec![(3, 5)]);
        let mut out = Vec::new();
        remapped_sprite_to_rgba(&[3], 1, 1, &palette, &remap, &mut out);
        // Index 3 → remap to 5 → palette[5] = [10,20,30] → 40,80,121
        assert_eq!(out, vec![40, 80, 121, 255]);
    }

    #[test]
    fn remapped_sprite_remap_to_zero_is_transparent() {
        let palette = make_remap_palette();
        let remap = SpriteColorRemap::new(vec![(3, 0)]);
        let mut out = Vec::new();
        remapped_sprite_to_rgba(&[3], 1, 1, &palette, &remap, &mut out);
        assert_eq!(out, vec![0, 0, 0, 0]);
    }

    #[test]
    fn remapped_sprite_unmapped_preserved() {
        let palette = make_remap_palette();
        let remap = SpriteColorRemap::new(vec![(3, 5)]);
        let mut out = Vec::new();
        remapped_sprite_to_rgba(&[7], 1, 1, &palette, &remap, &mut out);
        // Index 7 not remapped → palette[7] = [40,50,60] → 161,202,242
        assert_eq!(out, vec![161, 202, 242, 255]);
    }

    #[test]
    fn remapped_sprite_output_size() {
        let palette = make_remap_palette();
        let remap = SpriteColorRemap::new(vec![]);
        let mut out = Vec::new();
        remapped_sprite_to_rgba(&[0, 1, 2, 3], 2, 2, &palette, &remap, &mut out);
        assert_eq!(out.len(), 16); // 4 pixels * 4 bytes
    }

    #[test]
    fn remapped_sprite_empty_pixels_produces_no_output() {
        let palette = make_remap_palette();
        let remap = SpriteColorRemap::new(vec![]);
        let mut out = Vec::new();
        remapped_sprite_to_rgba(&[], 0, 0, &palette, &remap, &mut out);
        assert!(out.is_empty());
    }
}

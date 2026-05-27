use crate::consts::{HEIGHT, WIDTH};
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
            out.push((u32::from(r6) * 255 / 63) as u8);
            out.push((u32::from(g6) * 255 / 63) as u8);
            out.push((u32::from(b6) * 255 / 63) as u8);
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
                out.push((u32::from(r6) * 255 / 63) as u8);
                out.push((u32::from(g6) * 255 / 63) as u8);
                out.push((u32::from(b6) * 255 / 63) as u8);
                out.push(255);
            }
        }
    }
}

/// Convert a region from an indexed source buffer directly to RGBA bytes,
/// clipping to screen bounds. This is the direct-GPU counterpart of
/// `render_image_region_bitmap` — it avoids the intermediate `IndexedBitmap`
/// allocation by converting pixel-by-pixel to RGBA.
///
/// Returns `(screen_x, screen_y, vis_w, vis_h)` for the visible portion, or
/// `None` when the region is fully off-screen.
///
/// Source indices outside the source rect or past the source buffer bounds
/// produce transparent (zero) RGBA pixels.  Index `0` is also transparent.
pub fn indexed_region_to_rgba(
    src_pixels: &[u8],
    src_w: u32,
    src_h: u32,
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    request_w: u32,
    request_h: u32,
    palette: &Palette,
    out: &mut Vec<u8>,
) -> Option<(i32, i32, u32, u32)> {
    let vis_left = dst_x.max(0);
    let vis_top = dst_y.max(0);
    let vis_right = (dst_x + request_w as i32).min(WIDTH as i32);
    let vis_bottom = (dst_y + request_h as i32).min(HEIGHT as i32);
    let vis_w = (vis_right - vis_left).max(0) as u32;
    let vis_h = (vis_bottom - vis_top).max(0) as u32;

    if vis_w == 0 || vis_h == 0 {
        return None;
    }

    let src_off_x = vis_left - dst_x;
    let src_off_y = vis_top - dst_y;

    let output_len = (vis_w * vis_h * 4) as usize;
    out.reserve(output_len);
    let src_w_i32 = src_w as i32;
    let src_h_i32 = src_h as i32;

    for dy in 0..vis_h {
        let sy = src_y + src_off_y + dy as i32;
        for dx in 0..vis_w {
            let sx = src_x + src_off_x + dx as i32;
            if sx >= 0 && sx < src_w_i32 && sy >= 0 && sy < src_h_i32 {
                let src_idx = sy as usize * src_w as usize + sx as usize;
                if src_idx < src_pixels.len() {
                    let pixel = src_pixels[src_idx];
                    if pixel == 0 {
                        out.extend_from_slice(&[0, 0, 0, 0]);
                    } else {
                        let [r6, g6, b6] = palette.color(pixel as usize);
                        out.push((u32::from(r6) * 255 / 63) as u8);
                        out.push((u32::from(g6) * 255 / 63) as u8);
                        out.push((u32::from(b6) * 255 / 63) as u8);
                        out.push(255);
                    }
                    continue;
                }
            }
            out.extend_from_slice(&[0, 0, 0, 0]);
        }
    }

    Some((vis_left, vis_top, vis_w, vis_h))
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

    // ---------------------------------------------------------------------------
    // indexed_region_to_rgba tests
    // ---------------------------------------------------------------------------

    fn region_palette() -> Palette {
        let mut p = Palette::new();
        p.set(0, [0, 0, 0]);
        p.set(1, [63, 0, 0]); // full red
        p.set(2, [0, 63, 0]); // full green
        p
    }

    #[test]
    fn region_basic_full_visible() {
        let src = vec![1u8, 2, 0, 1]; // 2×2
        let mut out = Vec::new();
        let rect =
            indexed_region_to_rgba(&src, 2, 2, 0, 0, 0, 0, 2, 2, &region_palette(), &mut out);
        assert_eq!(rect, Some((0, 0, 2, 2)));
        // 1 → red: 63*255/63=255 → [255,0,0,255]
        // 2 → green: [0,255,0,255]
        // 0 → transparent
        assert_eq!(&out[0..4], &[255, 0, 0, 255]);
        assert_eq!(&out[4..8], &[0, 255, 0, 255]);
        assert_eq!(&out[8..12], &[0, 0, 0, 0]);
        assert_eq!(&out[12..16], &[255, 0, 0, 255]);
    }

    #[test]
    fn region_partial_clip_left_edge() {
        let src = vec![1u8; 4]; // 2×2
        let mut out = Vec::new();
        // Place at (-1, 0) so left 1 pixel is clipped
        let rect =
            indexed_region_to_rgba(&src, 2, 2, 0, 0, -1, 0, 2, 2, &region_palette(), &mut out);
        assert_eq!(rect, Some((0, 0, 1, 2))); // only 1 wide, 2 tall visible
        assert_eq!(out.len(), 8);
    }

    #[test]
    fn region_fully_offscreen_returns_none() {
        let src = vec![1u8; 4];
        let mut out = Vec::new();
        let rect =
            indexed_region_to_rgba(&src, 2, 2, 0, 0, -100, 0, 2, 2, &region_palette(), &mut out);
        assert!(rect.is_none());
        assert!(out.is_empty());
    }

    #[test]
    fn region_source_oob_is_transparent() {
        // src is 2×2 at source offset (1,1), request 2×2
        // Only the pixel at src (1,1) is valid; the rest are OOB → transparent
        let src = vec![0u8, 0, 0, 1]; // 2×2, pixel at (1,1) = 1
        let mut out = Vec::new();
        let rect =
            indexed_region_to_rgba(&src, 2, 2, 1, 1, 0, 0, 2, 2, &region_palette(), &mut out);
        assert_eq!(rect, Some((0, 0, 2, 2)));
        // Only the bottom-right pixel (1,1) should be non-transparent
        // Pixel (0,0): src (1,1) = 1 → red
        assert_eq!(&out[0..4], &[255, 0, 0, 255]);
        // Pixel (1,0): src (2,1) — OOB → transparent
        assert_eq!(&out[4..8], &[0, 0, 0, 0]);
        // Pixel (0,1): src (1,2) — OOB → transparent
        assert_eq!(&out[8..12], &[0, 0, 0, 0]);
        // Pixel (1,1): src (2,2) — OOB → transparent
        assert_eq!(&out[12..16], &[0, 0, 0, 0]);
    }

    #[test]
    fn region_zero_size_returns_none() {
        let src = vec![];
        let mut out = Vec::new();
        let rect =
            indexed_region_to_rgba(&src, 0, 0, 0, 0, 0, 0, 0, 0, &region_palette(), &mut out);
        assert!(rect.is_none());
    }

    #[test]
    fn region_zero_alpha_index_is_transparent() {
        // Index 0 is transparent even though palette has a colour at 0
        let mut pal = Palette::new();
        pal.set(0, [63, 63, 63]); // white at index 0 — should still be transparent
        pal.set(1, [63, 63, 63]); // white at index 1 — should be opaque
        let src = vec![0u8, 1]; // 1×2
        let mut out = Vec::new();
        indexed_region_to_rgba(&src, 2, 1, 0, 0, 0, 0, 2, 1, &pal, &mut out);
        // Index 0 → transparent
        assert_eq!(&out[0..4], &[0, 0, 0, 0]);
        // Index 1 → white after 6-bit scaling
        assert_eq!(&out[4..8], &[255, 255, 255, 255]);
    }

    #[test]
    fn region_clips_to_screen_bounds() {
        let src = vec![1u8; 16]; // 4×4
        let mut out = Vec::new();
        let screen_w = WIDTH as i32;
        let screen_h = HEIGHT as i32;
        // Place at bottom-right corner, partially off-screen
        let rect = indexed_region_to_rgba(
            &src,
            4,
            4,
            0,
            0,
            screen_w - 2,
            screen_h - 2,
            4,
            4,
            &region_palette(),
            &mut out,
        );
        // Only 2×2 visible
        assert_eq!(rect, Some((screen_w - 2, screen_h - 2, 2, 2)));
        assert_eq!(out.len(), 2 * 2 * 4);
    }

    #[test]
    fn region_src_oob_outside_src_rect_is_transparent() {
        // src is 3×3, request region extends beyond src dimensions.
        // src pixels: only src[4] (row 1, col 1) is set to 5 (red).
        let mut pal = Palette::new();
        pal.set(5, [63, 0, 0]); // full red in 6-bit
        let mut src = vec![0u8; 9];
        src[4] = 5; // pixel at (1,1) in buffer
        let mut out = Vec::new();
        // Source region starts at (src_x=1, src_y=1) within the buffer,
        // giving a valid 2×2 sub-region.
        // Screen pixel (0,0) maps to buffer (1,1) = src[4] = 5 → red
        let rect = indexed_region_to_rgba(&src, 3, 3, 1, 1, 0, 0, 4, 4, &pal, &mut out);
        // Source region is 2×2 (buffer (1,1)..(2,2)).
        // Screen pixels mapping to OOB (beyond buffer) are transparent.
        assert_eq!(rect, Some((0, 0, 4, 4)));
        assert_eq!(&out[0..4], &[255, 0, 0, 255], "pixel at (0,0) is red");
        // All other pixels transparent (index 0 or OOB)
        for i in (4..out.len()).step_by(4) {
            assert_eq!(out[i + 3], 0, "pixel at byte {i} should be transparent");
        }
    }
}

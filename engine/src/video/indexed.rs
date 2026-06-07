use crate::palette::Palette;
use crate::sprite::SpriteColorRecolor;

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

/// Convert a full sprite's indexed pixels to RGBA, applying direct
/// colour recolor (source index → explicit RGBA) with palette fallback
/// for non-recolored indices.
/// The output is a complete RGBA buffer of size `width * height * 4` suitable
/// for use as a static texture.
///
/// *   Source index `0` → fully transparent.
/// *   Non-zero source indices are checked against `recolor`; if present the
///     RGBA value is used directly.
/// *   Otherwise the index is looked up in `palette` (6‑bit → 8‑bit, opaque).
pub fn recolored_sprite_to_rgba(
    pixels: &[u8],
    width: u16,
    height: u16,
    palette: &Palette,
    recolor: &SpriteColorRecolor,
    out: &mut Vec<u8>,
) {
    let count = width as usize * height as usize;
    out.reserve(count * 4);
    for &pixel in pixels.iter().take(count) {
        if pixel == 0 {
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else if let Some(rgba) = recolor.get(pixel) {
            out.push(rgba.r);
            out.push(rgba.g);
            out.push(rgba.b);
            out.push(rgba.a);
        } else {
            let [r6, g6, b6] = palette.color(pixel as usize);
            out.push((u32::from(r6) * 255 / 63) as u8);
            out.push((u32::from(g6) * 255 / 63) as u8);
            out.push((u32::from(b6) * 255 / 63) as u8);
            out.push(255);
        }
    }
}

/// Copy a region from an RGBA source buffer to an output RGBA buffer,
/// clipping to screen bounds.  Returns `(screen_x, screen_y, vis_w, vis_h)`
/// for the visible portion, or `None` when fully off-screen.
///
/// Unlike `indexed_region_to_rgba`, this does _not_ do any palette lookup —
/// the source is already RGBA (4 bytes per pixel).
pub fn rgba_region_to_rgba(
    src_pixels: &[u8],
    src_w: u32,
    src_h: u32,
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    request_w: u32,
    request_h: u32,
    out: &mut Vec<u8>,
) -> Option<(i32, i32, u32, u32)> {
    let vis_left = dst_x.max(0);
    let vis_top = dst_y.max(0);
    let vis_right = (dst_x + request_w as i32).min(crate::consts::WIDTH as i32);
    let vis_bottom = (dst_y + request_h as i32).min(crate::consts::HEIGHT as i32);
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
                let src_idx = (sy as usize * src_w as usize + sx as usize) * 4;
                let end = src_idx + 4;
                if end <= src_pixels.len() {
                    out.extend_from_slice(&src_pixels[src_idx..end]);
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
    use crate::color::Rgba;
    use crate::consts::{HEIGHT, WIDTH};

    fn make_recolor_palette() -> Palette {
        let mut p = Palette::new();
        p.set(5, [10, 20, 30]);
        p.set(7, [40, 50, 60]);
        p
    }

    fn recolor_blue() -> Rgba {
        Rgba::rgb(0, 0, 255)
    }

    fn recolor_green() -> Rgba {
        Rgba::rgb(0, 255, 0)
    }

    #[test]
    fn recolored_sprite_zero_is_transparent() {
        let palette = make_recolor_palette();
        let recolor = SpriteColorRecolor::new(vec![(3, recolor_blue())]);
        let mut out = Vec::new();
        recolored_sprite_to_rgba(&[0, 0, 0], 1, 3, &palette, &recolor, &mut out);
        assert_eq!(out.len(), 12);
        assert_eq!(&out, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn recolored_sprite_recolors_to_rgba() {
        let palette = make_recolor_palette();
        let recolor = SpriteColorRecolor::new(vec![(3, recolor_blue())]);
        let mut out = Vec::new();
        recolored_sprite_to_rgba(&[3], 1, 1, &palette, &recolor, &mut out);
        assert_eq!(out, vec![0, 0, 255, 255]);
    }

    #[test]
    fn recolored_sprite_unmapped_falls_back_to_palette() {
        let palette = make_recolor_palette();
        let recolor = SpriteColorRecolor::new(vec![(3, recolor_blue())]);
        let mut out = Vec::new();
        recolored_sprite_to_rgba(&[7], 1, 1, &palette, &recolor, &mut out);
        assert_eq!(out, vec![161, 202, 242, 255]);
    }

    #[test]
    fn recolored_sprite_mixed_recolor_and_palette() {
        let palette = make_recolor_palette();
        let recolor = SpriteColorRecolor::new(vec![(3, recolor_blue()), (5, recolor_green())]);
        let mut out = Vec::new();
        recolored_sprite_to_rgba(&[0, 3, 5, 7], 2, 2, &palette, &recolor, &mut out);
        assert_eq!(out.len(), 16);
        assert_eq!(&out[0..4], &[0, 0, 0, 0]);
        assert_eq!(&out[4..8], &[0, 0, 255, 255]);
        assert_eq!(&out[8..12], &[0, 255, 0, 255]);
        assert_eq!(&out[12..16], &[161, 202, 242, 255]);
    }

    #[test]
    fn recolored_sprite_empty_pixels() {
        let palette = make_recolor_palette();
        let recolor = SpriteColorRecolor::new(vec![]);
        let mut out = Vec::new();
        recolored_sprite_to_rgba(&[], 0, 0, &palette, &recolor, &mut out);
        assert!(out.is_empty());
    }

    fn make_rgba_pixel(r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
        vec![r, g, b, a]
    }

    fn two_by_two_rgba() -> Vec<u8> {
        let mut buf = Vec::with_capacity(16);
        buf.extend_from_slice(&make_rgba_pixel(255, 0, 0, 255));
        buf.extend_from_slice(&make_rgba_pixel(0, 255, 0, 255));
        buf.extend_from_slice(&make_rgba_pixel(0, 0, 255, 255));
        buf.extend_from_slice(&make_rgba_pixel(128, 128, 128, 255));
        buf
    }

    #[test]
    fn rgba_region_full_visible() {
        let src = two_by_two_rgba();
        let mut out = Vec::new();
        let rect = rgba_region_to_rgba(&src, 2, 2, 0, 0, 0, 0, 2, 2, &mut out);
        assert_eq!(rect, Some((0, 0, 2, 2)));
        assert_eq!(out.len(), 16);
        assert_eq!(&out[0..4], &[255, 0, 0, 255]);
        assert_eq!(&out[4..8], &[0, 255, 0, 255]);
        assert_eq!(&out[8..12], &[0, 0, 255, 255]);
        assert_eq!(&out[12..16], &[128, 128, 128, 255]);
    }

    #[test]
    fn rgba_region_partial_clip_left() {
        let src = two_by_two_rgba();
        let mut out = Vec::new();
        let rect = rgba_region_to_rgba(&src, 2, 2, 0, 0, -1, 0, 2, 2, &mut out);
        assert_eq!(rect, Some((0, 0, 1, 2)));
        assert_eq!(out.len(), 8);
        assert_eq!(&out[0..4], &[0, 255, 0, 255]);
        assert_eq!(&out[4..8], &[128, 128, 128, 255]);
    }

    #[test]
    fn rgba_region_fully_offscreen() {
        let src = two_by_two_rgba();
        let mut out = Vec::new();
        let rect = rgba_region_to_rgba(&src, 2, 2, 0, 0, -100, 0, 2, 2, &mut out);
        assert!(rect.is_none());
        assert!(out.is_empty());
    }

    #[test]
    fn rgba_region_source_oob_is_transparent() {
        let src = two_by_two_rgba();
        let mut out = Vec::new();
        let rect = rgba_region_to_rgba(&src, 2, 2, 1, 1, 0, 0, 2, 2, &mut out);
        assert_eq!(rect, Some((0, 0, 2, 2)));
        assert_eq!(out.len(), 16);
        assert_eq!(&out[0..4], &[128, 128, 128, 255]);
        for i in (4..out.len()).step_by(4) {
            assert_eq!(out[i + 3], 0, "pixel at byte {i} should be transparent");
        }
    }

    #[test]
    fn rgba_region_zero_size() {
        let src = two_by_two_rgba();
        let mut out = Vec::new();
        let rect = rgba_region_to_rgba(&src, 2, 2, 0, 0, 0, 0, 0, 0, &mut out);
        assert!(rect.is_none());
    }

    #[test]
    fn rgba_region_clips_to_screen_bounds() {
        let src = vec![128u8; 4 * 4 * 4];
        let mut out = Vec::new();
        let screen_w = WIDTH as i32;
        let screen_h = HEIGHT as i32;
        let rect =
            rgba_region_to_rgba(&src, 4, 4, 0, 0, screen_w - 2, screen_h - 2, 4, 4, &mut out);
        assert_eq!(rect, Some((screen_w - 2, screen_h - 2, 2, 2)));
        assert_eq!(out.len(), 2 * 2 * 4);
    }
}
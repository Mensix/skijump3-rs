use crate::consts::{TILE_H, TILE_W};
use crate::oxide::draw::DitherPattern;

/// Write bright RGBA pixels into a full-screen RGBA buffer at pattern-hit
/// positions within the given rect.  The rect is clipped to
/// `(0, 0, screen_w, screen_h)`.  Tiling always uses `TILE_W`/`TILE_H`
/// constants.
///
/// `DitherPattern::Shifted` uses tile offset (2, 7),
/// `DitherPattern::Normal` uses (0, 0).  When `is_box` is true only the
/// 1-pixel border of the rect is processed.
pub(super) fn dither_rect_rgba(
    rgba: &mut [u8],
    screen_w: usize,
    screen_h: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    bright_r: u8,
    bright_g: u8,
    bright_b: u8,
    is_box: bool,
    pattern: DitherPattern,
    pattern_pixels: &[u8],
) {
    let (shift_x, shift_y) = match pattern {
        DitherPattern::Shifted => (2, 7),
        DitherPattern::Normal => (0, 0),
    };
    let left = x.max(0) as usize;
    let top = y.max(0) as usize;
    let right = ((x + w).min(screen_w as i32)).max(0) as usize;
    let bottom = ((y + h).min(screen_h as i32)).max(0) as usize;
    if left >= right || top >= bottom {
        return;
    }
    let rect_w = right - left;
    let rect_h = bottom - top;
    let tw = TILE_W as usize;
    let th = TILE_H as usize;
    for py in 0..rect_h {
        for px in 0..rect_w {
            if is_box {
                let sx_i = (left + px) as i32;
                let sy_i = (top + py) as i32;
                let on_top = sy_i == y;
                let on_bottom = sy_i == y + h - 1;
                let on_left = sx_i == x;
                let on_right = sx_i == x + w - 1;
                if !on_top && !on_bottom && !on_left && !on_right {
                    continue;
                }
            }
            let sx = left + px;
            let sy = top + py;
            let ax = ((sx as i32 + shift_x) as usize) % tw;
            let ay = ((sy as i32 + shift_y) as usize) % th;
            let pi = ay * tw + ax;
            if pi < pattern_pixels.len() && pattern_pixels[pi] != 0 {
                let idx = (sy * screen_w + sx) * 4;
                if idx + 3 < rgba.len() {
                    rgba[idx] = bright_r;
                    rgba[idx + 1] = bright_g;
                    rgba[idx + 2] = bright_b;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{FILL_BRIGHTEN, SHADOW_PIXEL, TILE_H, TILE_W};
    use crate::oxide::draw::DitherPattern;

    fn make_pattern() -> Vec<u8> {
        let mut d = vec![0u8; (TILE_W * TILE_H) as usize];
        d[0] = 1;
        d
    }

    #[test]
    fn dither_rect_brightens_eligible_hit() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = SHADOW_PIXEL + 1 + FILL_BRIGHTEN;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            0,
            0,
            19,
            13,
            bright,
            bright,
            bright,
            false,
            DitherPattern::Normal,
            &pattern,
        );

        assert_eq!(rgba[0], bright);
        assert_eq!(rgba[1], bright);
        assert_eq!(rgba[2], bright);
        assert_eq!(rgba[3], 255);

        let idx_miss = 4;
        assert_eq!(rgba[idx_miss], 0);
        assert_eq!(rgba[idx_miss + 3], 0);
    }

    #[test]
    fn dither_rect_pattern_shifted() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = SHADOW_PIXEL + 1 + FILL_BRIGHTEN;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            0,
            0,
            19,
            13,
            bright,
            bright,
            bright,
            false,
            DitherPattern::Shifted,
            &pattern,
        );

        let idx_hit = (6 * screen_w + 17) * 4;
        assert_eq!(rgba[idx_hit], bright);
        assert_eq!(rgba[idx_hit + 3], 255);

        let idx_miss = 0;
        assert_eq!(rgba[idx_miss], 0);
        assert_eq!(rgba[idx_miss + 3], 0);
    }

    #[test]
    fn dither_rect_skips_non_hit() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = SHADOW_PIXEL + 1 + FILL_BRIGHTEN;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            1,
            0,
            19,
            13,
            bright,
            bright,
            bright,
            false,
            DitherPattern::Normal,
            &pattern,
        );

        let idx_miss = 4;
        assert_eq!(rgba[idx_miss], 0);
        assert_eq!(rgba[idx_miss + 3], 0);
    }

    #[test]
    fn dither_rect_box_handles_border_only() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let mut pattern = vec![0u8; (TILE_W * TILE_H) as usize];
        pattern[0] = 1;
        let bright = 248;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            0,
            0,
            19,
            13,
            bright,
            bright,
            bright,
            true,
            DitherPattern::Normal,
            &pattern,
        );

        let idx_corner = 0;
        assert_eq!(rgba[idx_corner], bright);
        assert_eq!(rgba[idx_corner + 3], 255);

        let idx_interior = (screen_w + 1) * 4;
        assert_eq!(rgba[idx_interior], 0);
        assert_eq!(rgba[idx_interior + 3], 0);
    }

    #[test]
    fn dither_rect_clips_offscreen() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = 248;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            -5,
            -5,
            19,
            13,
            bright,
            bright,
            bright,
            false,
            DitherPattern::Normal,
            &pattern,
        );

        assert_eq!(rgba[0], bright);
        assert_eq!(rgba[3], 255);

        let idx_outside = 14 * 4;
        assert_eq!(rgba[idx_outside], 0);
        assert_eq!(rgba[idx_outside + 3], 0);
    }

    #[test]
    fn dither_rect_zero_size_is_noop() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = 248;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            0,
            0,
            0,
            10,
            bright,
            bright,
            bright,
            false,
            DitherPattern::Normal,
            &pattern,
        );
        assert!(rgba.iter().all(|&b| b == 0));
    }

    #[test]
    fn dither_rect_box_negative_x_dithers_visible_right_edge() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let mut pattern = vec![0u8; (TILE_W * TILE_H) as usize];
        pattern[2] = 1;
        let bright = 248;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            -2,
            0,
            19,
            10,
            bright,
            bright,
            bright,
            true,
            DitherPattern::Normal,
            &pattern,
        );

        let idx0 = 0;
        assert_eq!(rgba[idx0 + 3], 0, "top-left clipped miss");

        let idx2 = 8;
        assert_eq!(rgba[idx2], bright, "top edge hit at x=2");
        assert_eq!(rgba[idx2 + 3], 255);

        let idx_interior = 5 * screen_w * 4;
        assert_eq!(
            rgba[idx_interior + 3],
            0,
            "interior pixel should be transparent"
        );
    }

    #[test]
    fn dither_rect_box_negative_y_dithers_visible_bottom_edge() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let mut pattern = vec![0u8; (TILE_W * TILE_H) as usize];
        pattern[7 * TILE_W as usize] = 1;
        let bright = 248;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            0,
            -5,
            10,
            19,
            bright,
            bright,
            bright,
            true,
            DitherPattern::Normal,
            &pattern,
        );

        let idx_hit = 7 * screen_w * 4;
        assert_eq!(rgba[idx_hit], bright, "top edge hit at y=7");
        assert_eq!(rgba[idx_hit + 3], 255);

        let idx_bottom = 14 * screen_w * 4;
        assert_eq!(
            rgba[idx_bottom + 3],
            0,
            "clipped bottom pixel should be transparent"
        );
    }

    #[test]
    fn dither_rect_fully_offscreen_is_noop() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![99u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = 248;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            -100,
            0,
            10,
            10,
            bright,
            bright,
            bright,
            false,
            DitherPattern::Normal,
            &pattern,
        );
        assert!(rgba.iter().all(|&b| b == 99));
    }

    #[test]
    fn dither_rect_full_screen_hit() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let mut pattern = vec![0u8; (TILE_W * TILE_H) as usize];
        pattern[0] = 1;
        let bright = 248;

        dither_rect_rgba(
            &mut rgba,
            screen_w,
            screen_h,
            0,
            0,
            screen_w as i32,
            screen_h as i32,
            bright,
            bright,
            bright,
            false,
            DitherPattern::Normal,
            &pattern,
        );

        for &(px, py) in &[
            (0usize, 0usize),
            (TILE_W as usize, 0usize),
            (0usize, TILE_H as usize),
            (TILE_W as usize * 2, TILE_H as usize * 2),
        ] {
            let idx = (py * screen_w + px) * 4;
            assert_eq!(rgba[idx], bright, "pixel ({px},{py}) R");
            assert_eq!(rgba[idx + 3], 255, "pixel ({px},{py}) A");
        }

        for &(px, py) in &[(1usize, 0usize), (0usize, 1usize), (5usize, 3usize)] {
            let idx = (py * screen_w + px) * 4;
            assert_eq!(rgba[idx + 3], 0, "pixel ({px},{py}) should be transparent");
        }
    }
}

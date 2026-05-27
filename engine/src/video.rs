use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::consts::{FILL_BRIGHTEN, HEIGHT, TARGET_FPS, TILE_H, TILE_W, WIDTH};
use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

pub struct Renderer {
    canvas: sdl2::render::WindowCanvas,
    overlay_rgba: Vec<u8>,
    palette: Palette,
    last_tick: Instant,
    frame_texture: Texture,
    textures: HashMap<TextureId, Texture>,
    next_texture_id: u32,
}

impl Renderer {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let video = sdl.video()?;

        let window = video
            .window("Ski Jump International v3", WIDTH * 2, HEIGHT * 2)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        canvas
            .set_logical_size(WIDTH, HEIGHT)
            .map_err(|e| e.to_string())?;

        let tc = canvas.texture_creator();
        let mut frame_texture = tc
            .create_texture(
                PixelFormatEnum::ABGR8888,
                sdl2::render::TextureAccess::Streaming,
                WIDTH,
                HEIGHT,
            )
            .map_err(|e| e.to_string())?;
        frame_texture.set_blend_mode(BlendMode::Blend);

        Ok(Self {
            canvas,
            overlay_rgba: vec![0u8; (WIDTH * HEIGHT * 4) as usize],
            palette: Palette::new(),
            last_tick: Instant::now(),
            frame_texture,
            textures: HashMap::new(),
            next_texture_id: 1,
        })
    }

    pub fn create_rgba_texture(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Result<TextureId, String> {
        let tc = self.canvas.texture_creator();
        let mut texture = tc
            .create_texture(
                PixelFormatEnum::ABGR8888,
                sdl2::render::TextureAccess::Static,
                width,
                height,
            )
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        texture
            .update(None, pixels, (width * 4) as usize)
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;

        let id = TextureId(self.next_texture_id);
        self.next_texture_id += 1;
        self.textures.insert(id, texture);
        Ok(id)
    }

    pub fn draw_texture(
        &mut self,
        id: TextureId,
        src: Option<Rect>,
        dst: Option<Rect>,
    ) -> Result<(), String> {
        let Some(texture) = self.textures.get(&id) else {
            return Err("TextureId not found".into());
        };
        self.canvas.copy(texture, src, dst)?;
        Ok(())
    }

    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    pub fn wait_frame(&mut self) {
        let elapsed = self.last_tick.elapsed();
        let frame_time = Duration::from_secs_f64(1.0 / TARGET_FPS as f64);
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
        self.last_tick = Instant::now();
    }

    // GPU frame layering API ------------------------------------------------

    /// Start a new GPU frame. Clears the canvas.
    pub fn begin_frame(&mut self) {
        self.canvas.clear();
    }

    /// Finish the frame and present to screen.
    pub fn end_frame(&mut self) {
        self.canvas.present();
    }

    /// Write bright RGBA pixels into `overlay_rgba` for a single rect,
    /// at positions where the dither pattern sprite has a non-zero pixel.
    /// The rect is clipped to screen bounds. When `is_box` is true, only
    /// the 1-pixel border is processed.
    /// Tiling always uses the original `TILE_W`/`TILE_H` constants,
    /// matching the legacy CPU FillArea behaviour.
    pub fn dither_overlay_rect(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: u8,
        is_box: bool,
        thing: u8,
        pattern: &[u8],
    ) -> Result<(), String> {
        let bright_idx = (color + FILL_BRIGHTEN) as usize;
        if bright_idx >= 256 {
            return Ok(());
        }
        let [r6, g6, b6] = self.palette.color(bright_idx);
        let r = (r6 as u32 * 255 / 63) as u8;
        let g = (g6 as u32 * 255 / 63) as u8;
        let b = (b6 as u32 * 255 / 63) as u8;
        dither_rect_rgba(
            &mut self.overlay_rgba,
            WIDTH as usize,
            HEIGHT as usize,
            x,
            y,
            w,
            h,
            r,
            g,
            b,
            is_box,
            thing,
            pattern,
        );
        Ok(())
    }

    /// Upload the current `overlay_rgba` as a transparent overlay via
    /// `frame_texture` and copy it to the canvas.  Clears `overlay_rgba`
    /// afterwards so the next frame starts fresh.
    pub fn flush_dither_overlay(&mut self) -> Result<(), String> {
        self.frame_texture
            .update(None, &self.overlay_rgba, (WIDTH * 4) as usize)
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;
        self.canvas.copy(&self.frame_texture, None, None)?;
        self.overlay_rgba.fill(0);
        Ok(())
    }

    // Indexed-color rect drawing for GPU path -------------------------------

    /// Draw a filled rectangle using a palette index.
    /// Clips to (WIDTH, HEIGHT). Negative w/h are treated as zero.
    pub fn draw_indexed_fill_rect(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: u8,
    ) -> Result<(), String> {
        let x = x.max(0);
        let y = y.max(0);
        let w = w.min(WIDTH as i32 - x).max(0);
        let h = h.min(HEIGHT as i32 - y).max(0);
        if w <= 0 || h <= 0 {
            return Ok(());
        }
        let [r6, g6, b6] = self.palette.color(color as usize);
        let r = (r6 as u32 * 255 / 63) as u8;
        let g = (g6 as u32 * 255 / 63) as u8;
        let b = (b6 as u32 * 255 / 63) as u8;
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(r, g, b));
        self.canvas.fill_rect(Rect::new(x, y, w as u32, h as u32))?;
        Ok(())
    }

    /// Draw a 1-pixel-wide outlined rectangle using a palette index.
    /// Implemented as four fill_rect calls (matches old CPU `Box` behavior).
    pub fn draw_indexed_box(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: u8,
    ) -> Result<(), String> {
        self.draw_indexed_fill_rect(x, y, w, 1, color)?;
        self.draw_indexed_fill_rect(x, y + h - 1, w, 1, color)?;
        self.draw_indexed_fill_rect(x, y, 1, h, color)?;
        self.draw_indexed_fill_rect(x + w - 1, y, 1, h, color)?;
        Ok(())
    }

    // Indexed overlay upload -------------------------------------------------

    /// Upload a small indexed pixel buffer as an ABGR8888 texture and draw it.
    /// Index 0 becomes fully transparent; other indices are opaque via the
    /// current palette. This creates and destroys a temporary texture each
    /// call -- acceptable for occasional use (text rendering, sprites).
    pub fn draw_indexed_overlay_pixels(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<(), String> {
        let mut rgba = Vec::with_capacity(pixels.len() * 4);
        for &idx in pixels {
            if idx == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let [r6, g6, b6] = self.palette.color(idx as usize);
                rgba.push((r6 as u32 * 255 / 63) as u8);
                rgba.push((g6 as u32 * 255 / 63) as u8);
                rgba.push((b6 as u32 * 255 / 63) as u8);
                rgba.push(255);
            }
        }

        let tc = self.canvas.texture_creator();
        let mut texture = tc
            .create_texture(
                PixelFormatEnum::ABGR8888,
                sdl2::render::TextureAccess::Static,
                width,
                height,
            )
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        texture
            .update(None, &rgba, (width * 4) as usize)
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;

        self.canvas
            .copy(&texture, None, Rect::new(x, y, width, height))?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Dither helpers (testable without SDL)
// ---------------------------------------------------------------------------

/// Write bright RGBA pixels into a full-screen RGBA buffer at pattern-hit
/// positions within the given rect.  The rect is clipped to
/// `(0, 0, screen_w, screen_h)`.  Tiling always uses `TILE_W`/`TILE_H`
/// constants, matching the legacy CPU FillArea behaviour.
///
/// `thing` controls the tile offset: `thing == 64` shifts by (2, 7),
/// anything else uses (0, 0).  When `is_box` is true only the 1-pixel
/// border of the rect is processed.
fn dither_rect_rgba(
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
    thing: u8,
    pattern: &[u8],
) {
    let (shift_x, shift_y) = if thing == 64 { (2, 7) } else { (0, 0) };
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
            if pi < pattern.len() && pattern[pi] != 0 {
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

    fn make_pattern() -> Vec<u8> {
        let mut d = vec![0u8; (TILE_W * TILE_H) as usize];
        d[0] = 1; // hit only at tile origin
        d
    }

    #[test]
    fn dither_rect_brightens_eligible_hit() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = SHADOW_PIXEL + 1 + FILL_BRIGHTEN; // 248

        // Rect at (0,0, 19,13) so tile (0,0) maps to pixel (0,0) which is a hit
        dither_rect_rgba(
            &mut rgba, screen_w, screen_h, 0, 0, 19, 13, bright, bright, bright, false, 63,
            &pattern,
        );

        // Pixel (0,0) should be brightened
        assert_eq!(rgba[0], bright);
        assert_eq!(rgba[1], bright);
        assert_eq!(rgba[2], bright);
        assert_eq!(rgba[3], 255);

        // Pixel (1,0) -- pattern miss (index 1) -- should be transparent
        let idx_miss = (0 * screen_w + 1) * 4;
        assert_eq!(rgba[idx_miss], 0);
        assert_eq!(rgba[idx_miss + 3], 0);
    }

    #[test]
    fn dither_rect_thing_64_shift() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let pattern = make_pattern();
        let bright = SHADOW_PIXEL + 1 + FILL_BRIGHTEN;

        // With thing=64, shift is (2,7).  Pixel at (17,6):
        //   ax = (17+2)%19 = 0, ay = (6+7)%13 = 0 -> hit
        dither_rect_rgba(
            &mut rgba, screen_w, screen_h, 0, 0, 19, 13, bright, bright, bright, false, 64,
            &pattern,
        );

        let idx_hit = (6 * screen_w + 17) * 4;
        assert_eq!(rgba[idx_hit], bright);
        assert_eq!(rgba[idx_hit + 3], 255);

        // Pixel at (0,0): ax = 2, ay = 7 -> miss -> transparent
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

        // Rect at (1,0, 19,13) -- tile origin (0,0) now maps to screen (1,0)
        // which is offset from the tile -> no hit
        dither_rect_rgba(
            &mut rgba, screen_w, screen_h, 1, 0, 19, 13, bright, bright, bright, false, 63,
            &pattern,
        );

        // Pixel (1,0): ax = 1%19 = 1, ay = 0 -> miss
        let idx_miss = (0 * screen_w + 1) * 4;
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

        // Box at (0,0, 19,13) -- only border pixels at tile-origin positions
        // get brightened.  The top-left corner pixel (0,0) is a hit.
        // The interior pixel (1,1) should remain transparent.
        dither_rect_rgba(
            &mut rgba, screen_w, screen_h, 0, 0, 19, 13, bright, bright, bright, true, 63, &pattern,
        );

        // Top-left corner is on border -> hit
        let idx_corner = 0;
        assert_eq!(rgba[idx_corner], bright);
        assert_eq!(rgba[idx_corner + 3], 255);

        // Interior pixel (1,1) is not on border -> should be transparent
        let idx_interior = (1 * screen_w + 1) * 4;
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

        // Rect at (-5, -5, 19, 13) -- clips to (0,0, 14,8)
        dither_rect_rgba(
            &mut rgba, screen_w, screen_h, -5, -5, 19, 13, bright, bright, bright, false, 63,
            &pattern,
        );

        // Only the clipped region (0..14, 0..8) should be touched.
        // Pixel (0,0) is a hit (tile origin) -> brightened
        assert_eq!(rgba[0], bright);
        assert_eq!(rgba[3], 255);

        // Pixel (14, 0) is outside the clipped rect -> untouched
        let idx_outside = (0 * screen_w + 14) * 4;
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
            &mut rgba, screen_w, screen_h, 0, 0, 0, 10, bright, bright, bright, false, 63, &pattern,
        );
        assert!(rgba.iter().all(|&b| b == 0));
    }

    #[test]
    fn dither_rect_box_negative_x_dithers_visible_right_edge() {
        let screen_w = 320usize;
        let screen_h = 200usize;
        let mut rgba = vec![0u8; screen_w * screen_h * 4];
        let mut pattern = vec![0u8; (TILE_W * TILE_H) as usize];
        // Make pixel at tile (2,0) non-zero (ax = 2 for x=0 with no shift)
        pattern[2] = 1;
        let bright = 248;

        // Box at (-2, 0, 19, 10).  Clipped left edge at x=0.
        // Visible right-edge pixel (0, 0) maps to ax=(0+0)%19=0 -> miss
        // Visible right-edge pixel (1, 0) maps to ax=1 -> miss
        // Visible right-edge pixel (2, 0) maps to ax=2 -> hit
        // Left-edge of visible portion (x=0) is NOT the original box's left
        // edge (x=-2), so it's only part of border if it's top, bottom, or
        // the original box left edge.
        dither_rect_rgba(
            &mut rgba, screen_w, screen_h, -2, 0, 19, 10, bright, bright, bright, true, 63,
            &pattern,
        );

        // Pixel (0,0): visible, on original top edge -> border, ax=0 -> miss
        let idx0 = (0 * screen_w + 0) * 4;
        assert_eq!(rgba[idx0 + 3], 0, "top-left clipped miss");

        // Pixel (2,0): visible, on original top edge -> border, ax=2 -> hit
        let idx2 = (0 * screen_w + 2) * 4;
        assert_eq!(rgba[idx2], bright, "top edge hit at x=2");
        assert_eq!(rgba[idx2 + 3], 255);

        // Pixel (0, 5): interior of visible portion, but original left edge
        // at x=-2 is offscreen, so clipped pixel at x=0 is NOT on original
        // left border -> should NOT be brightened
        let idx_interior = (5 * screen_w + 0) * 4;
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
        // Make pixel at tile (0,7) non-zero (ay = 7 for y=0 with no shift)
        pattern[7 * TILE_W as usize] = 1;
        let bright = 248;

        // Box at (0, -5, 10, 19).  Clips to (0,0, 10, 14).
        // Pixel (0,0): on original top edge at y=-5 -> border, ay=0 -> miss
        // Pixel (0,7): on original top edge -> border, ay=7 -> hit
        // Pixel (0,14): on clipped bottom (y=14), but NOT original bottom
        // (y=-5+19-1=13) -> border only if on top/left/right edge.
        dither_rect_rgba(
            &mut rgba, screen_w, screen_h, 0, -5, 10, 19, bright, bright, bright, true, 63,
            &pattern,
        );

        // Pixel (0,7): visible, on original top edge -> border, ay=7 -> hit
        let idx_hit = (7 * screen_w + 0) * 4;
        assert_eq!(rgba[idx_hit], bright, "top edge hit at y=7");
        assert_eq!(rgba[idx_hit + 3], 255);

        // Pixel (0,14): visible, clipped bottom (y=14 is NOT y+h-1=13),
        // not on left/right/top edge of original box -> should be transparent
        let idx_bottom = (14 * screen_w + 0) * 4;
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
            &mut rgba, screen_w, screen_h, -100, 0, 10, 10, bright, bright, bright, false, 63,
            &pattern,
        );
        assert!(rgba.iter().all(|&b| b == 99));
    }

    #[test]
    fn dither_rect_full_screen_hit() {
        // Verify that every pixel at tile-origin positions across the
        // full screen gets brightened.
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
            63,
            &pattern,
        );

        // Check a few tile-origin positions
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

        // A few non-hit positions should be transparent
        for &(px, py) in &[(1usize, 0usize), (0usize, 1usize), (5usize, 3usize)] {
            let idx = (py * screen_w + px) * 4;
            assert_eq!(rgba[idx + 3], 0, "pixel ({px},{py}) should be transparent");
        }
    }
}

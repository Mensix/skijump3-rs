use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::consts::{FILL_BRIGHTEN, HEIGHT, TARGET_FPS, WIDTH};
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
    /// Caller draws background textures, then `draw_indexed_overlay`.
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
        pattern_w: u32,
        pattern_h: u32,
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
            pattern_w as usize,
            pattern_h as usize,
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
    /// call — acceptable for occasional use (text rendering, sprites).
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
/// `(0, 0, screen_w, screen_h)`.
///
/// `thing` controls the tile offset: `thing == 64` shifts by (2, 7),
/// anything else uses (0, 0).  When `is_box` is true only the 1-pixel
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
    pattern_w: usize,
    pattern_h: usize,
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
    let pattern_w = pattern_w.max(1);
    let pattern_h = pattern_h.max(1);
    for py in 0..rect_h {
        for px in 0..rect_w {
            if is_box {
                let on_top = (top + py) == (y as usize);
                let on_bottom = (top + py) == ((y + h - 1) as usize);
                let on_left = (left + px) == (x as usize);
                let on_right = (left + px) == ((x + w - 1) as usize);
                if !on_top && !on_bottom && !on_left && !on_right {
                    continue;
                }
            }
            let sx = left + px;
            let sy = top + py;
            let ax = ((sx as i32 + shift_x) as usize) % pattern_w;
            let ay = ((sy as i32 + shift_y) as usize) % pattern_h;
            let pi = ay * pattern_w + ax;
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
            63,
            &pattern,
            TILE_W as usize,
            TILE_H as usize,
        );

        // Pixel (0,0) should be brightened
        assert_eq!(rgba[0], bright);
        assert_eq!(rgba[1], bright);
        assert_eq!(rgba[2], bright);
        assert_eq!(rgba[3], 255);

        // Pixel (1,0) — pattern miss (index 1) — should be transparent
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
        //   ax = (17+2)%19 = 0, ay = (6+7)%13 = 0 → hit
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
            64,
            &pattern,
            TILE_W as usize,
            TILE_H as usize,
        );

        let idx_hit = (6 * screen_w + 17) * 4;
        assert_eq!(rgba[idx_hit], bright);
        assert_eq!(rgba[idx_hit + 3], 255);

        // Pixel at (0,0): ax = 2, ay = 7 → miss → transparent
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

        // Rect at (1,0, 19,13) — tile origin (0,0) now maps to screen (1,0)
        // which is offset from the tile → no hit
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
            63,
            &pattern,
            TILE_W as usize,
            TILE_H as usize,
        );

        // Pixel (1,0): ax = 1%19 = 1, ay = 0 → miss
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

        // Box at (0,0, 19,13) — only border pixels at tile-origin positions
        // get brightened.  The top-left corner pixel (0,0) is a hit.
        // The interior pixel (1,1) should remain transparent.
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
            63,
            &pattern,
            TILE_W as usize,
            TILE_H as usize,
        );

        // Top-left corner is on border → hit
        let idx_corner = 0;
        assert_eq!(rgba[idx_corner], bright);
        assert_eq!(rgba[idx_corner + 3], 255);

        // Interior pixel (1,1) is not on border → should be transparent
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

        // Rect at (-5, -5, 19, 13) — clips to (0,0, 14,8)
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
            63,
            &pattern,
            TILE_W as usize,
            TILE_H as usize,
        );

        // Only the clipped region (0..14, 0..8) should be touched.
        // Pixel (0,0) is a hit (tile origin) → brightened
        assert_eq!(rgba[0], bright);
        assert_eq!(rgba[3], 255);

        // Pixel (14, 0) is outside the clipped rect → untouched
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
            63,
            &pattern,
            TILE_W as usize,
            TILE_H as usize,
        );
        assert!(rgba.iter().all(|&b| b == 0));
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
            63,
            &pattern,
            TILE_W as usize,
            TILE_H as usize,
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
            TILE_W as usize,
            TILE_H as usize,
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

use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{BlendMode, ScaleMode, Texture, WindowCanvas};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::color::Rgba;
use crate::consts::TARGET_FPS;
use crate::consts::{HEIGHT, WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

impl Rect {
    #[must_use]
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    #[must_use]
    pub const fn x(&self) -> i32 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> i32 {
        self.y
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.w
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.h
    }
}

impl From<Rect> for sdl3::rect::Rect {
    fn from(r: Rect) -> Self {
        sdl3::rect::Rect::new(r.x, r.y, r.w, r.h)
    }
}

impl From<Rect> for sdl3::render::FRect {
    fn from(r: Rect) -> Self {
        sdl3::render::FRect::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32)
    }
}

pub struct Renderer {
    canvas: WindowCanvas,
    scratch_rgba: Vec<u8>,
    last_tick: Instant,
    scratch_texture: Texture,
    textures: HashMap<TextureId, Texture>,
    next_texture_id: u32,
}

impl Renderer {
    pub fn new(sdl: &sdl3::Sdl) -> Result<Self, String> {
        let video = sdl.video().map_err(|e| e.to_string())?;

        let window = video
            .window("Ski Jump International v3", WIDTH * 2, HEIGHT * 2)
            .resizable()
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas();

        canvas
            .set_logical_size(
                WIDTH,
                HEIGHT,
                sdl3::sys::render::SDL_RendererLogicalPresentation::STRETCH,
            )
            .map_err(|e| e.to_string())?;

        let tc = canvas.texture_creator();
        let mut scratch_texture = tc
            .create_texture(
                PixelFormat::ABGR8888,
                sdl3::render::TextureAccess::Streaming,
                WIDTH,
                HEIGHT,
            )
            .map_err(|e| e.to_string())?;
        scratch_texture.set_blend_mode(BlendMode::Blend);
        scratch_texture.set_scale_mode(ScaleMode::Nearest);

        Ok(Self {
            canvas,
            scratch_rgba: Vec::new(),
            last_tick: Instant::now(),
            scratch_texture,
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
                PixelFormat::ABGR8888,
                sdl3::render::TextureAccess::Static,
                width,
                height,
            )
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        texture.set_scale_mode(ScaleMode::Nearest);
        texture
            .update(None, pixels, (width * 4) as usize)
            .map_err(|e: sdl3::render::UpdateTextureError| e.to_string())?;

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
        self.canvas
            .copy(
                texture,
                src.map(sdl3::render::FRect::from),
                dst.map(sdl3::render::FRect::from),
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn wait_frame(&mut self) {
        let frame_time = Duration::from_secs_f64(1.0 / f64::from(TARGET_FPS));
        let elapsed = self.last_tick.elapsed();
        if elapsed < frame_time {
            let remaining = frame_time - elapsed;
            if remaining > Duration::from_millis(2) {
                std::thread::sleep(remaining - Duration::from_millis(2));
            }
            while self.last_tick.elapsed() < frame_time {
                std::hint::spin_loop();
            }
        }
        self.last_tick = Instant::now();
    }

    // GPU frame layering API ------------------------------------------------

    pub fn begin_frame(&mut self) {
        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        self.canvas.clear();
    }

    pub fn end_frame(&mut self) {
        let _ = self.canvas.present();
    }

    // RGBA texture helpers --------------------------------------------------

    pub fn create_pattern_texture(
        &mut self,
        pattern_pixels: &[u8],
        tile_w: u32,
        tile_h: u32,
    ) -> Result<TextureId, String> {
        let rgba = pattern_pixels_to_rgba(pattern_pixels, tile_w, tile_h);
        self.create_rgba_texture(&rgba, tile_w, tile_h)
    }

    pub fn draw_tiled_pattern(
        &mut self,
        rect: Rect,
        color: Rgba,
        pattern_id: TextureId,
        tile_w: u32,
        tile_h: u32,
    ) -> Result<(), String> {
        let (tint_r, tint_g, tint_b) = pattern_tint(color);

        // Save old colour tint and set new one
        let (old_r, old_g, old_b) = {
            let Some(t) = self.textures.get_mut(&pattern_id) else {
                return Err("Pattern texture not found".into());
            };
            let old = t.color_mod();
            t.set_color_mod(tint_r, tint_g, tint_b);
            old
        };

        // Tile the pattern (immutable borrow for copy)
        let Some(texture) = self.textures.get(&pattern_id) else {
            return Err("Pattern texture not found".into());
        };

        for_tiled_segments(rect, tile_w, tile_h, |src, dst| {
            self.canvas
                .copy(
                    texture,
                    Some(sdl3::render::FRect::from(src)),
                    Some(sdl3::render::FRect::from(dst)),
                )
                .map_err(|e| e.to_string())
        })?;

        if let Some(t) = self.textures.get_mut(&pattern_id) {
            t.set_color_mod(old_r, old_g, old_b);
        }

        Ok(())
    }

    pub fn draw_rgba_region_pixels(
        &mut self,
        pixels: &[u8],
        src_w: u32,
        src_h: u32,
        src_x: i32,
        src_y: i32,
        dst_x: i32,
        dst_y: i32,
        w: u32,
        h: u32,
    ) -> Result<(), String> {
        self.scratch_rgba.clear();
        let Some((vis_left, vis_top, vis_w, vis_h)) = rgba_region_to_rgba(
            pixels,
            src_w,
            src_h,
            src_x,
            src_y,
            dst_x,
            dst_y,
            w,
            h,
            &mut self.scratch_rgba,
        ) else {
            return Ok(());
        };
        self.scratch_texture
            .update(
                Some(sdl3::rect::Rect::from(Rect::new(0, 0, vis_w, vis_h))),
                &self.scratch_rgba,
                (vis_w * 4) as usize,
            )
            .map_err(|e: sdl3::render::UpdateTextureError| e.to_string())?;
        self.canvas
            .copy(
                &self.scratch_texture,
                Some(sdl3::render::FRect::from(Rect::new(0, 0, vis_w, vis_h))),
                Some(sdl3::render::FRect::from(Rect::new(
                    vis_left, vis_top, vis_w, vis_h,
                ))),
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // RGBA primitives -------------------------------------------------------

    pub fn draw_fill_rect(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Rgba,
    ) -> Result<(), String> {
        let x = x.max(0);
        let y = y.max(0);
        let w = w.min(WIDTH as i32 - x).max(0);
        let h = h.min(HEIGHT as i32 - y).max(0);
        if w <= 0 || h <= 0 {
            return Ok(());
        }
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas
            .fill_rect(Some(sdl3::render::FRect::new(
                x as f32,
                y as f32,
                w as f32,
                h as f32,
            )))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn draw_box(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) -> Result<(), String> {
        self.draw_fill_rect(x, y, w, 1, color)?;
        self.draw_fill_rect(x, y + h - 1, w, 1, color)?;
        self.draw_fill_rect(x, y, 1, h, color)?;
        self.draw_fill_rect(x + w - 1, y, 1, h, color)?;
        Ok(())
    }
}

/// Copy a region from an RGBA source buffer to an output RGBA buffer,
/// clipping to screen bounds.  Returns `(screen_x, screen_y, vis_w, vis_h)`
/// for the visible portion, or `None` when fully off-screen.
fn rgba_region_to_rgba(
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

fn pattern_pixels_to_rgba(pattern_pixels: &[u8], tile_w: u32, tile_h: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((tile_w * tile_h * 4) as usize);
    for &pixel in pattern_pixels {
        if pixel != 0 {
            rgba.extend_from_slice(&[255, 255, 255, 255]);
        } else {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        }
    }
    rgba
}

fn pattern_tint(color: Rgba) -> (u8, u8, u8) {
    let brighten = |c: u8| (u32::from(c) * 130 / 100).min(255) as u8;
    (brighten(color.r), brighten(color.g), brighten(color.b))
}

fn for_tiled_segments(
    rect: Rect,
    tile_w: u32,
    tile_h: u32,
    mut f: impl FnMut(Rect, Rect) -> Result<(), String>,
) -> Result<(), String> {
    let tw = tile_w as i32;
    let th = tile_h as i32;
    if tw <= 0 || th <= 0 {
        return Ok(());
    }
    let end_x = (rect.x() + rect.width() as i32).min(WIDTH as i32);
    let end_y = (rect.y() + rect.height() as i32).min(HEIGHT as i32);
    let mut ty = rect.y().max(0);
    while ty < end_y {
        let src_y = ty.rem_euclid(th);
        let vis_h = (th - src_y).min(end_y - ty) as u32;
        let mut tx = rect.x().max(0);
        while tx < end_x {
            let src_x = tx.rem_euclid(tw);
            let vis_w = (tw - src_x).min(end_x - tx) as u32;
            f(
                Rect::new(src_x, src_y, vis_w, vis_h),
                Rect::new(tx, ty, vis_w, vis_h),
            )?;
            tx += vis_w as i32;
        }
        ty += vis_h as i32;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let rect = rgba_region_to_rgba(
            &src,
            4,
            4,
            0,
            0,
            WIDTH as i32 - 2,
            HEIGHT as i32 - 2,
            4,
            4,
            &mut out,
        );
        assert_eq!(rect, Some((WIDTH as i32 - 2, HEIGHT as i32 - 2, 2, 2)));
        assert_eq!(out.len(), 2 * 2 * 4);
    }
}

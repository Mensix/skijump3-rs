#[cfg(not(target_arch = "wasm32"))]
use sdl2::pixels::{Color, PixelFormatEnum};
#[cfg(not(target_arch = "wasm32"))]
pub use sdl2::rect::Rect;
#[cfg(not(target_arch = "wasm32"))]
use sdl2::render::{BlendMode, Texture};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::Clamped;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use crate::color::Rgba;
#[cfg(not(target_arch = "wasm32"))]
use crate::consts::TARGET_FPS;
use crate::consts::{HEIGHT, WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
pub struct Renderer {
    canvas: sdl2::render::WindowCanvas,
    scratch_rgba: Vec<u8>,
    last_tick: Instant,
    scratch_texture: Texture,
    textures: HashMap<TextureId, Texture>,
    next_texture_id: u32,
}

#[cfg(not(target_arch = "wasm32"))]
impl Renderer {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let video = sdl.video()?;

        let window = video
            .window("Ski Jump International v3", WIDTH * 2, HEIGHT * 2)
            .resizable()
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
        let mut scratch_texture = tc
            .create_texture(
                PixelFormatEnum::ABGR8888,
                sdl2::render::TextureAccess::Streaming,
                WIDTH,
                HEIGHT,
            )
            .map_err(|e| e.to_string())?;
        scratch_texture.set_blend_mode(BlendMode::Blend);

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

    /// Start a new GPU frame. Clears the canvas.
    pub fn begin_frame(&mut self) {
        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        self.canvas.clear();
    }

    /// Finish the frame and present to screen.
    pub fn end_frame(&mut self) {
        self.canvas.present();
    }

    // Pattern-tile support ------------------------------------------------

    /// Create a 19×13 white-on-transparent pattern texture from indexed
    /// sprite pixel data.  Non-zero bytes become white (255,255,255,255);
    /// zero bytes become transparent (0,0,0,0).  Use `draw_tiled_pattern`
    /// to tile this over a rectangle with a given colour.
    pub fn create_pattern_texture(
        &mut self,
        pattern_pixels: &[u8],
        tile_w: u32,
        tile_h: u32,
    ) -> Result<TextureId, String> {
        let mut rgba = Vec::with_capacity((tile_w * tile_h * 4) as usize);
        for &pixel in pattern_pixels {
            if pixel != 0 {
                rgba.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
        self.create_rgba_texture(&rgba, tile_w, tile_h)
    }

    /// Tile a white-on-transparent pattern texture over `rect`, tinted by
    /// `color` brightened by 130 %.  The fill must already be drawn
    /// underneath – this only adds the pattern overlay.  Tiling is
    /// screen-aligned (same as the old dither behaviour).
    pub fn draw_tiled_pattern(
        &mut self,
        rect: Rect,
        color: Rgba,
        pattern_id: TextureId,
        tile_w: u32,
        tile_h: u32,
    ) -> Result<(), String> {
        let brighten = |c: u8| (u32::from(c) * 130 / 100).min(255) as u8;
        let tint = Color::RGBA(brighten(color.r), brighten(color.g), brighten(color.b), 255);

        // Save old colour tint and set new one
        let (old_r, old_g, old_b) = {
            let Some(t) = self.textures.get_mut(&pattern_id) else {
                return Err("Pattern texture not found".into());
            };
            let old = t.color_mod();
            t.set_color_mod(tint.r, tint.g, tint.b);
            old
        };

        // Tile the pattern (immutable borrow for copy)
        let Some(texture) = self.textures.get(&pattern_id) else {
            return Err("Pattern texture not found".into());
        };
        let tw = tile_w as i32;
        let th = tile_h as i32;
        let end_x = (rect.x() + rect.width() as i32).min(WIDTH as i32);
        let end_y = (rect.y() + rect.height() as i32).min(HEIGHT as i32);

        let mut ty = rect.y();
        while ty < end_y {
            let src_y = ty % th;
            let vis_h = (th - src_y).min(end_y - ty) as u32;

            let mut tx = rect.x();
            while tx < end_x {
                let src_x = tx % tw;
                let vis_w = (tw - src_x).min(end_x - tx) as u32;

                self.canvas.copy(
                    texture,
                    Rect::new(src_x.max(0), src_y.max(0), vis_w, vis_h),
                    Rect::new(tx.max(0), ty.max(0), vis_w, vis_h),
                )?;

                tx += vis_w as i32;
            }
            ty += vis_h as i32;
        }

        // Restore colour tint
        if let Some(t) = self.textures.get_mut(&pattern_id) {
            t.set_color_mod(old_r, old_g, old_b);
        }

        Ok(())
    }

    /// Draw a region of pre-computed RGBA pixels via the reusable scratch
    /// texture.  Clips to screen bounds.  Source pixels are 4 bytes per pixel
    /// (ABGR8888 order to match SDL2).
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
                Some(Rect::new(0, 0, vis_w, vis_h)),
                &self.scratch_rgba,
                (vis_w * 4) as usize,
            )
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;
        self.canvas.copy(
            &self.scratch_texture,
            Some(Rect::new(0, 0, vis_w, vis_h)),
            Rect::new(vis_left, vis_top, vis_w, vis_h),
        )?;
        Ok(())
    }

    // RGBA primitives -------------------------------------------------------

    /// Draw a filled rectangle using an explicit RGBA colour.
    /// Clips to (WIDTH, HEIGHT). Negative w/h are treated as zero.
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
        self.canvas.fill_rect(Rect::new(x, y, w as u32, h as u32))?;
        Ok(())
    }

    /// Draw a 1-pixel-wide outlined rectangle using an explicit RGBA colour.
    pub fn draw_box(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) -> Result<(), String> {
        self.draw_fill_rect(x, y, w, 1, color)?;
        self.draw_fill_rect(x, y + h - 1, w, 1, color)?;
        self.draw_fill_rect(x, y, 1, h, color)?;
        self.draw_fill_rect(x + w - 1, y, 1, h, color)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
struct TextureData {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    color_mod: (u8, u8, u8),
}

#[cfg(target_arch = "wasm32")]
pub struct Renderer {
    _canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    framebuffer: Vec<u8>,
    scratch_rgba: Vec<u8>,
    textures: HashMap<TextureId, TextureData>,
    next_texture_id: u32,
}

#[cfg(target_arch = "wasm32")]
impl Renderer {
    pub fn new_canvas(canvas_id: &str) -> Result<Self, String> {
        let window = web_sys::window().ok_or_else(|| "window not available".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "document not available".to_string())?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| format!("canvas #{canvas_id} not found"))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| format!("#{canvas_id} is not a canvas"))?;
        canvas.set_width(WIDTH);
        canvas.set_height(HEIGHT);
        let ctx = canvas
            .get_context("2d")
            .map_err(|e| format!("failed to get canvas context: {e:?}"))?
            .ok_or_else(|| "2d context not available".to_string())?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "context is not CanvasRenderingContext2d".to_string())?;
        Ok(Self {
            _canvas: canvas,
            ctx,
            framebuffer: vec![0; (WIDTH * HEIGHT * 4) as usize],
            scratch_rgba: Vec::new(),
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
        let id = TextureId(self.next_texture_id);
        self.next_texture_id += 1;
        self.textures.insert(
            id,
            TextureData {
                pixels: pixels.to_vec(),
                width,
                height,
                color_mod: (255, 255, 255),
            },
        );
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
        let src = src.unwrap_or_else(|| Rect::new(0, 0, texture.width, texture.height));
        let dst = dst.unwrap_or_else(|| Rect::new(0, 0, texture.width, texture.height));
        blit_texture(
            &mut self.framebuffer,
            &texture.pixels,
            texture.width,
            texture.height,
            texture.color_mod,
            src,
            dst,
        );
        Ok(())
    }

    pub fn wait_frame(&mut self) {}

    pub fn begin_frame(&mut self) {
        self.framebuffer.fill(0);
    }

    pub fn end_frame(&mut self) {
        let data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&self.framebuffer), WIDTH, HEIGHT)
                .expect("valid framebuffer dimensions");
        let _ = self.ctx.put_image_data(&data, 0.0, 0.0);
    }

    pub fn create_pattern_texture(
        &mut self,
        pattern_pixels: &[u8],
        tile_w: u32,
        tile_h: u32,
    ) -> Result<TextureId, String> {
        let mut rgba = Vec::with_capacity((tile_w * tile_h * 4) as usize);
        for &pixel in pattern_pixels {
            if pixel != 0 {
                rgba.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
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
        let brighten = |c: u8| (u32::from(c) * 130 / 100).min(255) as u8;
        let Some(texture) = self.textures.get_mut(&pattern_id) else {
            return Err("Pattern texture not found".into());
        };
        let old_mod = texture.color_mod;
        texture.color_mod = (brighten(color.r), brighten(color.g), brighten(color.b));

        let tw = tile_w as i32;
        let th = tile_h as i32;
        let end_x = (rect.x() + rect.width() as i32).min(WIDTH as i32);
        let end_y = (rect.y() + rect.height() as i32).min(HEIGHT as i32);
        let mut ty = rect.y();
        while ty < end_y {
            let src_y = ty.rem_euclid(th);
            let vis_h = (th - src_y).min(end_y - ty) as u32;
            let mut tx = rect.x();
            while tx < end_x {
                let src_x = tx.rem_euclid(tw);
                let vis_w = (tw - src_x).min(end_x - tx) as u32;
                self.draw_texture(
                    pattern_id,
                    Some(Rect::new(src_x, src_y, vis_w, vis_h)),
                    Some(Rect::new(tx.max(0), ty.max(0), vis_w, vis_h)),
                )?;
                tx += vis_w as i32;
            }
            ty += vis_h as i32;
        }
        if let Some(texture) = self.textures.get_mut(&pattern_id) {
            texture.color_mod = old_mod;
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
        blend_region(
            &mut self.framebuffer,
            &self.scratch_rgba,
            vis_w,
            vis_h,
            vis_left,
            vis_top,
            (255, 255, 255),
        );
        Ok(())
    }

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
        for yy in y..(y + h) {
            for xx in x..(x + w) {
                blend_pixel(
                    &mut self.framebuffer,
                    xx,
                    yy,
                    color.r,
                    color.g,
                    color.b,
                    color.a,
                );
            }
        }
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
///
/// The source is RGBA (4 bytes per pixel).
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

#[cfg(target_arch = "wasm32")]
fn blit_texture(
    dst_pixels: &mut [u8],
    src_pixels: &[u8],
    src_w: u32,
    src_h: u32,
    color_mod: (u8, u8, u8),
    src: Rect,
    dst: Rect,
) {
    let copy_w = src.width().min(dst.width());
    let copy_h = src.height().min(dst.height());
    if copy_w == 0 || copy_h == 0 {
        return;
    }
    for y in 0..copy_h {
        let sy = src.y() + y as i32;
        let dy = dst.y() + y as i32;
        if sy < 0 || sy >= src_h as i32 || dy < 0 || dy >= HEIGHT as i32 {
            continue;
        }
        for x in 0..copy_w {
            let sx = src.x() + x as i32;
            let dx = dst.x() + x as i32;
            if sx < 0 || sx >= src_w as i32 || dx < 0 || dx >= WIDTH as i32 {
                continue;
            }
            let src_idx = (sy as usize * src_w as usize + sx as usize) * 4;
            if src_idx + 3 >= src_pixels.len() {
                continue;
            }
            let r = (u32::from(src_pixels[src_idx]) * u32::from(color_mod.0) / 255) as u8;
            let g = (u32::from(src_pixels[src_idx + 1]) * u32::from(color_mod.1) / 255) as u8;
            let b = (u32::from(src_pixels[src_idx + 2]) * u32::from(color_mod.2) / 255) as u8;
            let a = src_pixels[src_idx + 3];
            blend_pixel(dst_pixels, dx, dy, r, g, b, a);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn blend_region(
    dst_pixels: &mut [u8],
    src_pixels: &[u8],
    src_w: u32,
    src_h: u32,
    dst_x: i32,
    dst_y: i32,
    color_mod: (u8, u8, u8),
) {
    blit_texture(
        dst_pixels,
        src_pixels,
        src_w,
        src_h,
        color_mod,
        Rect::new(0, 0, src_w, src_h),
        Rect::new(dst_x, dst_y, src_w, src_h),
    );
}

#[cfg(target_arch = "wasm32")]
fn blend_pixel(dst_pixels: &mut [u8], x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if a == 0 || x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return;
    }
    let idx = (y as usize * WIDTH as usize + x as usize) * 4;
    if idx + 3 >= dst_pixels.len() {
        return;
    }
    if a == 255 {
        dst_pixels[idx] = r;
        dst_pixels[idx + 1] = g;
        dst_pixels[idx + 2] = b;
        dst_pixels[idx + 3] = 255;
        return;
    }
    let inv_a = 255 - u32::from(a);
    let a = u32::from(a);
    dst_pixels[idx] = ((u32::from(r) * a + u32::from(dst_pixels[idx]) * inv_a) / 255) as u8;
    dst_pixels[idx + 1] = ((u32::from(g) * a + u32::from(dst_pixels[idx + 1]) * inv_a) / 255) as u8;
    dst_pixels[idx + 2] = ((u32::from(b) * a + u32::from(dst_pixels[idx + 2]) * inv_a) / 255) as u8;
    dst_pixels[idx + 3] = 255;
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
        let screen_w = WIDTH as i32;
        let screen_h = HEIGHT as i32;
        let rect =
            rgba_region_to_rgba(&src, 4, 4, 0, 0, screen_w - 2, screen_h - 2, 4, 4, &mut out);
        assert_eq!(rect, Some((screen_w - 2, screen_h - 2, 2, 2)));
        assert_eq!(out.len(), 2 * 2 * 4);
    }
}

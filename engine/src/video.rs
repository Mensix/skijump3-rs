use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::atlas::AtlasRegion;
use crate::consts::{FILL_BRIGHTEN, HEIGHT, TARGET_FPS, WIDTH};
use crate::palette::Palette;

mod dither;
pub(crate) mod indexed;
use dither::dither_rect_rgba;
pub use indexed::indexed_pixels_to_rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

pub struct Renderer {
    canvas: sdl2::render::WindowCanvas,
    overlay_rgba: Vec<u8>,
    scratch_rgba: Vec<u8>,
    palette: Palette,
    palette_revision: u64,
    last_tick: Instant,
    frame_texture: Texture,
    scratch_texture: Texture,
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
            overlay_rgba: vec![0u8; (WIDTH * HEIGHT * 4) as usize],
            scratch_rgba: Vec::new(),
            palette: Palette::new(),
            palette_revision: 0,
            last_tick: Instant::now(),
            frame_texture,
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

    pub fn set_palette(&mut self, palette: Palette) {
        if palette != self.palette {
            self.palette = palette;
            self.palette_revision += 1;
        }
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn palette_revision(&self) -> u64 {
        self.palette_revision
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
    /// Tiling always uses the original `TILE_W`/`TILE_H` constants.
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

    /// Convert indexed pixels to RGBA and draw via the reusable scratch
    /// texture.  Index 0 becomes fully transparent; other indices are
    /// opaque via the current palette.
    pub fn draw_indexed_overlay_pixels(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<(), String> {
        self.scratch_rgba.clear();
        indexed_pixels_to_rgba(pixels, &self.palette, &mut self.scratch_rgba);
        self.scratch_texture
            .update(
                Some(Rect::new(0, 0, width, height)),
                &self.scratch_rgba,
                (width * 4) as usize,
            )
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;
        self.canvas.copy(
            &self.scratch_texture,
            Some(Rect::new(0, 0, width, height)),
            Rect::new(x, y, width, height),
        )?;
        Ok(())
    }

    /// Convert a region of indexed pixels directly to RGBA and draw via the
    /// reusable scratch texture.  Clips to screen bounds.  Source indices
    /// outside the source rect or past the source buffer produce transparent
    /// pixels.  Index `0` is also transparent.
    pub fn draw_indexed_region_pixels(
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
        let Some((vis_left, vis_top, vis_w, vis_h)) = indexed::indexed_region_to_rgba(
            pixels,
            src_w,
            src_h,
            src_x,
            src_y,
            dst_x,
            dst_y,
            w,
            h,
            &self.palette,
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

    /// Draw a region from an RGBA atlas texture at (`x`, `y`) with center
    /// offset applied.  Clipping is handled by SDL2.
    pub fn draw_atlas_region(
        &mut self,
        texture_id: TextureId,
        region: &AtlasRegion,
        x: i32,
        y: i32,
    ) -> Result<(), String> {
        let dst_x = x - region.center_x as i32;
        let dst_y = y - region.center_y as i32;
        let src = Rect::new(
            region.x as i32,
            region.y as i32,
            region.width,
            region.height,
        );
        let dst = Rect::new(dst_x, dst_y, region.width, region.height);
        self.draw_texture(texture_id, Some(src), Some(dst))
    }
}

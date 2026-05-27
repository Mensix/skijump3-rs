use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::consts::{HEIGHT, TARGET_FPS, WIDTH};
use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

pub struct Renderer {
    canvas: sdl2::render::WindowCanvas,
    indexed_pixels: Vec<u8>,
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
            indexed_pixels: vec![0u8; (WIDTH * HEIGHT) as usize],
            overlay_rgba: vec![0u8; (WIDTH * HEIGHT * 4) as usize],
            palette: Palette::new(),
            last_tick: Instant::now(),
            frame_texture,
            textures: HashMap::new(),
            next_texture_id: 1,
        })
    }

    pub fn create_indexed_texture(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
        palette: &Palette,
    ) -> Result<TextureId, String> {
        let mut rgba = Vec::with_capacity(pixels.len() * 4);
        indexed_to_rgba(pixels, palette, &mut rgba);

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

        let id = TextureId(self.next_texture_id);
        self.next_texture_id += 1;
        self.textures.insert(id, texture);
        Ok(id)
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

    pub fn blit(&mut self, pixels: &[u8]) {
        assert_eq!(pixels.len(), self.indexed_pixels.len());
        self.indexed_pixels.copy_from_slice(pixels);
    }

    // Legacy full-frame upload: all pixels opaque (alpha = 255).
    pub fn present_legacy(&mut self) -> Result<(), String> {
        self.indexed_to_opaque_rgba();
        self.frame_texture
            .update(None, &self.overlay_rgba, (WIDTH * 4) as usize)
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;

        self.canvas.clear();
        self.canvas.copy(&self.frame_texture, None, None)?;
        self.canvas.present();
        Ok(())
    }

    // GPU frame layering API ------------------------------------------------

    /// Start a new GPU frame. Clears the canvas.
    /// Caller draws background textures, then `draw_legacy_framebuffer_overlay`.
    pub fn begin_frame(&mut self) {
        self.canvas.clear();
    }

    /// Finish the frame and present to screen.
    pub fn end_frame(&mut self) {
        self.canvas.present();
    }

    /// Convert the legacy indexed framebuffer to an RGBA overlay and draw it.
    /// Index 0 becomes transparent (alpha = 0); all other indices are opaque.
    pub fn draw_legacy_framebuffer_overlay(&mut self, pixels: &[u8]) -> Result<(), String> {
        self.indexed_to_overlay_rgba(pixels);
        self.frame_texture
            .update(None, &self.overlay_rgba, (WIDTH * 4) as usize)
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;
        self.canvas.copy(&self.frame_texture, None, None)?;
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
        self.canvas
            .fill_rect(Rect::new(x, y, w as u32, h as u32))?;
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

    // Internal helpers -------------------------------------------------------

    fn indexed_to_opaque_rgba(&mut self) {
        self.overlay_rgba.clear();
        for &idx in &self.indexed_pixels {
            let [r6, g6, b6] = self.palette.color(idx as usize);
            self.overlay_rgba.push((r6 as u32 * 255 / 63) as u8);
            self.overlay_rgba.push((g6 as u32 * 255 / 63) as u8);
            self.overlay_rgba.push((b6 as u32 * 255 / 63) as u8);
            self.overlay_rgba.push(255);
        }
    }

    fn indexed_to_overlay_rgba(&mut self, pixels: &[u8]) {
        self.overlay_rgba.clear();
        self.overlay_rgba.reserve(pixels.len() * 4);
        for &idx in pixels {
            if idx == 0 {
                self.overlay_rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let [r6, g6, b6] = self.palette.color(idx as usize);
                self.overlay_rgba.push((r6 as u32 * 255 / 63) as u8);
                self.overlay_rgba.push((g6 as u32 * 255 / 63) as u8);
                self.overlay_rgba.push((b6 as u32 * 255 / 63) as u8);
                self.overlay_rgba.push(255);
            }
        }
    }
}

fn indexed_to_rgba(pixels: &[u8], palette: &Palette, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(pixels.len() * 4);
    for &idx in pixels {
        let [r6, g6, b6] = palette.color(idx as usize);
        out.push((r6 as u32 * 255 / 63) as u8);
        out.push((g6 as u32 * 255 / 63) as u8);
        out.push((b6 as u32 * 255 / 63) as u8);
        out.push(255);
    }
}

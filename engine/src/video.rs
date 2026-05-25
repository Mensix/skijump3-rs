use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::consts::{HEIGHT, TARGET_FPS, WIDTH};
use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

pub struct Renderer {
    canvas: sdl2::render::WindowCanvas,
    indexed_pixels: Vec<u8>,
    rgb_pixels: Vec<u8>,
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
        let frame_texture = tc
            .create_texture(
                PixelFormatEnum::RGB24,
                sdl2::render::TextureAccess::Streaming,
                WIDTH,
                HEIGHT,
            )
            .map_err(|e| e.to_string())?;

        Ok(Self {
            canvas,
            indexed_pixels: vec![0u8; (WIDTH * HEIGHT) as usize],
            rgb_pixels: vec![0u8; (WIDTH * HEIGHT * 3) as usize],
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
                PixelFormatEnum::RGBA8888,
                sdl2::render::TextureAccess::Static,
                width,
                height,
            )
            .map_err(|e| e.to_string())?;
        texture
            .update(None, &rgba, (width * 4) as usize)
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

    pub fn present(&mut self) -> Result<(), String> {
        for i in 0..self.indexed_pixels.len().min(self.rgb_pixels.len() / 3) {
            let idx = self.indexed_pixels[i] as usize;
            let [r, g, b] = self.palette.color(idx);
            let pos = i * 3;
            self.rgb_pixels[pos] = ((r as u32) * 255 / 63) as u8;
            self.rgb_pixels[pos + 1] = ((g as u32) * 255 / 63) as u8;
            self.rgb_pixels[pos + 2] = ((b as u32) * 255 / 63) as u8;
        }

        self.frame_texture
            .update(None, &self.rgb_pixels, (WIDTH * 3) as usize)
            .map_err(|e: sdl2::render::UpdateTextureError| e.to_string())?;

        self.canvas.clear();
        self.canvas.copy(&self.frame_texture, None, None)?;
        self.canvas.present();
        Ok(())
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

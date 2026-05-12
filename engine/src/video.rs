use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;
use std::time::{Duration, Instant};

use crate::consts::{HEIGHT, TARGET_FPS, WIDTH};
use crate::palette::Palette;

pub struct Renderer {
    canvas: sdl2::render::WindowCanvas,
    indexed_pixels: Vec<u8>,
    rgb_pixels: Vec<u8>,
    palette: Palette,
    last_tick: Instant,
}

impl Renderer {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let video = sdl.video().map_err(|e| e.to_string())?;

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

        canvas.set_logical_size(WIDTH, HEIGHT).map_err(|e| e.to_string())?;

        Ok(Self {
            canvas,
            indexed_pixels: vec![0u8; (WIDTH * HEIGHT) as usize],
            rgb_pixels: vec![0u8; (WIDTH * HEIGHT * 3) as usize],
            palette: Palette::new(),
            last_tick: Instant::now(),
        })
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
            let clamped_idx = idx.min(255);
            let [r, g, b] = self.palette.color(clamped_idx);
            let pos = i * 3;
            self.rgb_pixels[pos] = ((r as u32) * 255 / 63) as u8;
            self.rgb_pixels[pos + 1] = ((g as u32) * 255 / 63) as u8;
            self.rgb_pixels[pos + 2] = ((b as u32) * 255 / 63) as u8;
        }

        let tc = self.canvas.texture_creator();
        let mut texture = tc
            .create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, WIDTH, HEIGHT)
            .map_err(|e| e.to_string())?;

        texture
            .update(None, &self.rgb_pixels, (WIDTH * 3) as usize)
            .map_err(|e| e.to_string())?;

        self.canvas.clear();
        self.canvas.copy(&texture, None, None).map_err(|e| e.to_string())?;
        self.canvas.present();
        Ok(())
    }
}

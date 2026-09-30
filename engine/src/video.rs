use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{BlendMode, FPoint, ScaleMode, Texture, WindowCanvas};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::color::Rgba;
use crate::consts::TARGET_FPS;
use crate::consts::{HEIGHT, SCALE, WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

#[cfg(test)]
impl TextureId {
    pub(crate) const fn test_id(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub const fn x(&self) -> i32 {
        self.x
    }

    pub const fn y(&self) -> i32 {
        self.y
    }

    pub const fn width(&self) -> u32 {
        self.w
    }

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
    last_tick: Instant,
    textures: HashMap<TextureId, Texture>,
    point_scratch: Vec<FPoint>,
    next_texture_id: u32,
    vsync_enabled: bool,
}

impl Renderer {
    pub fn new(sdl: &sdl3::Sdl, software_rendering: bool) -> Result<Self, String> {
        Self::new_with_vsync(sdl, software_rendering, true)
    }

    pub fn new_with_vsync(
        sdl: &sdl3::Sdl,
        software_rendering: bool,
        request_vsync: bool,
    ) -> Result<Self, String> {
        let video = sdl
            .video()
            .map_err(|error| format!("SDL video init failed: {error}"))?;

        let window = video
            .window("Ski Jump International v3", WIDTH * SCALE, HEIGHT * SCALE)
            .resizable()
            .maximized()
            .position_centered()
            .build()
            .map_err(|error| format!("SDL window creation failed: {error}"))?;

        let mut canvas = if software_rendering {
            sdl3::render::create_renderer(window, Some(c"software"))
                .map_err(|error| format!("SDL software renderer creation failed: {error}"))?
        } else {
            window.into_canvas()
        };

        let vsync_configured = unsafe {
            sdl3::sys::render::SDL_SetRenderVSync(canvas.raw(), i32::from(request_vsync))
        };
        let vsync_enabled = request_vsync && vsync_configured;

        canvas
            .set_logical_size(
                WIDTH,
                HEIGHT,
                sdl3::sys::render::SDL_RendererLogicalPresentation::INTEGER_SCALE,
            )
            .map_err(|error| format!("SDL logical size setup failed: {error}"))?;

        Ok(Self {
            canvas,
            last_tick: Instant::now(),
            textures: HashMap::new(),
            point_scratch: Vec::new(),
            next_texture_id: 1,
            vsync_enabled,
        })
    }

    pub fn create_rgba_texture(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Option<TextureId> {
        if width == 0 || height == 0 {
            return None;
        }
        let pitch = usize::try_from(width.checked_mul(4)?).ok()?;
        let expected_len = pitch.checked_mul(usize::try_from(height).ok()?)?;
        if pixels.len() != expected_len {
            return None;
        }
        let tc = self.canvas.texture_creator();
        let Ok(mut texture) = tc.create_texture(
            PixelFormat::ABGR8888,
            sdl3::render::TextureAccess::Static,
            width,
            height,
        ) else {
            return None;
        };
        texture.set_blend_mode(BlendMode::Blend);
        texture.set_scale_mode(ScaleMode::Nearest);
        if texture.update(None, pixels, pitch).is_err() {
            return None;
        }

        let id = TextureId(self.next_texture_id);
        self.next_texture_id += 1;
        self.textures.insert(id, texture);
        Some(id)
    }

    pub fn remove_texture(&mut self, id: TextureId) -> bool {
        self.textures.remove(&id).is_some()
    }

    pub fn draw_texture(&mut self, id: TextureId, src: Option<Rect>, dst: Option<Rect>) {
        self.draw_texture_modulated_with_offset(id, src, dst, None, (0.0, 0.0));
    }

    pub fn set_icon(&mut self, pixels: &mut [u8], width: u32, height: u32) {
        let pitch = width.checked_mul(4);
        let expected = pitch.and_then(|pitch| {
            usize::try_from(pitch)
                .ok()?
                .checked_mul(usize::try_from(height).ok()?)
        });
        if width == 0 || height == 0 || expected.is_none_or(|len| pixels.len() != len) {
            return;
        }
        let Ok(surface) = sdl3::surface::Surface::from_data(
            pixels,
            width,
            height,
            pitch.unwrap_or(0),
            PixelFormat::RGBA32,
        ) else {
            return;
        };
        let _ = self.canvas.window_mut().set_icon(&surface);
    }

    pub fn draw_texture_modulated_with_offset(
        &mut self,
        id: TextureId,
        src: Option<Rect>,
        dst: Option<Rect>,
        modulation: Option<Rgba>,
        destination_offset: (f32, f32),
    ) {
        let old_modulation = if let Some(color) = modulation {
            let Some(texture) = self.textures.get_mut(&id) else {
                return;
            };
            let old = texture.color_mod();
            texture.set_color_mod(color.r, color.g, color.b);
            Some(old)
        } else {
            None
        };
        if let Some(texture) = self.textures.get(&id) {
            let _ = self.canvas.copy(
                texture,
                src.map(sdl3::render::FRect::from),
                dst.map(|rect| {
                    sdl3::render::FRect::new(
                        rect.x() as f32 + destination_offset.0,
                        rect.y() as f32 + destination_offset.1,
                        rect.width() as f32,
                        rect.height() as f32,
                    )
                }),
            );
        }
        if let Some((r, g, b)) = old_modulation {
            if let Some(texture) = self.textures.get_mut(&id) {
                texture.set_color_mod(r, g, b);
            }
        }
    }

    pub fn wait_frame(&mut self) {
        if self.vsync_enabled {
            self.last_tick = Instant::now();
            return;
        }

        let frame_time = Duration::from_secs_f64(1.0 / f64::from(TARGET_FPS));
        let elapsed = self.last_tick.elapsed();
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
        self.last_tick = Instant::now();
    }

    pub fn begin_frame(&mut self) {
        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        self.canvas.clear();
    }

    pub fn end_frame(&mut self) {
        let _ = self.canvas.present();
    }

    pub fn create_pattern_texture(
        &mut self,
        pattern_pixels: &[u8],
        tile_w: u32,
        tile_h: u32,
    ) -> Option<TextureId> {
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
    ) {
        let (tint_r, tint_g, tint_b) = pattern_tint(color);

        let (old_r, old_g, old_b) = {
            let Some(t) = self.textures.get_mut(&pattern_id) else {
                return;
            };
            let old = t.color_mod();
            t.set_color_mod(tint_r, tint_g, tint_b);
            old
        };

        let Some(texture) = self.textures.get(&pattern_id) else {
            return;
        };

        for_tiled_segments(rect, tile_w, tile_h, |src, dst| {
            let _ = self.canvas.copy(
                texture,
                Some(sdl3::render::FRect::from(src)),
                Some(sdl3::render::FRect::from(dst)),
            );
        });

        if let Some(t) = self.textures.get_mut(&pattern_id) {
            t.set_color_mod(old_r, old_g, old_b);
        }
    }

    pub fn draw_fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        let right = x.saturating_add(w);
        let bottom = y.saturating_add(h);
        let left = x.max(0);
        let top = y.max(0);
        let right = right.min(WIDTH as i32);
        let bottom = bottom.min(HEIGHT as i32);
        let w = (right - left).max(0);
        let h = (bottom - top).max(0);
        if w <= 0 || h <= 0 {
            return;
        }
        self.canvas.set_draw_color(color.to_sdl());
        let _ = self.canvas.fill_rect(Some(sdl3::render::FRect::new(
            left as f32,
            top as f32,
            w as f32,
            h as f32,
        )));
    }

    pub fn draw_points(&mut self, points: &[(i32, i32)], color: Rgba) {
        if points.is_empty() {
            return;
        }
        self.canvas.set_draw_color(color.to_sdl());
        self.point_scratch.clear();
        self.point_scratch
            .extend(points.iter().map(|&(x, y)| FPoint::new(x as f32, y as f32)));
        let _ = self.canvas.draw_points(self.point_scratch.as_slice());
    }

    pub fn draw_box(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        self.draw_fill_rect(x, y, w, 1, color);
        self.draw_fill_rect(x, y + h - 1, w, 1, color);
        self.draw_fill_rect(x, y, 1, h, color);
        self.draw_fill_rect(x + w - 1, y, 1, h, color);
    }
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

fn for_tiled_segments(rect: Rect, tile_w: u32, tile_h: u32, mut f: impl FnMut(Rect, Rect)) {
    let tw = tile_w as i32;
    let th = tile_h as i32;
    if tw <= 0 || th <= 0 {
        return;
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
            );
            tx += vis_w as i32;
        }
        ty += vis_h as i32;
    }
}

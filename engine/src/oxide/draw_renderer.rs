use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::color::Rgba;
use crate::consts::PATTERN_SPRITE;
use crate::oxide::draw::{DrawCommand, SpriteDraw, TextAlign};
use crate::oxide::Font;
use crate::palette::Palette;
use crate::sprite::{SpriteData, SpriteMaterial};
use crate::video::{Renderer, TextureId};

const TEXT_SHADOW: Rgba = Rgba::rgb(0, 0, 0);
const MAX_CACHE_SIZE: usize = 256;

struct DitherRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Rgba,
    is_box: bool,
}

impl DitherRect {
    const fn new(x: i32, y: i32, w: i32, h: i32, color: Rgba, is_box: bool) -> Self {
        Self {
            x,
            y,
            w,
            h,
            color,
            is_box,
        }
    }
}

struct DitherFillCollector {
    pending: Vec<DitherRect>,
}

impl DitherFillCollector {
    fn new(mut pending: Vec<DitherRect>) -> Self {
        pending.clear();
        Self { pending }
    }

    fn track_fillbox(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        self.track_rect(x, y, w, h, color, false);
    }

    fn track_box(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        self.track_rect(x, y, w, h, color, true);
    }

    fn track_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba, is_box: bool) {
        self.pending
            .push(DitherRect::new(x, y, w, h, color, is_box));
    }

    fn apply_fill_area(
        &mut self,
        renderer: &mut Renderer,
        sprites: &[SpriteData],
        thing: u8,
        colors: &[Rgba],
    ) -> Result<(), String> {
        if let Some(pattern) = sprites.get(PATTERN_SPRITE) {
            for rect in &self.pending {
                if !colors.contains(&rect.color) {
                    continue;
                }
                renderer.dither_overlay_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    rect.color,
                    rect.is_box,
                    thing,
                    &pattern.pixels,
                )?;
            }
        }
        if !self.pending.is_empty() {
            renderer.flush_dither_overlay()?;
            self.pending.clear();
        }
        Ok(())
    }

    fn into_pending(self) -> Vec<DitherRect> {
        self.pending
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct SpriteTextureCacheKey {
    sprite_idx: u16,
    material: SpriteMaterial,
}

struct SpriteTextureCacheEntry {
    texture_id: TextureId,
    center_x: i8,
    center_y: i8,
    width: u16,
    height: u16,
    last_frame: u64,
}

#[derive(Default)]
struct SpriteTextureCache {
    entries: HashMap<SpriteTextureCacheKey, SpriteTextureCacheEntry>,
    rgba_scratch: Vec<u8>,
}

impl SpriteTextureCache {
    fn draw(
        &mut self,
        renderer: &mut Renderer,
        palette: &Palette,
        sprite_idx: u16,
        x: i32,
        y: i32,
        material: &SpriteMaterial,
        sprite: &SpriteData,
        frame: u64,
    ) -> Result<(), String> {
        if self.entries.len() >= MAX_CACHE_SIZE {
            self.evict_stale(frame);
        }
        let key = SpriteTextureCacheKey {
            sprite_idx,
            material: material.clone(),
        };
        match self.entries.entry(key) {
            Entry::Occupied(o) => {
                let entry = o.into_mut();
                entry.last_frame = frame;
                draw_sprite_texture(renderer, entry, x, y)
            }
            Entry::Vacant(v) => {
                sprite.render_material_rgba(palette, material, &mut self.rgba_scratch);
                if self.rgba_scratch.iter().any(|&b| b != 0) {
                    let tex_id = renderer.create_rgba_texture(
                        &self.rgba_scratch,
                        u32::from(sprite.width),
                        u32::from(sprite.height),
                    )?;
                    let entry = SpriteTextureCacheEntry {
                        texture_id: tex_id,
                        center_x: sprite.center_x,
                        center_y: sprite.center_y,
                        width: sprite.width,
                        height: sprite.height,
                        last_frame: frame,
                    };
                    draw_sprite_texture(renderer, &entry, x, y)?;
                    v.insert(entry);
                }
                Ok(())
            }
        }
    }

    fn evict_stale(&mut self, frame: u64) {
        self.entries.retain(|_, e| e.last_frame == frame);
        if self.entries.len() >= MAX_CACHE_SIZE {
            let mut entries: Vec<_> = self.entries.drain().collect();
            entries.sort_by_key(|(_, e)| std::cmp::Reverse(e.last_frame));
            entries.truncate(MAX_CACHE_SIZE / 2);
            self.entries = entries.into_iter().collect();
        }
    }
}

fn draw_sprite_texture(
    renderer: &mut Renderer,
    entry: &SpriteTextureCacheEntry,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let dst_x = x - i32::from(entry.center_x);
    let dst_y = y - i32::from(entry.center_y);
    renderer.draw_texture(
        entry.texture_id,
        None,
        Some(sdl2::rect::Rect::new(
            dst_x,
            dst_y,
            u32::from(entry.width),
            u32::from(entry.height),
        )),
    )
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextCacheKey {
    text: String,
    x: i32,
    y: i32,
    color: Rgba,
}

struct TextCacheEntry {
    texture_id: TextureId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    last_frame: u64,
}

#[derive(Default)]
struct TextCache {
    entries: HashMap<TextCacheKey, TextCacheEntry>,
    rgba_scratch: Vec<u8>,
}

impl TextCache {
    fn evict_stale(&mut self, frame: u64) {
        self.entries.retain(|_, e| e.last_frame == frame);
        if self.entries.len() >= MAX_CACHE_SIZE {
            let mut entries: Vec<_> = self.entries.drain().collect();
            entries.sort_by_key(|(_, e)| std::cmp::Reverse(e.last_frame));
            entries.truncate(MAX_CACHE_SIZE / 2);
            self.entries = entries.into_iter().collect();
        }
    }

    fn draw(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        text: &str,
        x: i32,
        y: i32,
        color: Rgba,
        align: TextAlign,
        frame: u64,
    ) -> Result<(), String> {
        if self.entries.len() >= MAX_CACHE_SIZE {
            self.evict_stale(frame);
        }
        let text_w = font.string_width(text) as i32;
        let fx = match align {
            TextAlign::Center => x - text_w / 2,
            TextAlign::Right => x - text_w,
            TextAlign::Left => x,
        };

        let key = TextCacheKey {
            text: text.to_owned(),
            x: fx,
            y,
            color,
        };
        match self.entries.entry(key) {
            Entry::Occupied(o) => {
                let entry = o.into_mut();
                entry.last_frame = frame;
                draw_text_texture(renderer, entry)
            }
            Entry::Vacant(v) => {
                if let Some(bitmap) =
                    font.render_string_rgba(text, fx, y, color, TEXT_SHADOW, &mut self.rgba_scratch)
                {
                    if bitmap.pixels.iter().any(|&b| b != 0) {
                        let tex_id = renderer.create_rgba_texture(
                            &bitmap.pixels,
                            bitmap.width,
                            bitmap.height,
                        )?;
                        let entry = TextCacheEntry {
                            texture_id: tex_id,
                            x: bitmap.x,
                            y: bitmap.y,
                            width: bitmap.width,
                            height: bitmap.height,
                            last_frame: frame,
                        };
                        draw_text_texture(renderer, &entry)?;
                        v.insert(entry);
                    }
                }
                Ok(())
            }
        }
    }
}

fn draw_text_texture(renderer: &mut Renderer, entry: &TextCacheEntry) -> Result<(), String> {
    renderer.draw_texture(
        entry.texture_id,
        None,
        Some(sdl2::rect::Rect::new(
            entry.x,
            entry.y,
            entry.width,
            entry.height,
        )),
    )
}

pub struct DrawCommandRenderer {
    pending_dither_rects: Vec<DitherRect>,
    sprite_texture_cache: SpriteTextureCache,
    text_cache: TextCache,
    frame_counter: u64,
}

impl Default for DrawCommandRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawCommandRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_dither_rects: Vec::new(),
            sprite_texture_cache: SpriteTextureCache::default(),
            text_cache: TextCache::default(),
            frame_counter: 0,
        }
    }

    pub fn render_frame(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        palette: &Palette,
        sprites: &[SpriteData],
        commands: &[DrawCommand],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        renderer.begin_frame();
        if let Some(bg) = background {
            renderer.draw_texture(bg, None, None)?;
        }

        let frame = self.frame_counter;
        self.frame_counter = self.frame_counter.wrapping_add(1);

        let pending_dither_rects = std::mem::take(&mut self.pending_dither_rects);
        let mut dither = DitherFillCollector::new(pending_dither_rects);

        for command in commands {
            self.render_command(
                renderer,
                font,
                palette,
                sprites,
                command,
                &mut dither,
                frame,
            )?;
        }

        self.pending_dither_rects = dither.into_pending();
        renderer.end_frame();
        Ok(())
    }

    fn render_command(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        palette: &Palette,
        sprites: &[SpriteData],
        command: &DrawCommand,
        dither: &mut DitherFillCollector,
        frame: u64,
    ) -> Result<(), String> {
        match command {
            DrawCommand::Fill(rect, color) => {
                renderer.draw_fill_rect(rect.x, rect.y, rect.w, rect.h, *color)?;
                dither.track_fillbox(rect.x, rect.y, rect.w, rect.h, *color);
            }
            DrawCommand::Stroke(rect, color) => {
                renderer.draw_box(rect.x, rect.y, rect.w, rect.h, *color)?;
                dither.track_box(rect.x, rect.y, rect.w, rect.h, *color);
            }
            DrawCommand::DitherFill { thing, colors } => {
                dither.apply_fill_area(renderer, sprites, *thing, colors)?;
            }
            DrawCommand::Text(run) => {
                self.text_cache.draw(
                    renderer,
                    font,
                    &run.text,
                    run.position.x,
                    run.position.y,
                    run.color,
                    run.align,
                    frame,
                )?;
            }
            DrawCommand::Sprite(SpriteDraw { idx, position }) => {
                if let Some(sprite_data) = sprites.get(*idx as usize) {
                    let material = SpriteMaterial::default();
                    self.sprite_texture_cache.draw(
                        renderer,
                        palette,
                        *idx,
                        position.x,
                        position.y,
                        &material,
                        sprite_data,
                        frame,
                    )?;
                }
            }
            DrawCommand::SpriteWithMaterial { sprite, material } => {
                if let Some(sprite_data) = sprites.get(sprite.idx as usize) {
                    self.sprite_texture_cache.draw(
                        renderer,
                        palette,
                        sprite.idx,
                        sprite.position.x,
                        sprite.position.y,
                        material,
                        sprite_data,
                        frame,
                    )?;
                }
            }
            DrawCommand::Image { pixels, w, h } => {
                renderer.draw_rgba_region_pixels(pixels, *w, *h, 0, 0, 0, 0, *w, *h)?;
            }
            DrawCommand::ImageRegion(region) => {
                renderer.draw_rgba_region_pixels(
                    &region.pixels,
                    region.src_w,
                    region.src_h,
                    region.src_x,
                    region.src_y,
                    region.dst_x,
                    region.dst_y,
                    region.w,
                    region.h,
                )?;
            }
        }
        Ok(())
    }
}

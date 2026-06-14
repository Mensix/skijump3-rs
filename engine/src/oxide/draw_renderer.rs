use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::color::Rgba;
use crate::consts::PATTERN_SPRITE;
use crate::oxide::draw::{DrawCommand, SpriteDraw, TextAlign};
use crate::oxide::Font;
use crate::sprite::{SpriteColorRecolor, SpriteData};
use crate::video::{Renderer, TextureId};

const TEXT_SHADOW: Rgba = Rgba::rgb(0, 0, 0);
const MAX_CACHE_SIZE: usize = 256;

const DITHER_FILL_COLORS: [Rgba; 5] = [
    Rgba::from_rgb6(18, 13, 34),
    Rgba::from_rgb6(34, 13, 18),
    Rgba::from_rgb6(20, 20, 20),
    Rgba::from_rgb6(0, 25, 0),
    Rgba::from_rgb6(28, 8, 24),
];

fn is_fill_area_dither_color(color: Rgba) -> bool {
    DITHER_FILL_COLORS.contains(&color)
}

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
    remaining_fill_areas: usize,
}

impl DitherFillCollector {
    fn new(mut pending: Vec<DitherRect>, fill_area_count: usize) -> Self {
        pending.clear();
        Self {
            pending,
            remaining_fill_areas: fill_area_count,
        }
    }

    const fn has_pending_fill_area(&self) -> bool {
        self.remaining_fill_areas > 0
    }

    fn track_fillbox(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        self.track_rect(x, y, w, h, color, false);
    }

    fn track_box(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        self.track_rect(x, y, w, h, color, true);
    }

    fn track_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba, is_box: bool) {
        if self.has_pending_fill_area() && is_fill_area_dither_color(color) {
            self.pending
                .push(DitherRect::new(x, y, w, h, color, is_box));
        }
    }

    fn apply_fill_area(
        &mut self,
        renderer: &mut Renderer,
        sprites: &[SpriteData],
        thing: u8,
    ) -> Result<(), String> {
        if let Some(pattern) = sprites.get(PATTERN_SPRITE) {
            for rect in &self.pending {
                renderer.dither_overlay_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    rect.color,
                    rect.is_box,
                    thing,
                    &pattern.data,
                )?;
            }
        }
        if !self.pending.is_empty() {
            renderer.flush_dither_overlay()?;
            self.pending.clear();
        }
        self.remaining_fill_areas = self.remaining_fill_areas.saturating_sub(1);
        Ok(())
    }

    fn into_pending(self) -> Vec<DitherRect> {
        self.pending
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct RemappedSpriteCacheKey {
    sprite_idx: u16,
    recolor: SpriteColorRecolor,
}

struct RemappedSpriteCacheEntry {
    texture_id: TextureId,
    center_x: i8,
    center_y: i8,
    width: u16,
    height: u16,
}

#[derive(Default)]
struct RemappedSpriteCache {
    entries: HashMap<RemappedSpriteCacheKey, RemappedSpriteCacheEntry>,
    rgba_scratch: Vec<u8>,
}

impl RemappedSpriteCache {
    fn draw(
        &mut self,
        renderer: &mut Renderer,
        sprite_idx: u16,
        x: i32,
        y: i32,
        recolor: &SpriteColorRecolor,
        sprite: &SpriteData,
    ) -> Result<(), String> {
        if self.entries.len() >= MAX_CACHE_SIZE {
            self.entries.clear();
        }
        let key = RemappedSpriteCacheKey {
            sprite_idx,
            recolor: recolor.clone(),
        };
        match self.entries.entry(key) {
            Entry::Occupied(o) => draw_remapped_texture(renderer, o.get(), x, y),
            Entry::Vacant(v) => {
                sprite.render_recolored_rgba(recolor, &mut self.rgba_scratch);
                if self.rgba_scratch.iter().any(|&b| b != 0) {
                    let tex_id = renderer.create_rgba_texture(
                        &self.rgba_scratch,
                        u32::from(sprite.width),
                        u32::from(sprite.height),
                    )?;
                    let entry = RemappedSpriteCacheEntry {
                        texture_id: tex_id,
                        center_x: sprite.center_x,
                        center_y: sprite.center_y,
                        width: sprite.width,
                        height: sprite.height,
                    };
                    draw_remapped_texture(renderer, &entry, x, y)?;
                    v.insert(entry);
                }
                Ok(())
            }
        }
    }
}

fn draw_remapped_texture(
    renderer: &mut Renderer,
    entry: &RemappedSpriteCacheEntry,
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
}

#[derive(Default)]
struct TextCache {
    entries: HashMap<TextCacheKey, TextCacheEntry>,
    rgba_scratch: Vec<u8>,
}

impl TextCache {
    fn draw(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        text: &str,
        x: i32,
        y: i32,
        color: Rgba,
        align: TextAlign,
    ) -> Result<(), String> {
        if self.entries.len() >= MAX_CACHE_SIZE {
            self.entries.clear();
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
            Entry::Occupied(o) => draw_text_texture(renderer, o.get()),
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

fn count_fill_area_commands(commands: &[DrawCommand]) -> usize {
    commands
        .iter()
        .filter(|cmd| matches!(cmd, DrawCommand::DitherFill(_)))
        .count()
}

pub struct DrawCommandRenderer {
    pending_dither_rects: Vec<DitherRect>,
    remapped_sprite_cache: RemappedSpriteCache,
    text_cache: TextCache,
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
            remapped_sprite_cache: RemappedSpriteCache::default(),
            text_cache: TextCache::default(),
        }
    }

    pub fn render_frame(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        commands: &[DrawCommand],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        renderer.begin_frame();
        if let Some(bg) = background {
            renderer.draw_texture(bg, None, None)?;
        }

        let fill_area_count = count_fill_area_commands(commands);
        let pending_dither_rects = std::mem::take(&mut self.pending_dither_rects);
        let mut dither = DitherFillCollector::new(pending_dither_rects, fill_area_count);

        for command in commands {
            self.render_command(renderer, font, sprites, command, &mut dither)?;
        }

        self.pending_dither_rects = dither.into_pending();
        renderer.end_frame();
        Ok(())
    }

    fn render_command(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        command: &DrawCommand,
        dither: &mut DitherFillCollector,
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
            DrawCommand::DitherFill(thing) => {
                dither.apply_fill_area(renderer, sprites, *thing)?;
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
                )?;
            }
            DrawCommand::Sprite(SpriteDraw { idx, position }) => {
                if let Some(sprite) = sprites.get(*idx as usize) {
                    if let Some(bitmap) = sprite.render_rgba_bitmap(position.x, position.y) {
                        renderer.draw_rgba_region_pixels(
                            &bitmap.pixels,
                            bitmap.width,
                            bitmap.height,
                            0,
                            0,
                            bitmap.x,
                            bitmap.y,
                            bitmap.width,
                            bitmap.height,
                        )?;
                    }
                }
            }
            DrawCommand::SpriteRemapped { sprite, recolor } => {
                if let Some(sprite_data) = sprites.get(sprite.idx as usize) {
                    self.remapped_sprite_cache.draw(
                        renderer,
                        sprite.idx,
                        sprite.position.x,
                        sprite.position.y,
                        recolor,
                        sprite_data,
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

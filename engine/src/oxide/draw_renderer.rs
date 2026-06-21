use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::color::Rgba;
use crate::consts::{TILE_H, TILE_W};
use crate::oxide::draw::{DrawCommand, SpriteDraw, TextAlign};
use crate::oxide::Font;
use crate::sprite::{BakedSpriteTexture, BakedSpriteTextures};
use crate::video::{Rect, Renderer, TextureId};

const TEXT_SHADOW: Rgba = Rgba::rgb(0, 0, 0);
const MAX_CACHE_SIZE: usize = 256;

fn draw_baked_sprite_texture(
    renderer: &mut Renderer,
    texture: &BakedSpriteTexture,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let dst_x = x - i32::from(texture.center_x);
    let dst_y = y - i32::from(texture.center_y);
    renderer.draw_texture(
        texture.texture_id,
        None,
        Some(Rect::new(
            dst_x,
            dst_y,
            u32::from(texture.width),
            u32::from(texture.height),
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
        Some(Rect::new(entry.x, entry.y, entry.width, entry.height)),
    )
}

pub struct DrawCommandRenderer {
    text_cache: TextCache,
    frame_counter: u64,
}

pub(crate) struct DrawRenderAssets<'a> {
    pub(crate) font: &'a Font,
    pub(crate) baked_sprites: &'a BakedSpriteTextures,
    pub(crate) pattern_texture: TextureId,
}

impl Default for DrawCommandRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawCommandRenderer {
    pub fn new() -> Self {
        Self {
            text_cache: TextCache::default(),
            frame_counter: 0,
        }
    }

    pub(crate) fn render_frame(
        &mut self,
        renderer: &mut Renderer,
        assets: DrawRenderAssets<'_>,
        commands: &[DrawCommand],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        renderer.begin_frame();
        if let Some(bg) = background {
            renderer.draw_texture(bg, None, None)?;
        }

        let frame = self.frame_counter;
        self.frame_counter = self.frame_counter.wrapping_add(1);

        for command in commands {
            self.render_command(renderer, &assets, command, frame)?;
        }

        renderer.end_frame();
        Ok(())
    }

    fn render_command(
        &mut self,
        renderer: &mut Renderer,
        assets: &DrawRenderAssets<'_>,
        command: &DrawCommand,
        frame: u64,
    ) -> Result<(), String> {
        match command {
            DrawCommand::Fill(rect, color) => {
                renderer.draw_fill_rect(rect.x, rect.y, rect.w, rect.h, *color)?;
            }
            DrawCommand::Stroke(rect, color) => {
                renderer.draw_box(rect.x, rect.y, rect.w, rect.h, *color)?;
            }
            DrawCommand::PatternFill(rect, color) => {
                renderer.draw_fill_rect(rect.x, rect.y, rect.w, rect.h, *color)?;
                renderer.draw_tiled_pattern(
                    Rect::new(rect.x, rect.y, rect.w as u32, rect.h as u32),
                    *color,
                    assets.pattern_texture,
                    TILE_W,
                    TILE_H,
                )?;
            }
            DrawCommand::PatternStroke(rect, color) => {
                renderer.draw_box(rect.x, rect.y, rect.w, rect.h, *color)?;
                renderer.draw_tiled_pattern(
                    Rect::new(rect.x, rect.y, rect.w as u32, rect.h as u32),
                    *color,
                    assets.pattern_texture,
                    TILE_W,
                    TILE_H,
                )?;
            }
            DrawCommand::Text(run) => {
                self.text_cache.draw(
                    renderer,
                    assets.font,
                    &run.text,
                    run.position.x,
                    run.position.y,
                    run.color,
                    run.align,
                    frame,
                )?;
            }
            DrawCommand::Sprite(SpriteDraw { idx, position }) => {
                if let Some(texture) = assets.baked_sprites.default_sprite(*idx) {
                    return draw_baked_sprite_texture(renderer, texture, position.x, position.y);
                }
            }
            DrawCommand::SpriteWithMaterial { sprite, material } => {
                if let Some(texture) = assets.baked_sprites.material_sprite(sprite.idx, material) {
                    return draw_baked_sprite_texture(
                        renderer,
                        texture,
                        sprite.position.x,
                        sprite.position.y,
                    );
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

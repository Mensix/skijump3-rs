use std::collections::HashMap;

use crate::bitmap::RgbaBitmap;
use crate::color::Rgba;
use crate::consts::{HEIGHT, TILE_H, TILE_W, WIDTH};
use crate::oxide::draw::{DrawCommand, Rect as DrawRect, SpriteDraw, TextAlign, TextRun};
use crate::oxide::Font;
use crate::sprite::{BakedSpriteTexture, BakedSpriteTextures};
use crate::video::{Rect, Renderer, TextureId};

const TEXT_SHADOW: Rgba = Rgba::rgb(0, 0, 0);
const MAX_CACHE_SIZE: usize = 256;
const MAX_CACHE_AGE: u64 = 300;
const MAX_STATIC_TEXTURES: usize = 128;
const MAX_STATIC_TEXTURE_AGE: u64 = 600;

struct StaticTextureEntry {
    texture_id: TextureId,
    last_used: u64,
}

fn draw_baked_sprite_texture(
    renderer: &mut Renderer,
    texture: &BakedSpriteTexture,
    x: i32,
    y: i32,
) {
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
    );
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TextCacheStyle {
    x: i32,
    y: i32,
    color: Rgba,
}

#[derive(Clone, Copy)]
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
    entries: HashMap<TextCacheStyle, HashMap<String, TextCacheEntry>>,
    len: usize,
    rgba_scratch: Vec<u8>,
}

impl TextCache {
    fn evict(&mut self, frame: u64, reserve: usize) -> Vec<TextureId> {
        let mut removed = Vec::new();
        self.entries.retain(|_, texts| {
            texts.retain(|_, entry| {
                if frame.saturating_sub(entry.last_frame) > MAX_CACHE_AGE {
                    removed.push(entry.texture_id);
                    false
                } else {
                    true
                }
            });
            !texts.is_empty()
        });
        self.len -= removed.len();

        let lru_count = (self.len + reserve).saturating_sub(MAX_CACHE_SIZE);
        if lru_count == 0 {
            return removed;
        }

        let mut last_frames = self
            .entries
            .values()
            .flat_map(|texts| texts.values().map(|entry| entry.last_frame))
            .collect::<Vec<_>>();
        last_frames.sort_unstable();
        let cutoff = last_frames[lru_count - 1];
        let mut remaining = lru_count;
        self.entries.retain(|_, texts| {
            texts.retain(|_, entry| {
                if remaining > 0 && entry.last_frame <= cutoff {
                    removed.push(entry.texture_id);
                    remaining -= 1;
                    false
                } else {
                    true
                }
            });
            !texts.is_empty()
        });
        self.len -= lru_count;
        removed
    }

    fn evict_textures(&mut self, renderer: &mut Renderer, frame: u64, reserve: usize) {
        for texture_id in self.evict(frame, reserve) {
            renderer.remove_texture(texture_id);
        }
    }

    fn lookup(&mut self, style: &TextCacheStyle, text: &str, frame: u64) -> Option<TextCacheEntry> {
        let entry = self.entries.get_mut(style)?.get_mut(text)?;
        entry.last_frame = frame;
        Some(*entry)
    }

    fn insert_rasterized(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        run: &TextRun,
        fx: i32,
        style: TextCacheStyle,
        frame: u64,
    ) {
        self.evict_textures(renderer, frame, 1);
        let Some(bitmap) = font.render_string_rgba(
            &run.text,
            fx,
            run.position.y,
            run.color,
            TEXT_SHADOW,
            &mut self.rgba_scratch,
        ) else {
            return;
        };
        let RgbaBitmap {
            pixels,
            x,
            y,
            width,
            height,
        } = bitmap;
        if pixels.iter().any(|&b| b != 0) {
            let texture_id = renderer.create_rgba_texture(&pixels, width, height);
            self.rgba_scratch = pixels;
            let Some(texture_id) = texture_id else {
                return;
            };
            let entry = TextCacheEntry {
                texture_id,
                x,
                y,
                width,
                height,
                last_frame: frame,
            };
            draw_text_texture(renderer, &entry);
            self.entries
                .entry(style)
                .or_default()
                .insert(run.text.clone(), entry);
            self.len += 1;
        } else {
            self.rgba_scratch = pixels;
        }
    }

    fn draw(&mut self, renderer: &mut Renderer, font: &Font, run: &TextRun, frame: u64) {
        let text_w = font.string_width(&run.text) as i32;
        let fx = match run.align {
            TextAlign::Center => run.position.x - text_w / 2,
            TextAlign::Right => run.position.x - text_w,
            TextAlign::Left => run.position.x,
        };

        let style = TextCacheStyle {
            x: fx,
            y: run.position.y,
            color: run.color,
        };
        if let Some(entry) = self.lookup(&style, &run.text, frame) {
            draw_text_texture(renderer, &entry);
            return;
        }

        self.insert_rasterized(renderer, font, run, fx, style, frame);
    }
}

fn draw_text_texture(renderer: &mut Renderer, entry: &TextCacheEntry) {
    renderer.draw_texture(
        entry.texture_id,
        None,
        Some(Rect::new(entry.x, entry.y, entry.width, entry.height)),
    );
}

pub struct DrawCommandRenderer {
    text_cache: TextCache,
    static_textures: HashMap<u64, StaticTextureEntry>,
    frame_counter: u64,
}

pub struct DrawRenderAssets<'a> {
    pub font: &'a Font,
    pub baked_sprites: &'a BakedSpriteTextures,
    pub pattern_texture: TextureId,
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
            static_textures: HashMap::new(),
            frame_counter: 0,
        }
    }

    pub(crate) fn render_frame(
        &mut self,
        renderer: &mut Renderer,
        assets: DrawRenderAssets<'_>,
        commands: &[DrawCommand],
        background: Option<TextureId>,
    ) {
        renderer.begin_frame();
        if let Some(bg) = background {
            renderer.draw_texture(bg, None, None);
        }

        let frame = self.frame_counter;
        self.frame_counter = self.frame_counter.wrapping_add(1);
        self.text_cache.evict_textures(renderer, frame, 0);
        self.evict_static_textures(renderer, frame);

        for command in commands {
            self.render_command(renderer, &assets, command, frame);
        }

        renderer.end_frame();
    }

    fn render_command(
        &mut self,
        renderer: &mut Renderer,
        assets: &DrawRenderAssets<'_>,
        command: &DrawCommand,
        frame: u64,
    ) {
        match command {
            DrawCommand::Fill(rect, color) => {
                renderer.draw_fill_rect(rect.x, rect.y, rect.w, rect.h, *color);
            }
            DrawCommand::Stroke(rect, color) => {
                renderer.draw_box(rect.x, rect.y, rect.w, rect.h, *color);
            }
            DrawCommand::PatternFill(rect, color) => {
                renderer.draw_fill_rect(rect.x, rect.y, rect.w, rect.h, *color);
                renderer.draw_tiled_pattern(
                    Rect::new(rect.x, rect.y, rect.w as u32, rect.h as u32),
                    *color,
                    assets.pattern_texture,
                    TILE_W,
                    TILE_H,
                );
            }
            DrawCommand::PatternStroke(rect, color) => {
                renderer.draw_box(rect.x, rect.y, rect.w, rect.h, *color);
                renderer.draw_tiled_pattern(
                    Rect::new(rect.x, rect.y, rect.w as u32, rect.h as u32),
                    *color,
                    assets.pattern_texture,
                    TILE_W,
                    TILE_H,
                );
            }
            DrawCommand::Text(run) => {
                self.text_cache.draw(renderer, assets.font, run, frame);
            }
            DrawCommand::Sprite(SpriteDraw { idx, position }) => {
                if let Some(texture) = assets.baked_sprites.default_sprite(*idx) {
                    draw_baked_sprite_texture(renderer, texture, position.x, position.y);
                }
            }
            DrawCommand::SpriteWithMaterial { sprite, material } => {
                assets
                    .baked_sprites
                    .ensure_material(sprite.idx, material, renderer);
                if let Some(texture) = assets.baked_sprites.material_sprite(sprite.idx, material) {
                    draw_baked_sprite_texture(
                        renderer,
                        &texture,
                        sprite.position.x,
                        sprite.position.y,
                    );
                }
            }
            DrawCommand::StaticImageRegion(region) => {
                let image_id = region.image.id();
                let texture_id = if let Some(entry) = self.static_textures.get_mut(&image_id) {
                    entry.last_used = frame;
                    entry.texture_id
                } else {
                    let Some(texture_id) = renderer.create_rgba_texture(
                        region.image.pixels(),
                        region.image.width(),
                        region.image.height(),
                    ) else {
                        return;
                    };
                    self.static_textures.insert(
                        image_id,
                        StaticTextureEntry {
                            texture_id,
                            last_used: frame,
                        },
                    );
                    texture_id
                };
                if let Some((source, destination)) = clip_static_region(
                    region.image.width(),
                    region.image.height(),
                    region.source,
                    region.destination,
                ) {
                    renderer.draw_texture_modulated_with_offset(
                        texture_id,
                        Some(Rect::new(
                            source.x,
                            source.y,
                            source.w as u32,
                            source.h as u32,
                        )),
                        Some(Rect::new(
                            destination.x,
                            destination.y,
                            destination.w as u32,
                            destination.h as u32,
                        )),
                        region.modulation,
                        (
                            region.destination_offset.0 as f32 / 256.0,
                            region.destination_offset.1 as f32 / 256.0,
                        ),
                    );
                }
            }
            DrawCommand::Pixels(batches) => {
                for (color, range) in batches.colors.iter().zip(&batches.ranges) {
                    renderer.draw_points(&batches.points[range.clone()], *color);
                }
            }
        }
    }

    fn evict_static_textures(&mut self, renderer: &mut Renderer, frame: u64) {
        let stale = stale_static_images(&self.static_textures, frame);
        for image_id in stale {
            if let Some(entry) = self.static_textures.remove(&image_id) {
                renderer.remove_texture(entry.texture_id);
            }
        }

        while self.static_textures.len() > MAX_STATIC_TEXTURES {
            let Some(image_id) = self
                .static_textures
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(image_id, _)| *image_id)
            else {
                break;
            };
            if let Some(entry) = self.static_textures.remove(&image_id) {
                renderer.remove_texture(entry.texture_id);
            }
        }
    }
}

fn stale_static_images(textures: &HashMap<u64, StaticTextureEntry>, frame: u64) -> Vec<u64> {
    textures
        .iter()
        .filter(|(_, entry)| frame.saturating_sub(entry.last_used) > MAX_STATIC_TEXTURE_AGE)
        .map(|(image_id, _)| *image_id)
        .collect()
}

fn clip_static_region(
    image_w: u32,
    image_h: u32,
    mut source: DrawRect,
    mut destination: DrawRect,
) -> Option<(DrawRect, DrawRect)> {
    if source.w <= 0 || source.h <= 0 || destination.w <= 0 || destination.h <= 0 {
        return None;
    }
    clip_axis(
        &mut source.x,
        &mut source.w,
        &mut destination.x,
        &mut destination.w,
        0,
        image_w as i32,
    )?;
    clip_axis(
        &mut source.y,
        &mut source.h,
        &mut destination.y,
        &mut destination.h,
        0,
        image_h as i32,
    )?;
    clip_axis(
        &mut destination.x,
        &mut destination.w,
        &mut source.x,
        &mut source.w,
        0,
        WIDTH as i32,
    )?;
    clip_axis(
        &mut destination.y,
        &mut destination.h,
        &mut source.y,
        &mut source.h,
        0,
        HEIGHT as i32,
    )?;
    Some((source, destination))
}

fn clip_axis(
    clipped_start: &mut i32,
    clipped_len: &mut i32,
    paired_start: &mut i32,
    paired_len: &mut i32,
    min: i32,
    max: i32,
) -> Option<()> {
    let end = i64::from(*clipped_start) + i64::from(*clipped_len);
    let new_start = i64::from((*clipped_start).max(min));
    let new_end = end.min(i64::from(max));
    if new_start >= new_end {
        return None;
    }
    let old_start = i64::from(*clipped_start);
    let old_len = i64::from(*clipped_len);
    let pair_start = i64::from(*paired_start);
    let pair_len = i64::from(*paired_len);
    let pair_at = |offset: i64| pair_start + offset * pair_len / old_len;
    let pair_end_at = |offset: i64| pair_start + (offset * pair_len + old_len - 1) / old_len;
    let left_offset = new_start - old_start;
    let right_offset = new_end - old_start;
    let new_pair_start = pair_at(left_offset);
    let new_pair_end = pair_end_at(right_offset);
    *clipped_start = new_start as i32;
    *clipped_len = (new_end - new_start) as i32;
    *paired_start = new_pair_start as i32;
    *paired_len = (new_pair_end - new_pair_start).max(1) as i32;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::HEIGHT;
    use crate::oxide::StaticImage;

    fn insert_entry(cache: &mut TextCache, text: String, last_frame: u64, id: u32) {
        cache
            .entries
            .entry(TextCacheStyle {
                x: 0,
                y: 0,
                color: Rgba::rgb(255, 255, 255),
            })
            .or_default()
            .insert(
                text,
                TextCacheEntry {
                    texture_id: TextureId::test_id(id),
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                    last_frame,
                },
            );
        cache.len += 1;
    }

    #[test]
    fn cache_evicts_lru_to_leave_capacity_for_insert() {
        let mut cache = TextCache::default();
        for id in 0..MAX_CACHE_SIZE as u32 {
            insert_entry(&mut cache, id.to_string(), u64::from(id), id);
        }

        let removed = cache.evict(MAX_CACHE_SIZE as u64, 1);

        assert_eq!(removed, vec![TextureId::test_id(0)]);
        assert_eq!(cache.len, MAX_CACHE_SIZE - 1);
        assert_eq!(
            cache.entries.values().map(HashMap::len).sum::<usize>(),
            cache.len
        );
    }

    #[test]
    fn cache_evicts_only_entries_past_max_age_when_under_capacity() {
        let mut cache = TextCache::default();
        insert_entry(&mut cache, "stale".into(), 0, 1);
        insert_entry(&mut cache, "recent".into(), MAX_CACHE_AGE, 2);

        let removed = cache.evict(MAX_CACHE_AGE + 1, 0);

        assert_eq!(removed, vec![TextureId::test_id(1)]);
        assert_eq!(cache.len, 1);
        assert!(cache
            .entries
            .values()
            .any(|texts| texts.contains_key("recent")));
    }

    #[test]
    fn static_region_clips_negative_source() {
        let clipped = clip_static_region(
            100,
            100,
            DrawRect::new(-10, 5, 30, 20),
            DrawRect::new(40, 50, 30, 20),
        );

        assert_eq!(
            clipped,
            Some((DrawRect::new(0, 5, 20, 20), DrawRect::new(50, 50, 20, 20),))
        );
    }

    #[test]
    fn static_region_clips_source_and_screen_edges() {
        let clipped = clip_static_region(
            100,
            100,
            DrawRect::new(90, 90, 20, 20),
            DrawRect::new(-5, HEIGHT as i32 - 5, 20, 20),
        );

        assert_eq!(
            clipped,
            Some((
                DrawRect::new(95, 90, 5, 5),
                DrawRect::new(0, HEIGHT as i32 - 5, 5, 5),
            ))
        );
    }

    #[test]
    fn static_region_rejects_empty_or_fully_clipped_regions() {
        assert_eq!(
            clip_static_region(
                10,
                10,
                DrawRect::new(10, 0, 1, 1),
                DrawRect::new(0, 0, 1, 1)
            ),
            None
        );
        assert_eq!(
            clip_static_region(
                10,
                10,
                DrawRect::new(0, 0, 1, 1),
                DrawRect::new(-2, 0, 1, 1)
            ),
            None
        );
    }

    #[test]
    fn static_texture_eviction_selects_old_entries() {
        let old = StaticImage::new(vec![255, 0, 0, 255], 1, 1);
        let fresh = StaticImage::new(vec![0, 255, 0, 255], 1, 1);
        let mut textures = HashMap::new();
        textures.insert(
            old.id(),
            StaticTextureEntry {
                texture_id: TextureId::test_id(1),
                last_used: 0,
            },
        );
        textures.insert(
            fresh.id(),
            StaticTextureEntry {
                texture_id: TextureId::test_id(2),
                last_used: MAX_STATIC_TEXTURE_AGE,
            },
        );

        assert_eq!(
            stale_static_images(&textures, MAX_STATIC_TEXTURE_AGE + 1),
            vec![old.id()]
        );
    }
}

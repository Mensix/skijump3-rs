use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use crate::color::Rgba;
use crate::consts::PATTERN_SPRITE;
use crate::sprite::{SpriteColorRecolor, SpriteData};
use crate::ui::Font;
use crate::video::{Renderer, TextureId};

#[derive(Debug, Clone)]
struct ImageRegion {
    pixels: Rc<[u8]>,
    src_w: u32,
    src_h: u32,
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    w: u32,
    h: u32,
}

#[derive(Debug, Clone)]
enum Element {
    Image(Rc<[u8]>, u32, u32),
    ImageRegion(ImageRegion),
    Text { text: String, x: i32, y: i32, color: Rgba, right: bool, center: bool },
    Sprite(u16, i32, i32),
    Fillbox { x: i32, y: i32, w: i32, h: i32, color: Rgba },
    FillArea { thing: u8 },
    Box { x: i32, y: i32, w: i32, h: i32, color: Rgba },
    SpriteRemapped(u16, i32, i32, SpriteColorRecolor),
    Container(Vec<Element>),
}

enum RenderCommand<'a> {
    Fillbox {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Rgba,
    },
    Box {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Rgba,
    },
    FillArea {
        thing: u8,
    },
    Text {
        text: &'a str,
        x: i32,
        y: i32,
        color: Rgba,
        right: bool,
        center: bool,
    },
    Sprite {
        idx: u16,
        x: i32,
        y: i32,
    },
    SpriteRemapped {
        idx: u16,
        x: i32,
        y: i32,
        recolor: &'a SpriteColorRecolor,
    },
    Image {
        pixels: &'a [u8],
        w: u32,
        h: u32,
    },
    ImageRegion(&'a ImageRegion),
}

fn flatten_elements<'a>(elements: &'a [Element], out: &mut Vec<RenderCommand<'a>>) {
    for element in elements {
        match element {
            Element::Fillbox { x, y, w, h, color } => out.push(RenderCommand::Fillbox {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                color: *color,
            }),
            Element::Box { x, y, w, h, color } => out.push(RenderCommand::Box {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                color: *color,
            }),
            Element::FillArea { thing } => out.push(RenderCommand::FillArea { thing: *thing }),
            Element::Text {
                text,
                x,
                y,
                color,
                right,
                center,
            } => out.push(RenderCommand::Text {
                text,
                x: *x,
                y: *y,
                color: *color,
                right: *right,
                center: *center,
            }),
            Element::Sprite(idx, x, y) => out.push(RenderCommand::Sprite {
                idx: *idx,
                x: *x,
                y: *y,
            }),
            Element::SpriteRemapped(idx, x, y, recolor) => {
                out.push(RenderCommand::SpriteRemapped {
                    idx: *idx,
                    x: *x,
                    y: *y,
                    recolor,
                })
            }
            Element::Image(pixels, w, h) => out.push(RenderCommand::Image {
                pixels: pixels.as_ref(),
                w: *w,
                h: *h,
            }),
            Element::ImageRegion(region) => out.push(RenderCommand::ImageRegion(region)),
            Element::Container(children) => flatten_elements(children, out),
        }
    }
}

struct DitherRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Rgba,
    is_box: bool,
}

// Dither-eligible fill colours, including screen backgrounds and menu-remapped
// equivalents that should receive the original patterned fill overlay.
const DITHER_FILL_COLORS: [Rgba; 5] = [
    Rgba::from_rgb6(18, 13, 34), // 243 default (BG_LEFT)
    Rgba::from_rgb6(34, 13, 18), // 244 default (BG_RIGHT)
    Rgba::from_rgb6(20, 20, 20), // 245        (FILL_DIM)
    Rgba::from_rgb6(0, 25, 0),   // KOTH       (BG_KOTH)
    Rgba::from_rgb6(28, 8, 24),  // Team Cup   (BG_TEAMCUP, MuutaMenu 1,2)
];

fn is_fill_area_dither_color(color: Rgba) -> bool {
    DITHER_FILL_COLORS.contains(&color)
}

const TEXT_SHADOW: Rgba = Rgba::rgb(0, 0, 0);

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
            self.pending.push(DitherRect {
                x,
                y,
                w,
                h,
                color,
                is_box,
            });
        }
    }

    fn apply_fill_area(
        &mut self,
        renderer: &mut Renderer,
        sprites: &[SpriteData],
        thing: u8,
    ) -> Result<(), String> {
        if let Some(pattern) = sprites.get(PATTERN_SPRITE) {
            for rect in self.pending.iter() {
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

// ---------------------------------------------------------------------------
// Remapped sprite RGBA cache
// ---------------------------------------------------------------------------

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
        let key = RemappedSpriteCacheKey {
            sprite_idx,
            recolor: recolor.clone(),
        };

        match self.entries.entry(key) {
            Entry::Occupied(o) => draw_remapped_sprite_entry(renderer, o.get(), x, y),
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
                    draw_remapped_sprite_entry(renderer, &entry, x, y)?;
                    v.insert(entry);
                }
                Ok(())
            }
        }
    }
}

fn draw_remapped_sprite_entry(
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

// ---------------------------------------------------------------------------
// Text RGBA cache
// ---------------------------------------------------------------------------

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
        right: bool,
        center: bool,
    ) -> Result<(), String> {
        let text_w = font.string_width(text) as i32;
        let fx = if center {
            x - text_w / 2
        } else if right {
            x - text_w
        } else {
            x
        };

        let key = TextCacheKey {
            text: text.to_owned(),
            x: fx,
            y,
            color,
        };

        match self.entries.entry(key) {
            Entry::Occupied(o) => draw_text_entry(renderer, o.get()),
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
                        draw_text_entry(renderer, &entry)?;
                        v.insert(entry);
                    }
                }
                Ok(())
            }
        }
    }
}

fn draw_text_entry(renderer: &mut Renderer, entry: &TextCacheEntry) -> Result<(), String> {
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

/// Persistent context for element rendering.  Owns reusable per-frame
/// scratch state so temporary allocations do not escape each frame.
pub struct ElementRenderContext {
    pending_dither_rects: Vec<DitherRect>,
    remapped_sprite_cache: RemappedSpriteCache,
    text_cache: TextCache,
}

impl Default for ElementRenderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementRenderContext {
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
        elements: &[Element],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        let mut commands = Vec::with_capacity(elements.len());
        flatten_elements(elements, &mut commands);

        let mut ew = ElementWorker {
            renderer,
            font,
            sprites,
            pending_dither_rects: &mut self.pending_dither_rects,
            text_cache: &mut self.text_cache,
            remapped_sprite_cache: &mut self.remapped_sprite_cache,
        };
        ew.render_frame(&commands, background)
    }
}

// ---------------------------------------------------------------------------
// Internal per-frame worker — borrows everything from the context.
// ---------------------------------------------------------------------------

struct ElementWorker<'a> {
    renderer: &'a mut Renderer,
    font: &'a Font,
    sprites: &'a [SpriteData],
    pending_dither_rects: &'a mut Vec<DitherRect>,
    text_cache: &'a mut TextCache,
    remapped_sprite_cache: &'a mut RemappedSpriteCache,
}

impl ElementWorker<'_> {
    fn render_frame(
        &mut self,
        commands: &[RenderCommand<'_>],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        self.renderer.begin_frame();
        if let Some(bg) = background {
            self.renderer.draw_texture(bg, None, None)?;
        }

        let fill_area_count = count_fill_area_commands(commands);
        let pending_dither_rects = std::mem::take(self.pending_dither_rects);
        let mut dither = DitherFillCollector::new(pending_dither_rects, fill_area_count);

        for command in commands {
            self.render_command(command, &mut dither)?;
        }

        *self.pending_dither_rects = dither.into_pending();

        self.renderer.end_frame();
        Ok(())
    }

    fn render_command(
        &mut self,
        command: &RenderCommand<'_>,
        dither: &mut DitherFillCollector,
    ) -> Result<(), String> {
        match command {
            RenderCommand::Fillbox { x, y, w, h, color } => {
                self.renderer.draw_fill_rect(*x, *y, *w, *h, *color)?;
                dither.track_fillbox(*x, *y, *w, *h, *color);
            }
            RenderCommand::Box { x, y, w, h, color } => {
                self.renderer.draw_box(*x, *y, *w, *h, *color)?;
                dither.track_box(*x, *y, *w, *h, *color);
            }
            RenderCommand::FillArea { thing } => {
                dither.apply_fill_area(self.renderer, self.sprites, *thing)?;
            }
            RenderCommand::Text {
                text,
                x,
                y,
                color,
                right,
                center,
            } => {
                self.text_cache.draw(
                    self.renderer,
                    self.font,
                    text,
                    *x,
                    *y,
                    *color,
                    *right,
                    *center,
                )?;
            }
            RenderCommand::Sprite { idx, x, y } => {
                if let Some(sprite) = self.sprites.get(*idx as usize) {
                    if let Some(bitmap) = sprite.render_rgba_bitmap(*x, *y) {
                        self.renderer.draw_rgba_region_pixels(
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
            RenderCommand::SpriteRemapped { idx, x, y, recolor } => {
                if let Some(sprite) = self.sprites.get(*idx as usize) {
                    self.remapped_sprite_cache.draw(
                        self.renderer,
                        *idx,
                        *x,
                        *y,
                        recolor,
                        sprite,
                    )?;
                }
            }
            RenderCommand::Image { pixels, w, h } => {
                self.renderer
                    .draw_rgba_region_pixels(pixels, *w, *h, 0, 0, 0, 0, *w, *h)?;
            }
            RenderCommand::ImageRegion(region) => {
                self.renderer.draw_rgba_region_pixels(
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

fn count_fill_area_commands(commands: &[RenderCommand<'_>]) -> usize {
    commands
        .iter()
        .filter(|command| matches!(command, RenderCommand::FillArea { .. }))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dither_rect_tracks_eligible_rects_in_pending() {
        let dr = DitherRect {
            x: 10,
            y: 20,
            w: 30,
            h: 40,
            color: Rgba::rgb(137, 52, 72),
            is_box: false,
        };
        assert_eq!(dr.x, 10);
        assert_eq!(dr.y, 20);
        assert_eq!(dr.w, 30);
        assert_eq!(dr.h, 40);
        assert_eq!(dr.color, Rgba::rgb(137, 52, 72));
        assert!(!dr.is_box);
    }

    #[test]
    fn dither_rect_box_default_is_box() {
        let dr = DitherRect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
            color: Rgba::rgb(80, 80, 80),
            is_box: true,
        };
        assert!(dr.is_box);
        assert_eq!(dr.color, Rgba::rgb(80, 80, 80));
    }

    #[test]
    fn is_fill_area_dither_color_eligible() {
        assert!(is_fill_area_dither_color(DITHER_FILL_COLORS[0]));
        assert!(is_fill_area_dither_color(DITHER_FILL_COLORS[1]));
        assert!(is_fill_area_dither_color(DITHER_FILL_COLORS[2]));
    }

    #[test]
    fn is_fill_area_dither_color_ineligible() {
        assert!(!is_fill_area_dither_color(Rgba::rgb(0, 0, 0)));
        assert!(!is_fill_area_dither_color(Rgba::rgb(100, 100, 100)));
        assert!(!is_fill_area_dither_color(Rgba::rgb(255, 255, 255)));
    }

    #[test]
    fn is_fill_area_dither_color_black_not_eligible() {
        assert!(!is_fill_area_dither_color(Rgba::rgb(0, 0, 0)));
    }
}

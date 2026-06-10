use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::color::Rgba;
use crate::consts::PATTERN_SPRITE;
use crate::sprite::{SpriteColorRecolor, SpriteData};
use crate::ui::{Element, Font};
use crate::video::{Renderer, TextureId};

struct DitherRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Rgba,
    is_box: bool,
}

// Dither-eligible fill colours — precomputed from the original Pascal
// STANDARD_UI_PALETTE entries for palette indices 243/244/245 plus any
// MuutaMenu-remapped equivalents (e.g. Team Cup bishop's purple).
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

// ---------------------------------------------------------------------------
// Text RGBA cache
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextCacheKey {
    text: String,
    x: i32,
    y: i32,
    color: Rgba,
    right: bool,
    center: bool,
}

struct TextCacheEntry {
    texture_id: TextureId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

/// Persistent context for element rendering.  Owns reusable per-frame
/// scratch state so temporary allocations do not escape each frame.
pub struct ElementRenderContext {
    pending_dither_rects: Vec<DitherRect>,
    remapped_sprite_cache: HashMap<RemappedSpriteCacheKey, RemappedSpriteCacheEntry>,
    remapped_rgba_scratch: Vec<u8>,
    text_cache: HashMap<TextCacheKey, TextCacheEntry>,
    text_rgba_scratch: Vec<u8>,
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
            remapped_sprite_cache: HashMap::new(),
            remapped_rgba_scratch: Vec::new(),
            text_cache: HashMap::new(),
            text_rgba_scratch: Vec::new(),
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
        self.pending_dither_rects.clear();
        let mut ew = ElementWorker {
            renderer,
            font,
            sprites,
            pending_dither_rects: &mut self.pending_dither_rects,
            remapped_sprite_cache: &mut self.remapped_sprite_cache,
            remapped_rgba_scratch: &mut self.remapped_rgba_scratch,
            text_cache: &mut self.text_cache,
            text_rgba_scratch: &mut self.text_rgba_scratch,
        };
        ew.render_frame(elements, background)
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
    remapped_sprite_cache: &'a mut HashMap<RemappedSpriteCacheKey, RemappedSpriteCacheEntry>,
    remapped_rgba_scratch: &'a mut Vec<u8>,
    text_cache: &'a mut HashMap<TextCacheKey, TextCacheEntry>,
    text_rgba_scratch: &'a mut Vec<u8>,
}

impl ElementWorker<'_> {
    fn render_frame(
        &mut self,
        elements: &[Element],
        background: Option<TextureId>,
    ) -> Result<(), String> {
        self.renderer.begin_frame();
        if let Some(bg) = background {
            self.renderer.draw_texture(bg, None, None)?;
        }

        let mut remaining_fill_areas = count_fill_areas(elements);

        for el in elements {
            self.render_element(el, &mut remaining_fill_areas)?;
        }

        self.renderer.end_frame();
        Ok(())
    }

    fn render_element(
        &mut self,
        element: &Element,
        remaining_fill_areas: &mut usize,
    ) -> Result<(), String> {
        match element {
            Element::Fillbox { x, y, w, h, color } if *remaining_fill_areas > 0 => {
                self.renderer.draw_fill_rect(*x, *y, *w, *h, *color)?;
                if is_fill_area_dither_color(*color) {
                    self.pending_dither_rects.push(DitherRect {
                        x: *x,
                        y: *y,
                        w: *w,
                        h: *h,
                        color: *color,
                        is_box: false,
                    });
                }
            }
            Element::Box { x, y, w, h, color } if *remaining_fill_areas > 0 => {
                self.renderer.draw_box(*x, *y, *w, *h, *color)?;
                if is_fill_area_dither_color(*color) {
                    self.pending_dither_rects.push(DitherRect {
                        x: *x,
                        y: *y,
                        w: *w,
                        h: *h,
                        color: *color,
                        is_box: true,
                    });
                }
            }
            Element::Fillbox { x, y, w, h, color } => {
                self.renderer.draw_fill_rect(*x, *y, *w, *h, *color)?;
            }
            Element::Box { x, y, w, h, color } => {
                self.renderer.draw_box(*x, *y, *w, *h, *color)?;
            }
            Element::FillArea { thing } => {
                if let Some(pattern) = self.sprites.get(PATTERN_SPRITE) {
                    for dr in self.pending_dither_rects.iter() {
                        self.renderer.dither_overlay_rect(
                            dr.x,
                            dr.y,
                            dr.w,
                            dr.h,
                            dr.color,
                            dr.is_box,
                            *thing,
                            &pattern.data,
                        )?;
                    }
                }
                if !self.pending_dither_rects.is_empty() {
                    self.renderer.flush_dither_overlay()?;
                    self.pending_dither_rects.clear();
                }
                *remaining_fill_areas = remaining_fill_areas.saturating_sub(1);
            }
            Element::Container(children) => {
                for child in children {
                    self.render_element(child, remaining_fill_areas)?;
                }
            }
            Element::Text {
                text,
                x,
                y,
                color,
                right,
                center,
            } => {
                let text_w = self.font.string_width(text) as i32;
                let fx = if *center {
                    x - text_w / 2
                } else if *right {
                    x - text_w
                } else {
                    *x
                };

                let key = TextCacheKey {
                    text: text.clone(),
                    x: fx,
                    y: *y,
                    color: *color,
                    right: *right,
                    center: *center,
                };

                match self.text_cache.entry(key) {
                    Entry::Occupied(o) => {
                        let e = o.get();
                        self.renderer.draw_texture(
                            e.texture_id,
                            None,
                            Some(sdl2::rect::Rect::new(e.x, e.y, e.width, e.height)),
                        )?;
                    }
                    Entry::Vacant(v) => {
                        if let Some(bitmap) = self.font.render_string_rgba(
                            text,
                            fx,
                            *y,
                            *color,
                            TEXT_SHADOW,
                            self.text_rgba_scratch,
                        ) {
                            if bitmap.pixels.iter().any(|&b| b != 0) {
                                let tex_id = self.renderer.create_rgba_texture(
                                    &bitmap.pixels,
                                    bitmap.width,
                                    bitmap.height,
                                )?;
                                let e = TextCacheEntry {
                                    texture_id: tex_id,
                                    x: bitmap.x,
                                    y: bitmap.y,
                                    width: bitmap.width,
                                    height: bitmap.height,
                                };
                                self.renderer.draw_texture(
                                    e.texture_id,
                                    None,
                                    Some(sdl2::rect::Rect::new(e.x, e.y, e.width, e.height)),
                                )?;
                                v.insert(e);
                            }
                        }
                    }
                }
            }
            Element::Sprite(idx, x, y) => {
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
            Element::SpriteRemapped(idx, x, y, recolor) => {
                if let Some(sprite) = self.sprites.get(*idx as usize) {
                    let key = RemappedSpriteCacheKey {
                        sprite_idx: *idx,
                        recolor: recolor.clone(),
                    };

                    match self.remapped_sprite_cache.entry(key) {
                        Entry::Occupied(o) => {
                            let e = o.get();
                            let dst_x = *x - i32::from(e.center_x);
                            let dst_y = *y - i32::from(e.center_y);
                            self.renderer.draw_texture(
                                e.texture_id,
                                None,
                                Some(sdl2::rect::Rect::new(
                                    dst_x,
                                    dst_y,
                                    u32::from(e.width),
                                    u32::from(e.height),
                                )),
                            )?;
                        }
                        Entry::Vacant(v) => {
                            sprite.render_recolored_rgba(recolor, self.remapped_rgba_scratch);

                            // Only create a texture when at least one opaque
                            // pixel exists (saves a GPU upload for invisible
                            // animation frames / fully-transparent sprites).
                            if self.remapped_rgba_scratch.iter().any(|&b| b != 0) {
                                let tex_id = self.renderer.create_rgba_texture(
                                    self.remapped_rgba_scratch,
                                    u32::from(sprite.width),
                                    u32::from(sprite.height),
                                )?;
                                let e = RemappedSpriteCacheEntry {
                                    texture_id: tex_id,
                                    center_x: sprite.center_x,
                                    center_y: sprite.center_y,
                                    width: sprite.width,
                                    height: sprite.height,
                                };
                                let dst_x = *x - i32::from(e.center_x);
                                let dst_y = *y - i32::from(e.center_y);
                                self.renderer.draw_texture(
                                    e.texture_id,
                                    None,
                                    Some(sdl2::rect::Rect::new(
                                        dst_x,
                                        dst_y,
                                        u32::from(e.width),
                                        u32::from(e.height),
                                    )),
                                )?;
                                v.insert(e);
                            }
                        }
                    }
                }
            }
            Element::Image(pixels, w, h) => {
                self.renderer
                    .draw_rgba_region_pixels(pixels, *w, *h, 0, 0, 0, 0, *w, *h)?;
            }
            Element::ImageRegion(region) => {
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

fn count_fill_areas(elements: &[Element]) -> usize {
    elements.iter().map(count_fill_areas_in_element).sum()
}

fn count_fill_areas_in_element(element: &Element) -> usize {
    match element {
        Element::FillArea { .. } => 1,
        Element::Container(children) => count_fill_areas(children),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_fill_areas_empty() {
        assert_eq!(count_fill_areas(&[]), 0);
    }

    #[test]
    fn count_fill_areas_no_fillareas() {
        let els = vec![
            Element::fillbox(0, 0, 10, 10, Rgba::rgb(1, 1, 1)),
            Element::text("hi", 0, 0, Rgba::rgb(2, 2, 2), false),
        ];
        assert_eq!(count_fill_areas(&els), 0);
    }

    #[test]
    fn count_fill_areas_single() {
        let els = vec![Element::fill_area(64)];
        assert_eq!(count_fill_areas(&els), 1);
    }

    #[test]
    fn count_fill_areas_nested_in_containers() {
        let els = vec![
            Element::container(vec![
                Element::fill_area(63),
                Element::container(vec![
                    Element::fill_area(63),
                    Element::fillbox(0, 0, 10, 10, Rgba::rgb(1, 1, 1)),
                ]),
            ]),
            Element::fill_area(64),
        ];
        assert_eq!(count_fill_areas(&els), 3);
    }

    #[test]
    fn count_fill_areas_mixed() {
        let els = vec![
            Element::fillbox(0, 0, 10, 10, Rgba::rgb(1, 1, 1)),
            Element::text("test", 0, 0, Rgba::rgb(2, 2, 2), false),
            Element::fill_area(63),
            Element::sprite(0, 0, 0),
        ];
        assert_eq!(count_fill_areas(&els), 1);
    }

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

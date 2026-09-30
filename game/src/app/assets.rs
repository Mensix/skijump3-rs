use std::io::Cursor;

use super::embedded::{main_png, sprite_png};
use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::materials;
use crate::gfx::png::load_png;
use crate::gfx::sprites::Sprite;
use crate::ui::{Font, Glyph};
use engine::color::Rgba;
use engine::consts::{FONT_GLYPH_COUNT, PATTERN_SPRITE, TILE_H, TILE_W};
use engine::sprite::{sprite_to_rgba, BakedSpriteTextures, BaseSprite};
use engine::video::{Renderer, TextureId};

const PATTERN_IDX: u16 = PATTERN_SPRITE as u16;

pub(super) struct LoadedAssets {
    pub(super) content_store: ContentStore,
    pub(super) font: Font,
    pub(super) main_background: TextureId,
    pub(super) baked_sprites: BakedSpriteTextures,
    pub(super) pattern_texture: TextureId,
}

pub(super) fn load(files: &FileStore, renderer: &mut Renderer) -> Option<LoadedAssets> {
    let content_store = ContentStore::load(files).ok()?;
    let main_background = load_background_texture(renderer)?;

    let mut glyphs: Vec<Glyph> = Vec::new();
    let mut base_sprites = Vec::new();
    for idx in 0..=176u16 {
        let data: &[u8] = sprite_png(idx);
        let (indices, palette, png_w, png_h) = decode_indexed_png(data)?;
        let (cx, cy) = sprite_center_from_png(data).unwrap_or((0, 0));

        if usize::from(idx) < FONT_GLYPH_COUNT {
            glyphs.push(Glyph {
                pixels: indices.clone().into_boxed_slice(),
                width: png_w as u16,
                height: png_h as u16,
                center_x: 0,
                center_y: 0,
            });
        }

        base_sprites.push(BaseSprite {
            sprite_idx: idx,
            indices,
            palette,
            width: png_w,
            height: png_h,
            center_x: cx,
            center_y: cy,
        });
    }
    let font = Font::from_sprites(&glyphs);

    if let Some(logo) = base_sprites
        .iter()
        .find(|sprite| sprite.sprite_idx == Sprite::Logo as u16)
    {
        let mut icon = Vec::new();
        sprite_to_rgba(logo, &logo.palette, &mut icon);
        renderer.set_icon(&mut icon, logo.width, logo.height);
    }

    let baked_sprites = BakedSpriteTextures::bake_with_png(
        renderer,
        &base_sprites,
        &materials::prebaked_sprite_materials(),
    )?;

    let pattern_data: &[u8] = sprite_png(PATTERN_IDX);
    let (pattern_indices, _, _, _) = decode_indexed_png(pattern_data)?;
    let pattern_texture = renderer.create_pattern_texture(&pattern_indices, TILE_W, TILE_H)?;

    Some(LoadedAssets {
        content_store,
        font,
        main_background,
        baked_sprites,
        pattern_texture,
    })
}

fn sprite_center_from_png(data: &[u8]) -> Option<(i8, i8)> {
    let cursor = Cursor::new(data);
    let decoder = png::Decoder::new(cursor);
    let reader = decoder.read_info().ok()?;
    for chunk in &reader.info().uncompressed_latin1_text {
        if chunk.keyword == "cXcY" {
            let (cx_str, cy_str) = chunk.text.split_once(',')?;
            let cx: i8 = cx_str.trim().parse().ok()?;
            let cy: i8 = cy_str.trim().parse().ok()?;
            return Some((cx, cy));
        }
    }
    None
}

fn decode_indexed_png(data: &[u8]) -> Option<(Vec<u8>, [Rgba; 256], u32, u32)> {
    let cursor = Cursor::new(data);
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info();

    let palette_rgb = info.palette.as_deref()?;
    if palette_rgb.is_empty() || !palette_rgb.len().is_multiple_of(3) {
        return None;
    }
    let trns = info.trns.as_deref().unwrap_or(&[]);

    let mut palette = [Rgba::transparent(); 256];
    for i in 0..256 {
        if i < palette_rgb.len() / 3 {
            palette[i] = Rgba {
                r: palette_rgb[i * 3],
                g: palette_rgb[i * 3 + 1],
                b: palette_rgb[i * 3 + 2],
                a: trns.get(i).copied().unwrap_or(255),
            };
        }
    }

    let width = info.width;
    let height = info.height;
    if width == 0 || height == 0 || width > u32::from(u16::MAX) || height > u32::from(u16::MAX) {
        return None;
    }
    let buffer_len = width.checked_mul(height)?.try_into().ok()?;
    let mut buf = vec![0u8; buffer_len];
    reader.next_frame(&mut buf).ok()?;

    Some((buf, palette, width, height))
}

fn load_background_texture(renderer: &mut Renderer) -> Option<TextureId> {
    let img = load_png(main_png())?;
    renderer.create_rgba_texture(&img.pixels, img.width, img.height)
}

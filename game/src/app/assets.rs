use std::io::Cursor;

use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::materials;
use engine::color::Rgba;
use engine::consts::{PATTERN_SPRITE, TILE_H, TILE_W};
use engine::oxide::Font;
use engine::sprite::{BakedSpriteTextures, BaseSprite};
use engine::video::{Renderer, TextureId};

const MAIN_PNG: &str = "MAIN.png";
const CONTENT_MANIFEST: &str = "content.toml";
const SPRITES_PNG_PREFIX: &str = "sprites/png/";

pub(super) struct LoadedAssets {
    pub(super) content_store: ContentStore,
    pub(super) font: Font,
    pub(super) main_background: TextureId,
    pub(super) baked_sprites: BakedSpriteTextures,
    pub(super) pattern_texture: TextureId,
}

pub(super) fn load(files: &FileStore, renderer: &mut Renderer) -> Result<LoadedAssets, String> {
    let content_store = ContentStore::load(files, CONTENT_MANIFEST).map_err(|e| e.to_string())?;
    let sprites = content_store.sprites.clone();
    let font = Font::from_sprites(&sprites);
    let main_background = load_background_texture(files, renderer)?;

    // Load indexed PNGs (type 3) — extract palette indices + embedded palette
    let mut base_sprites = Vec::new();
    for (idx, sprite) in sprites.iter().enumerate() {
        let path = format!("{SPRITES_PNG_PREFIX}{idx}.png");
        let data = match files.read(&path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let (indices, palette) = decode_indexed_png(&data)?;
        base_sprites.push(BaseSprite {
            sprite_idx: idx as u16,
            indices,
            palette,
            width: u32::from(sprite.width),
            height: u32::from(sprite.height),
            center_x: sprite.center_x,
            center_y: sprite.center_y,
        });
    }

    // Bake material variants at startup — override palette indices then resolve
    let baked_sprites = BakedSpriteTextures::bake_with_png(
        renderer,
        &base_sprites,
        &materials::prebaked_sprite_materials(),
    )?;

    let pattern_sprite = sprites
        .get(PATTERN_SPRITE)
        .ok_or_else(|| "Pattern sprite 62 not found".to_string())?;
    let pattern_texture =
        renderer.create_pattern_texture(&pattern_sprite.pixels, TILE_W, TILE_H)?;

    Ok(LoadedAssets {
        content_store,
        font,
        main_background,
        baked_sprites,
        pattern_texture,
    })
}

/// Decode a type-3 (indexed) PNG, returning (palette_indices, [Rgba; 256]).
fn decode_indexed_png(data: &[u8]) -> Result<(Vec<u8>, [Rgba; 256]), String> {
    let cursor = Cursor::new(data);
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let info = reader.info();

    let palette_rgb = info
        .palette
        .as_deref()
        .ok_or_else(|| "PNG has no PLTE chunk".to_string())?;
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

    let out_len = info.raw_bytes();
    let mut buf = vec![0u8; out_len];
    reader.next_frame(&mut buf).map_err(|e| e.to_string())?;

    Ok((buf, palette))
}

fn load_background_texture(
    files: &FileStore,
    renderer: &mut Renderer,
) -> Result<TextureId, String> {
    let png_data = files.read(MAIN_PNG).map_err(|e| e.to_string())?;
    let img = crate::gfx::png::load_png(&png_data).map_err(|e| e.to_string())?;
    renderer.create_rgba_texture(&img.pixels, img.width, img.height)
}

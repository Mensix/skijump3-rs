use std::io::Cursor;

use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::materials;
use engine::color::Rgba;
use engine::consts::{FONT_GLYPH_COUNT, PATTERN_SPRITE, TILE_H, TILE_W};
use engine::oxide::{Font, Glyph};
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
    let main_background = load_background_texture(files, renderer)?;

    // Load font glyphs from indexed PNGs (indices 0..FONT_GLYPH_COUNT)
    let mut glyphs: Vec<Glyph> = Vec::new();
    for idx in 0..FONT_GLYPH_COUNT {
        let path = format!("{SPRITES_PNG_PREFIX}{idx}.png");
        let data = files.read(&path).map_err(|e| e.to_string())?;
        let (indices, _palette, width, height) = decode_indexed_png(&data)?;
        glyphs.push(Glyph {
            pixels: indices.into_boxed_slice(),
            width: width as u16,
            height: height as u16,
            center_x: 0,
            center_y: 0,
        });
    }
    let font = Font::from_sprites(&glyphs);

    // Load center_x/center_y metadata from TOML
    let centers_raw = files
        .read("sprites/png/centers.toml")
        .map_err(|e| e.to_string())?;
    let centers_val: toml::Value = toml::from_slice(&centers_raw).map_err(|e| e.to_string())?;
    let centers_map = match &centers_val {
        toml::Value::Table(t) => t,
        _ => return Err("centers.toml is not a table".to_string()),
    };

    // Load base sprites from indexed PNGs + centers
    let mut base_sprites = Vec::new();
    for idx in 0..=176u16 {
        let path = format!("{SPRITES_PNG_PREFIX}{idx}.png");
        let data = match files.read(&path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let (indices, palette, png_w, png_h) = decode_indexed_png(&data)?;

        let (cx, cy) = centers_map
            .get(&idx.to_string())
            .and_then(|v| v.as_table())
            .and_then(|t| {
                Some((
                    t.get("cx")?.as_integer()? as i8,
                    t.get("cy")?.as_integer()? as i8,
                ))
            })
            .unwrap_or((0, 0));

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

    // Bake material variants at startup
    let baked_sprites = BakedSpriteTextures::bake_with_png(
        renderer,
        &base_sprites,
        &materials::prebaked_sprite_materials(),
    )?;

    // Load pattern sprite 62 from indexed PNG
    let pattern_path = format!("{SPRITES_PNG_PREFIX}{PATTERN_SPRITE}.png");
    let pattern_data = files
        .read(&pattern_path)
        .map_err(|_| "Pattern sprite 62 not found".to_string())?;
    let (pattern_indices, _palette, _pw, _ph) = decode_indexed_png(&pattern_data)?;
    let pattern_texture = renderer.create_pattern_texture(&pattern_indices, TILE_W, TILE_H)?;

    Ok(LoadedAssets {
        content_store,
        font,
        main_background,
        baked_sprites,
        pattern_texture,
    })
}

/// Decode a type-3 (indexed) PNG, returning (palette_indices, [Rgba; 256], width, height).
fn decode_indexed_png(data: &[u8]) -> Result<(Vec<u8>, [Rgba; 256], u32, u32), String> {
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

    let width = info.width;
    let height = info.height;
    let mut buf = vec![0u8; (width * height) as usize];
    reader.next_frame(&mut buf).map_err(|e| e.to_string())?;

    Ok((buf, palette, width, height))
}

fn load_background_texture(
    files: &FileStore,
    renderer: &mut Renderer,
) -> Result<TextureId, String> {
    let png_data = files.read(MAIN_PNG).map_err(|e| e.to_string())?;
    let img = crate::gfx::png::load_png(&png_data).map_err(|e| e.to_string())?;
    renderer.create_rgba_texture(&img.pixels, img.width, img.height)
}

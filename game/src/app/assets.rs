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

pub(super) fn load(files: &FileStore, renderer: &mut Renderer) -> LoadedAssets {
    let content_store = ContentStore::load(files, CONTENT_MANIFEST);
    let main_background = load_background_texture(files, renderer);

    let mut glyphs: Vec<Glyph> = Vec::new();
    for idx in 0..FONT_GLYPH_COUNT {
        let path = format!("{SPRITES_PNG_PREFIX}{idx}.png");
        let data = files.read(&path);
        let (indices, _palette, width, height) = decode_indexed_png(&data);
        glyphs.push(Glyph {
            pixels: indices.into_boxed_slice(),
            width: width as u16,
            height: height as u16,
            center_x: 0,
            center_y: 0,
        });
    }
    let font = Font::from_sprites(&glyphs);

    let mut base_sprites = Vec::new();
    for idx in 0..=176u16 {
        let path = format!("{SPRITES_PNG_PREFIX}{idx}.png");
        let data = files.read(&path);
        let (indices, palette, png_w, png_h) = decode_indexed_png(&data);
        let (cx, cy) = sprite_center_from_png(&data).unwrap_or((0, 0));

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

    let baked_sprites =
        BakedSpriteTextures::bake_with_png(renderer, &base_sprites, &materials::prebaked_sprite_materials())
            .unwrap();

    let pattern_path = format!("{SPRITES_PNG_PREFIX}{PATTERN_SPRITE}.png");
    let pattern_data = files.read(&pattern_path);
    let (pattern_indices, _palette, _pw, _ph) = decode_indexed_png(&pattern_data);
    let pattern_texture = renderer.create_pattern_texture(&pattern_indices, TILE_W, TILE_H).unwrap();

    LoadedAssets {
        content_store,
        font,
        main_background,
        baked_sprites,
        pattern_texture,
    }
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

fn decode_indexed_png(data: &[u8]) -> (Vec<u8>, [Rgba; 256], u32, u32) {
    let cursor = Cursor::new(data);
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info().unwrap();
    let info = reader.info();

    let palette_rgb = info.palette.as_deref().unwrap();
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
    reader.next_frame(&mut buf).unwrap();

    (buf, palette, width, height)
}

fn load_background_texture(files: &FileStore, renderer: &mut Renderer) -> TextureId {
    let png_data = files.read(MAIN_PNG);
    let img = crate::gfx::png::load_png(&png_data);
    renderer
        .create_rgba_texture(&img.pixels, img.width, img.height)
        .unwrap()
}

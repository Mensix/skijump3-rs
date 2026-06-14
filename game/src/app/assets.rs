use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::materials;
use crate::gfx::png::load_png;
use engine::color::Rgba;
use engine::consts::{PATTERN_SPRITE, TILE_H, TILE_W};
use engine::oxide::Font;
use engine::sprite::{BakedSpriteTextures, BaseSprite, SourceColorMap};
use engine::video::{Renderer, TextureId};

const MAIN_PNG: &str = "MAIN.png";
const CONTENT_MANIFEST: &str = "content.toml";
const SPRITES_PNG_PREFIX: &str = "sprites/png/";

#[derive(serde::Deserialize)]
struct SourceColorEntry {
    idx: u8,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(serde::Deserialize)]
struct SourceColorsFile {
    source: Vec<SourceColorEntry>,
}

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

    // Load base sprites from PNG files
    let mut base_sprites = Vec::new();
    for (idx, sprite) in sprites.iter().enumerate() {
        let path = format!("{SPRITES_PNG_PREFIX}{idx}.png");
        let data = match files.read(&path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let img = load_png(&data).map_err(|e| e.to_string())?;
        base_sprites.push(BaseSprite {
            sprite_idx: idx as u16,
            rgba: img.pixels,
            width: img.width,
            height: img.height,
            center_x: sprite.center_x,
            center_y: sprite.center_y,
        });
    }

    // Load source colors (original RGBA for each override palette index)
    let src_data = files
        .read("sprites/png/source_colors.toml")
        .map_err(|e| e.to_string())?;
    let colors_file: SourceColorsFile = toml::from_slice(&src_data).map_err(|e| e.to_string())?;

    let source_colors: Vec<(u8, Rgba)> = colors_file
        .source
        .iter()
        .map(|e| {
            (
                e.idx,
                Rgba {
                    r: e.r,
                    g: e.g,
                    b: e.b,
                    a: e.a,
                },
            )
        })
        .collect();
    let source_map = SourceColorMap::from_entries(&source_colors);

    // Bake material variants at startup by recolor-matching
    let baked_sprites = BakedSpriteTextures::bake_with_png(
        renderer,
        &base_sprites,
        &source_map,
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

fn load_background_texture(
    files: &FileStore,
    renderer: &mut Renderer,
) -> Result<TextureId, String> {
    let png_data = files.read(MAIN_PNG).map_err(|e| e.to_string())?;
    let img = load_png(&png_data).map_err(|e| e.to_string())?;
    renderer.create_rgba_texture(&img.pixels, img.width, img.height)
}

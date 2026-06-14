use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::materials;
use crate::gfx::png::load_png;
use engine::consts::{PATTERN_SPRITE, TILE_H, TILE_W};
use engine::oxide::Font;
use engine::sprite::{BakedSpriteTexture, BakedSpriteTextures, SpriteData};
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

    let mut baked_sprites = BakedSpriteTextures::new();

    for (idx, sprite) in sprites.iter().enumerate() {
        let path = format!("{SPRITES_PNG_PREFIX}{idx}.png");
        if let Ok(tex) = load_png_texture(files, renderer, &path, sprite) {
            baked_sprites.add_default(idx as u16, tex);
        }
    }

    for (sprite_idx, material) in &materials::prebaked_sprite_materials() {
        let path = format!("{SPRITES_PNG_PREFIX}{}_{}.png", sprite_idx, material.id().value());
        if let Ok(tex) = load_png_texture(files, renderer, &path, &sprites[*sprite_idx as usize]) {
            baked_sprites.add_material(*sprite_idx, material.clone(), tex);
        }
    }

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

fn load_png_texture(
    files: &FileStore,
    renderer: &mut Renderer,
    path: &str,
    sprite: &SpriteData,
) -> Result<BakedSpriteTexture, String> {
    let data = files.read(path).map_err(|e| e.to_string())?;
    let img = load_png(&data).map_err(|e| e.to_string())?;
    let texture_id = renderer.create_rgba_texture(&img.pixels, img.width, img.height)?;
    Ok(BakedSpriteTexture {
        texture_id,
        center_x: sprite.center_x,
        center_y: sprite.center_y,
        width: img.width as u16,
        height: img.height as u16,
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

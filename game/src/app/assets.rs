use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::materials;
use crate::gfx::png::load_png;
use engine::consts::{PATTERN_SPRITE, TILE_H, TILE_W};
use engine::oxide::Font;
use engine::sprite::{BakedSpriteTextures, BaseSprite, SpriteData};
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

    // Bake material variants at startup — alpha-channel recolor matching
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

fn load_background_texture(
    files: &FileStore,
    renderer: &mut Renderer,
) -> Result<TextureId, String> {
    let png_data = files.read(MAIN_PNG).map_err(|e| e.to_string())?;
    let img = load_png(&png_data).map_err(|e| e.to_string())?;
    renderer.create_rgba_texture(&img.pixels, img.width, img.height)
}

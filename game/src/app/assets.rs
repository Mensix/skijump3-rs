use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::materials;
use crate::gfx::palette::Rgb6Palette;
use crate::gfx::png::load_png;
use engine::consts::{PATTERN_SPRITE, TILE_H, TILE_W};
use engine::oxide::Font;
use engine::sprite::{BakedSpriteTextures, SpriteData};
use engine::video::{Renderer, TextureId};

const MAIN_PNG: &str = "MAIN.png";
const CONTENT_MANIFEST: &str = "content.toml";

pub(super) struct LoadedAssets {
    pub(super) content_store: ContentStore,
    pub(super) font: Font,
    pub(super) main_background: TextureId,
    pub(super) sprites: Vec<SpriteData>,
    pub(super) baked_sprites: BakedSpriteTextures,
    pub(super) pattern_texture: TextureId,
}

pub(super) fn load(files: &FileStore, renderer: &mut Renderer) -> Result<LoadedAssets, String> {
    let palette_toml = files.read("palette.toml").map_err(|e| e.to_string())?;
    let palette =
        Rgb6Palette::from_toml_bytes("palette.toml", &palette_toml).map_err(|e| e.to_string())?;
    let content_store = ContentStore::load(files, CONTENT_MANIFEST).map_err(|e| e.to_string())?;
    let sprites = content_store.sprites.clone();
    let font = Font::from_sprites(&sprites);
    let main_background = load_background_texture(files, renderer)?;
    let palette = palette.into_palette();
    let baked_sprites = BakedSpriteTextures::bake(
        renderer,
        &palette,
        &sprites,
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
        sprites,
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

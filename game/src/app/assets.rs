use crate::content::ContentStore;
use crate::files::FileStore;
use crate::gfx::palette::Rgb6Palette;
use crate::gfx::png::load_png;
use engine::oxide::Font;
use engine::palette::Palette;
use engine::sprite::SpriteData;
use engine::video::{Renderer, TextureId};

const MAIN_PNG: &str = "MAIN.png";
const CONTENT_MANIFEST: &str = "content.toml";

pub(super) struct LoadedAssets {
    pub(super) content_store: ContentStore,
    pub(super) font: Font,
    pub(super) main_background: TextureId,
    pub(super) palette: Palette,
    pub(super) sprites: Vec<SpriteData>,
}

pub(super) fn load(files: &FileStore, renderer: &mut Renderer) -> Result<LoadedAssets, String> {
    let palette_toml = files.read("palette.toml").map_err(|e| e.to_string())?;
    let palette =
        Rgb6Palette::from_toml_bytes("palette.toml", &palette_toml).map_err(|e| e.to_string())?;
    let content_store = ContentStore::load(files, CONTENT_MANIFEST).map_err(|e| e.to_string())?;
    let sprites = content_store.sprites.clone();
    let font = Font::from_sprites(&sprites);
    let main_background = load_background_texture(files, renderer)?;

    Ok(LoadedAssets {
        content_store,
        font,
        main_background,
        palette: palette.into_palette(),
        sprites,
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

use engine::atlas::{Atlas, AtlasRegion};
use engine::video::Renderer;
use serde::Deserialize;

use crate::error::AssetError;
use crate::files::FileStore;
use crate::gfx::png::load_png;

#[derive(Debug, Deserialize)]
struct AtlasManifest {
    format_version: u32,
    image: String,
    sprites: Vec<SpriteEntry>,
}

#[derive(Debug, Deserialize)]
struct SpriteEntry {
    index: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    center_x: i8,
    center_y: i8,
}

pub(crate) fn load_sprite_atlas(
    files: &FileStore,
    renderer: &mut Renderer,
    manifest_path: &str,
) -> Result<Atlas, AssetError> {
    let data = files
        .read(manifest_path)
        .map_err(|e| AssetError::io(manifest_path, e))?;
    let text = std::str::from_utf8(&data).map_err(|e| AssetError::utf8(manifest_path, e))?;
    let manifest: AtlasManifest =
        toml::from_str(text).map_err(|e| AssetError::toml(manifest_path, e))?;

    if manifest.format_version != 2 {
        return Err(AssetError::format_version(
            manifest_path,
            2,
            manifest.format_version,
        ));
    }

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };
    let image_path = format!("{base_dir}{}", manifest.image);

    let png_data = files
        .read(&image_path)
        .map_err(|e| AssetError::io(&image_path, e))?;
    let img = load_png(&png_data)?;
    let texture_id = renderer
        .create_rgba_texture(&img.pixels, img.width, img.height)
        .map_err(|e| AssetError::Custom(format!("Failed to create texture: {e}")))?;

    let mut regions: Vec<AtlasRegion> = Vec::with_capacity(manifest.sprites.len());

    for entry in &manifest.sprites {
        let expected_index = regions.len();
        if entry.index != expected_index {
            return Err(AssetError::Custom(format!(
                "Atlas sprite index mismatch in {manifest_path}: expected {expected_index}, got {}",
                entry.index
            )));
        }
        regions.push(AtlasRegion {
            x: entry.x,
            y: entry.y,
            width: entry.width,
            height: entry.height,
            center_x: entry.center_x,
            center_y: entry.center_y,
        });
    }

    Ok(Atlas {
        texture_id,
        regions,
    })
}

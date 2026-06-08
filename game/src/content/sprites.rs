use crate::error::AssetError;
use crate::files::FileStore;
use engine::sprite::SpriteData;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct SpriteManifest {
    format_version: u32,
    default: String,
    sets: Vec<SpriteSetEntry>,
}

#[derive(Debug, Deserialize)]
struct SpriteSetEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct SpriteSetToml {
    id: String,
    name: String,
    sprites: Vec<SpriteToml>,
}

#[derive(Debug, Deserialize)]
struct SpriteToml {
    index: usize,
    width: u16,
    height: u16,
    center_x: i8,
    center_y: i8,
    pixels: String,
}

pub(crate) fn load_sprites(
    files: &FileStore,
    manifest_path: &str,
) -> Result<Vec<SpriteData>, AssetError> {
    let manifest: SpriteManifest = super::read_toml(files, manifest_path)?;

    if manifest.format_version != 1 {
        return Err(AssetError::format_version(
            manifest_path,
            1,
            manifest.format_version,
        ));
    }
    if manifest.sets.is_empty() {
        return Err(AssetError::Custom(
            "Sprite manifest has no sets".to_string(),
        ));
    }

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut all_sprites: Vec<SpriteData> = Vec::new();
    let mut seen_set_ids: HashSet<String> = HashSet::new();

    for entry in &manifest.sets {
        let full_path = format!("{base_dir}{}", entry.file);
        let set: SpriteSetToml = super::read_toml(files, &full_path)?;

        if set.id != entry.id {
            return Err(AssetError::Custom(format!(
                "Sprite set id mismatch in {full_path}: manifest has '{}', file has '{}'",
                entry.id, set.id
            )));
        }
        if set.name.is_empty() {
            return Err(AssetError::Custom(format!(
                "Sprite set '{}' has empty name in {full_path}",
                entry.id
            )));
        }
        if set.sprites.is_empty() {
            return Err(AssetError::Custom(format!(
                "Sprite set '{}' has no sprites in {full_path}",
                entry.id
            )));
        }

        if !seen_set_ids.insert(set.id.clone()) {
            return Err(AssetError::Custom(format!(
                "Duplicate sprite set id '{}'",
                entry.id
            )));
        }

        let start_index = all_sprites.len();

        for (i, s) in set.sprites.iter().enumerate() {
            if s.width == 0 {
                return Err(AssetError::Custom(format!(
                    "Sprite {} in {full_path} has zero width",
                    s.index
                )));
            }
            if s.height == 0 {
                return Err(AssetError::Custom(format!(
                    "Sprite {} in {full_path} has zero height",
                    s.index
                )));
            }

            let expected_byte_count = s.width as usize * s.height as usize;
            let hex_clean: String = s.pixels.chars().filter(|c| !c.is_whitespace()).collect();
            if !hex_clean.len().is_multiple_of(2) {
                return Err(AssetError::Custom(format!(
                    "Sprite {} in {full_path} has odd number of hex digits ({})",
                    s.index,
                    hex_clean.len()
                )));
            }
            let decoded: Vec<u8> = (0..hex_clean.len())
                .step_by(2)
                .map(|j| {
                    u8::from_str_radix(&hex_clean[j..j + 2], 16).map_err(|_| {
                        AssetError::Custom(format!(
                            "Sprite {} in {full_path} has invalid hex at position {}",
                            s.index, j
                        ))
                    })
                })
                .collect::<Result<Vec<_>, AssetError>>()?;

            if decoded.len() != expected_byte_count {
                return Err(AssetError::Custom(format!(
                    "Sprite {} in {full_path}: decoded {} bytes but width*height = {}",
                    s.index,
                    decoded.len(),
                    expected_byte_count
                )));
            }

            let expected_index = start_index + i;
            if s.index != expected_index {
                return Err(AssetError::Custom(format!(
                    "Sprite index mismatch in {full_path}: expected {expected_index}, got {}",
                    s.index
                )));
            }

            all_sprites.push(SpriteData {
                data: decoded,
                rgba_data: Vec::new(),
                width: s.width,
                height: s.height,
                center_x: s.center_x,
                center_y: s.center_y,
            });
        }
    }

    if !seen_set_ids.contains(&manifest.default) {
        return Err(AssetError::Custom(format!(
            "Default sprite set '{}' not found in manifest",
            manifest.default
        )));
    }

    Ok(all_sprites)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::files::FileStore;

    fn minimal_sprite_hex(w: u16, h: u16) -> String {
        "00".repeat(w as usize * h as usize)
    }

    #[test]
    fn rejects_invalid_hex() {
        let (store, dir) = make_files();
        write(
            &dir,
            "sprites/manifest.toml",
            r#"
format_version = 1
default = "default"

[[sets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "sprites/default.toml",
            r#"
id = "default"
name = "Default"

[[sprites]]
index = 0
width = 2
height = 2
center_x = 0
center_y = 0
pixels = """
XX
"""
"#,
        );
        assert!(load_sprites(&store, "sprites/manifest.toml").is_err());
    }

    #[test]
    fn rejects_odd_hex_digits() {
        let (store, dir) = make_files();
        write(
            &dir,
            "sprites/manifest.toml",
            r#"
format_version = 1
default = "default"

[[sets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "sprites/default.toml",
            r#"
id = "default"
name = "Default"

[[sprites]]
index = 0
width = 2
height = 2
center_x = 0
center_y = 0
pixels = """
a
"""
"#,
        );
        assert!(load_sprites(&store, "sprites/manifest.toml").is_err());
    }

    #[test]
    fn rejects_width_height_zero() {
        let (store, dir) = make_files();
        write(
            &dir,
            "sprites/manifest.toml",
            r#"
format_version = 1
default = "default"

[[sets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "sprites/default.toml",
            r#"
id = "default"
name = "Default"

[[sprites]]
index = 0
width = 0
height = 2
center_x = 0
center_y = 0
pixels = ""
"#,
        );
        assert!(load_sprites(&store, "sprites/manifest.toml").is_err());
    }

    #[test]
    fn rejects_missing_default_set() {
        let (store, dir) = make_files();
        write(
            &dir,
            "sprites/manifest.toml",
            r#"
format_version = 1
default = "nonexistent"

[[sets]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "sprites/a.toml",
            &format!(
                r#"
id = "a"
name = "A"
[[sprites]]
index = 0
width = 1
height = 1
center_x = 0
center_y = 0
pixels = """
{}
"""
"#,
                minimal_sprite_hex(1, 1)
            ),
        );
        assert!(load_sprites(&store, "sprites/manifest.toml").is_err());
    }

    #[test]
    fn rejects_non_sequential_index() {
        let (store, dir) = make_files();
        write(
            &dir,
            "sprites/manifest.toml",
            r#"
format_version = 1
default = "default"

[[sets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "sprites/default.toml",
            r#"
id = "default"
name = "Default"

[[sprites]]
index = 0
width = 1
height = 1
center_x = 0
center_y = 0
pixels = "00"

[[sprites]]
index = 2
width = 1
height = 1
center_x = 0
center_y = 0
pixels = "00"
"#,
        );
        assert!(load_sprites(&store, "sprites/manifest.toml").is_err());
    }

    #[test]
    fn rejects_pixel_count_mismatch() {
        let (store, dir) = make_files();
        write(
            &dir,
            "sprites/manifest.toml",
            r#"
format_version = 1
default = "default"

[[sets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "sprites/default.toml",
            r#"
id = "default"
name = "Default"

[[sprites]]
index = 0
width = 2
height = 2
center_x = 0
center_y = 0
pixels = "0000"
"#,
        );
        assert!(load_sprites(&store, "sprites/manifest.toml").is_err());
    }

    #[test]
    fn loads_real_assets() {
        let store = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let sprites = load_sprites(&store, "sprites/manifest.toml").unwrap();
        assert_eq!(sprites.len(), 177);

        let logo = &sprites[61];
        assert_eq!(logo.width, 17);
        assert_eq!(logo.height, 15);

        let derived = &sprites[83];
        assert_eq!(derived.width, 24);
        assert_eq!(derived.height, 3);

        let last = &sprites[176];
        assert_eq!(last.width, 7);
        assert_eq!(last.height, 11);
        assert_eq!(last.center_x, 1);
        assert_eq!(last.center_y, 10);
    }

    #[test]
    fn jumper_sprite_indices_match_remap_sources() {
        use crate::gfx::palette::{
            JUMPER_SKI_SOURCE, JUMPER_SUIT_SOURCE_SHADE_1, JUMPER_SUIT_SOURCE_SHADE_3,
        };
        let store = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let sprites = load_sprites(&store, "sprites/manifest.toml").unwrap();

        let mut body_indices = HashSet::new();
        let mut ski_indices = HashSet::new();
        let mut all_indices: HashSet<u8> = HashSet::new();

        for (i, s) in sprites.iter().enumerate() {
            let non_zero: HashSet<_> = s.data.iter().copied().filter(|&p| p != 0).collect();
            all_indices.extend(&non_zero);
            match i {
                70..=89 => ski_indices.extend(non_zero),
                100..=164 => body_indices.extend(non_zero),
                _ => {}
            }
        }

        assert!(
            body_indices.contains(&JUMPER_SUIT_SOURCE_SHADE_1),
            "body sprites must use source shade 1 ({JUMPER_SUIT_SOURCE_SHADE_1})"
        );
        assert!(
            body_indices.contains(&JUMPER_SUIT_SOURCE_SHADE_3),
            "body sprites must use source shade 3 ({JUMPER_SUIT_SOURCE_SHADE_3})"
        );

        assert!(
            ski_indices.contains(&JUMPER_SKI_SOURCE),
            "ski sprites must use source ski index ({JUMPER_SKI_SOURCE})"
        );

        for idx in 249..=254u8 {
            assert!(
                !body_indices.contains(&idx),
                "body sprites use mutable index {idx} — stale cached RGBA risk"
            );
            assert!(
                !ski_indices.contains(&idx),
                "ski sprites use mutable index {idx} — stale cached RGBA risk"
            );
        }

        assert!(
            !body_indices.contains(&215u8),
            "body sprites should not use index 215"
        );
        assert!(
            !body_indices.contains(&217u8),
            "body sprites should not use index 217"
        );
    }
}

pub(crate) mod hills;
pub(crate) mod languages;
mod manifest;
pub(crate) mod names;
pub(crate) mod sprites;
#[cfg(test)]
pub(crate) mod test_support;

use crate::data::hill::HillCatalog;
use crate::error::AssetError;
use crate::files::FileStore;
use crate::text::lang::LangBase;
use engine::sprite::SpriteData;

/// Read a TOML file and deserialize.
pub(crate) fn read_toml<T>(files: &FileStore, path: &str) -> Result<T, AssetError>
where
    T: serde::de::DeserializeOwned,
{
    let data = files.read(path).map_err(|e| AssetError::io(path, e))?;
    let text = std::str::from_utf8(&data).map_err(|e| AssetError::utf8(path, e))?;
    toml::from_str(text).map_err(|e| AssetError::toml(path, e))
}

#[derive(Debug)]
pub struct ContentStore {
    pub langbase: LangBase,
    pub namesets: names::NameCatalog,
    pub hills: HillCatalog,
    pub sprites: Vec<SpriteData>,
}

impl ContentStore {
    pub fn load(files: &FileStore, content_manifest_path: &str) -> Result<Self, AssetError> {
        let cm = manifest::ContentManifest::load(files, content_manifest_path)?;

        let langbase_manifest = match cm.languages {
            Some(ref l) => &l.manifest,
            None => {
                return Err(AssetError::Custom(
                    "content.toml missing [languages] section".to_string(),
                ))
            }
        };
        let langbase = languages::load_languages(files, langbase_manifest)?;

        let namesets_manifest = match cm.namesets {
            Some(ref n) => &n.manifest,
            None => {
                return Err(AssetError::Custom(
                    "content.toml missing [namesets] section".to_string(),
                ))
            }
        };
        let namesets = names::load_namesets(files, namesets_manifest)?;

        let hills_manifest = match cm.hills {
            Some(ref h) => &h.manifest,
            None => {
                return Err(AssetError::Custom(
                    "content.toml missing [hills] section".to_string(),
                ))
            }
        };
        let hills = hills::load_hills(files, hills_manifest)?;

        let sprites_manifest = match cm.sprites {
            Some(ref s) => &s.manifest,
            None => {
                return Err(AssetError::Custom(
                    "content.toml missing [sprites] section".to_string(),
                ))
            }
        };
        let sprites = sprites::load_sprites(files, sprites_manifest)?;

        Ok(Self {
            langbase,
            namesets,
            hills,
            sprites,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    fn write_sprites(dir: &tempfile::TempDir) {
        write(
            dir,
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
            dir,
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
"#,
        );
    }

    #[test]
    fn loads_content_through_content_toml() {
        let (store, dir) = make_files();
        write(
            &dir,
            "content.toml",
            r#"format_version = 1

[languages]
manifest = "languages/manifest.toml"

[namesets]
manifest = "namesets/manifest.toml"

[hills]
manifest = "hills/manifest.toml"

[sprites]
manifest = "sprites/manifest.toml"
"#,
        );
        write_sprites(&dir);
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "english.toml"
"#,
        );
        write(
            &dir,
            "languages/english.toml",
            r#"
id = "english"
name = "English"

[strings]
6 = "Yes"
7 = "No"
"#,
        );
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "namesets/default.toml",
            r#"
id = "default"
name = "Default"
title = "Default Names"
names = ["Alice", "Bob"]
"#,
        );
        write(
            &dir,
            "hills/manifest.toml",
            r#"
format_version = 1
default = "default"

[[catalogs]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "hills/default.toml",
            r#"
id = "default"
name = "Default"
[[hills]]
id = "H"
name = "testhill"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "test"
checksum = 0
profile_checksum = 0
"#,
        );

        ContentStore::load(&store, "content.toml").unwrap();
    }

    #[test]
    fn content_toml_rejects_missing_namesets() {
        let (store, dir) = make_files();
        write(
            &dir,
            "content.toml",
            r#"format_version = 1

[languages]
manifest = "languages/manifest.toml"

[hills]
manifest = "hills/manifest.toml"

[sprites]
manifest = "sprites/manifest.toml"
"#,
        );
        write_sprites(&dir);
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "english.toml"
"#,
        );
        write(
            &dir,
            "languages/english.toml",
            r#"
id = "english"
name = "English"

[strings]
6 = "Yes"
"#,
        );
        write(
            &dir,
            "hills/manifest.toml",
            r#"
format_version = 1
default = "default"

[[catalogs]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "hills/default.toml",
            r#"
id = "default"
name = "Default"
[[hills]]
id = "H"
name = "t"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "t"
checksum = 0
profile_checksum = 0
"#,
        );

        let result = ContentStore::load(&store, "content.toml");
        assert!(result.is_err(), "expected error for missing [namesets]");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("missing [namesets]"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn content_toml_rejects_missing_hills() {
        let (store, dir) = make_files();
        write(
            &dir,
            "content.toml",
            r#"format_version = 1

[languages]
manifest = "languages/manifest.toml"

[namesets]
manifest = "namesets/manifest.toml"

[sprites]
manifest = "sprites/manifest.toml"
"#,
        );
        write_sprites(&dir);
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "english.toml"
"#,
        );
        write(
            &dir,
            "languages/english.toml",
            r#"
id = "english"
name = "English"

[strings]
6 = "Yes"
"#,
        );
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "namesets/default.toml",
            r#"
id = "default"
name = "Default"
title = "Default"
names = ["A"]
"#,
        );

        let result = ContentStore::load(&store, "content.toml");
        assert!(result.is_err(), "expected error for missing [hills]");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing [hills]"), "unexpected error: {err}");
    }

    #[test]
    fn content_toml_rejects_missing_sprites() {
        let (store, dir) = make_files();
        write(
            &dir,
            "content.toml",
            r#"format_version = 1

[languages]
manifest = "languages/manifest.toml"

[namesets]
manifest = "namesets/manifest.toml"

[hills]
manifest = "hills/manifest.toml"
"#,
        );
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "english.toml"
"#,
        );
        write(
            &dir,
            "languages/english.toml",
            r#"
id = "english"
name = "English"

[strings]
6 = "Yes"
"#,
        );
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "namesets/default.toml",
            r#"
id = "default"
name = "Default"
title = "Default"
names = ["A"]
"#,
        );
        write(
            &dir,
            "hills/manifest.toml",
            r#"
format_version = 1
default = "default"

[[catalogs]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "hills/default.toml",
            r#"
id = "default"
name = "Default"
[[hills]]
id = "H"
name = "t"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "t"
checksum = 0
profile_checksum = 0
"#,
        );

        let result = ContentStore::load(&store, "content.toml");
        assert!(result.is_err(), "expected error for missing [sprites]");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing [sprites]"), "unexpected error: {err}");
    }
}

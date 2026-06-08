use crate::error::AssetError;
use crate::files::FileStore;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ContentManifest {
    pub(crate) format_version: u32,
    pub(crate) languages: Option<ContentSection>,
    pub(crate) namesets: Option<ContentSection>,
    pub(crate) hills: Option<ContentSection>,
    pub(crate) sprites: Option<ContentSection>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContentSection {
    pub(crate) manifest: String,
}

impl ContentManifest {
    pub(crate) fn load(files: &FileStore, path: &str) -> Result<Self, AssetError> {
        let data = files.read(path).map_err(|e| AssetError::io(path, e))?;
        let text = std::str::from_utf8(&data).map_err(|e| AssetError::utf8(path, e))?;
        let manifest: ContentManifest =
            toml::from_str(text).map_err(|e| AssetError::toml(path, e))?;
        if manifest.format_version != 1 {
            return Err(AssetError::format_version(path, 1, manifest.format_version));
        }
        Ok(manifest)
    }
}

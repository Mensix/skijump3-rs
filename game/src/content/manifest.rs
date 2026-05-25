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
    pub(crate) fn load(files: &crate::files::FileStore, path: &str) -> Result<Self, String> {
        let data = files
            .read(path)
            .map_err(|e| format!("Failed to read {path}: {e}"))?;
        let text =
            std::str::from_utf8(&data).map_err(|e| format!("{path} is not valid UTF-8: {e}"))?;
        let manifest: ContentManifest =
            toml::from_str(text).map_err(|e| format!("Failed to parse {path}: {e}"))?;
        if manifest.format_version != 1 {
            return Err(format!(
                "Unsupported content manifest format_version: {}",
                manifest.format_version
            ));
        }
        Ok(manifest)
    }
}

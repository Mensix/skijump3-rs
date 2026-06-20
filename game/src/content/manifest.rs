use crate::files::FileStore;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ContentManifest {
    pub(crate) format_version: u32,
    pub(crate) languages: Option<ContentSection>,
    pub(crate) namesets: Option<ContentSection>,
    pub(crate) hills: Option<ContentSection>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContentSection {
    pub(crate) manifest: String,
}

impl ContentManifest {
    pub(crate) fn load(files: &FileStore, path: &str) -> Self {
        super::read_toml(files, path)
    }
}

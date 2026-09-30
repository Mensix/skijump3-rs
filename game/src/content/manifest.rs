use crate::files::FileStore;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ContentManifest {
    pub(crate) languages: Option<ContentSection>,
    pub(crate) namesets: Option<ContentSection>,
    pub(crate) hills: Option<ContentSection>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContentSection {
    pub(crate) manifest: String,
}

impl ContentManifest {
    pub(crate) fn load(files: &FileStore, path: &str) -> Result<Self, String> {
        super::read_toml(files, path)
    }
}

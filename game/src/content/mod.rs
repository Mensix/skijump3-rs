pub(crate) mod hills;
pub(crate) mod languages;
mod manifest;
pub(crate) mod names;
#[cfg(test)]
pub(crate) mod test_support;

use crate::data::hill::HillCatalog;
use crate::files::FileStore;
use crate::text::lang::LangBase;

pub(crate) fn read_toml<T>(files: &FileStore, path: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let data = files.read(path);
    let text = std::str::from_utf8(&data).map_err(|error| format!("{path}: {error}"))?;
    toml::from_str(text).map_err(|error| format!("{path}: {error}"))
}

#[derive(Debug, Default)]
pub struct ContentStore {
    pub langbase: LangBase,
    pub namesets: names::NameCatalog,
    pub hills: HillCatalog,
}

impl ContentStore {
    pub fn load(files: &FileStore) -> Result<Self, String> {
        let cm = manifest::ContentManifest::load(files, "content.toml")?;

        let languages = cm
            .languages
            .as_ref()
            .ok_or_else(|| "content manifest has no languages section".to_string())?;
        let langbase_manifest = &languages.manifest;
        let langbase = languages::load_languages(files, langbase_manifest)?;

        let namesets_section = cm
            .namesets
            .as_ref()
            .ok_or_else(|| "content manifest has no namesets section".to_string())?;
        let namesets_manifest = &namesets_section.manifest;
        let namesets = names::load_namesets(files, namesets_manifest)?;

        let hills_section = cm
            .hills
            .as_ref()
            .ok_or_else(|| "content manifest has no hills section".to_string())?;
        let hills_manifest = &hills_section.manifest;
        let hills = hills::load_hills(files, hills_manifest)?;

        Ok(Self {
            langbase,
            namesets,
            hills,
        })
    }
}

pub(crate) mod hills;
pub(crate) mod languages;
mod manifest;
pub(crate) mod names;
#[cfg(test)]
pub(crate) mod test_support;

use crate::data::hill::HillCatalog;
use crate::files::FileStore;
use crate::text::lang::LangBase;

pub(crate) fn read_toml<T>(files: &FileStore, path: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    let data = files.read(path);
    let text = std::str::from_utf8(&data).unwrap();
    toml::from_str(text).unwrap()
}

#[derive(Debug)]
pub struct ContentStore {
    pub langbase: LangBase,
    pub namesets: names::NameCatalog,
    pub hills: HillCatalog,
}

impl ContentStore {
    pub fn load(files: &FileStore, content_manifest_path: &str) -> Self {
        let cm = manifest::ContentManifest::load(files, content_manifest_path);

        let langbase_manifest = &cm.languages.as_ref().unwrap().manifest;
        let langbase = languages::load_languages(files, langbase_manifest);

        let namesets_manifest = &cm.namesets.as_ref().unwrap().manifest;
        let namesets = names::load_namesets(files, namesets_manifest);

        let hills_manifest = &cm.hills.as_ref().unwrap().manifest;
        let hills = hills::load_hills(files, hills_manifest);

        Self {
            langbase,
            namesets,
            hills,
        }
    }
}

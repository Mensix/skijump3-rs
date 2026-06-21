use crate::files::FileStore;
use crate::text::lang::{LangBase, Language};
use serde::Deserialize;
use std::collections::BTreeMap;

const NUM_STR: usize = 599;

#[derive(Debug, Deserialize)]
struct LanguageManifest {
    default: Option<String>,
    languages: Vec<LanguageEntry>,
}

#[derive(Debug, Deserialize)]
struct LanguageEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct LanguageToml {
    id: String,
    name: String,
    strings: BTreeMap<String, String>,
}

pub(crate) fn load_languages(
    files: &FileStore,
    manifest_path: &str,
) -> LangBase {
    let manifest: LanguageManifest = super::read_toml(files, manifest_path);

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut info: Vec<Language> = Vec::new();
    let mut all_strings: Vec<Vec<String>> = Vec::new();

    for entry in &manifest.languages {
        let full_path = format!("{base_dir}{}", entry.file);
        let lang_toml: LanguageToml = super::read_toml(files, &full_path);

        info.push(Language {
            id: lang_toml.id,
            name: lang_toml.name,
        });
        let mut strs = vec!["?".to_string(); NUM_STR];
        for (key, value) in &lang_toml.strings {
            if let Ok(idx) = key.parse::<usize>() {
                if idx < NUM_STR {
                    strs[idx] = value.clone();
                }
            }
        }
        all_strings.push(strs);
    }

    let fallback_lang = manifest
        .default
        .and_then(|default_id| {
            info.iter().position(|li| li.id == default_id)
        })
        .unwrap_or(0);

    LangBase::new(all_strings, info, fallback_lang)
}

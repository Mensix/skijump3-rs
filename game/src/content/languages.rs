use crate::files::FileStore;
use crate::text::lang::LangBase;
use serde::Deserialize;
use std::collections::BTreeMap;

const NUM_STR: usize = 599;

#[derive(Debug, Deserialize)]

struct LanguageManifest {
    languages: Vec<LanguageEntry>,
}

#[derive(Debug, Deserialize)]

struct LanguageEntry {
    file: String,
}

#[derive(Debug, Deserialize)]

struct LanguageToml {
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

    let mut language_names: Vec<String> = Vec::new();
    let mut all_strings: Vec<Vec<String>> = Vec::new();

    for entry in &manifest.languages {
        let full_path = format!("{base_dir}{}", entry.file);
        let lang_toml: LanguageToml = super::read_toml(files, &full_path);

        language_names.push(lang_toml.name);
        let mut strs = vec![String::new(); NUM_STR];
        for (key, value) in &lang_toml.strings {
            if let Ok(idx) = key.parse::<usize>() {
                if idx < NUM_STR {
                    strs[idx] = value.clone();
                }
            }
        }
        all_strings.push(strs);
    }

    LangBase::new(all_strings, language_names)
}

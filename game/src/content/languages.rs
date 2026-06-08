use crate::error::AssetError;
use crate::files::FileStore;
use crate::text::lang::LangBase;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

const NUM_STR: usize = 599;

#[derive(Debug, Deserialize)]
struct LanguageManifest {
    format_version: u32,
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
) -> Result<LangBase, AssetError> {
    let manifest: LanguageManifest = super::read_toml(files, manifest_path)?;

    if manifest.format_version != 1 {
        return Err(AssetError::format_version(
            manifest_path,
            1,
            manifest.format_version,
        ));
    }
    if manifest.languages.is_empty() {
        return Err(AssetError::Custom("Manifest has no languages".to_string()));
    }

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut language_names: Vec<String> = Vec::new();
    let mut all_strings: Vec<Vec<String>> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for entry in &manifest.languages {
        let full_path = format!("{base_dir}{}", entry.file);
        let lang_toml: LanguageToml = super::read_toml(files, &full_path)?;

        if lang_toml.id != entry.id {
            return Err(AssetError::Custom(format!(
                "Language id mismatch in {full_path}: manifest has '{}', file has '{}'",
                entry.id, lang_toml.id
            )));
        }
        if lang_toml.name.is_empty() {
            return Err(AssetError::Custom(format!(
                "Language '{}' has empty name in {full_path}",
                entry.id
            )));
        }
        if lang_toml.strings.is_empty() {
            return Err(AssetError::Custom(format!(
                "Language '{}' has no strings in {full_path}",
                entry.id
            )));
        }

        if !seen_ids.insert(lang_toml.id.clone()) {
            return Err(AssetError::Custom(format!(
                "Duplicate language id '{}'",
                entry.id
            )));
        }

        let mut strings: Vec<String> = (0..=NUM_STR).map(|_| "?".to_string()).collect();
        for (key, val) in &lang_toml.strings {
            let idx: usize = key.parse().map_err(|_| {
                AssetError::Custom(format!("Invalid string key '{key}' in {full_path}"))
            })?;
            if idx <= NUM_STR {
                strings[idx] = val.clone();
            }
        }

        language_names.push(lang_toml.name.clone());
        all_strings.push(strings);
    }

    // Build display-name mapping (language id → translated name strings)
    let mut display_names: Vec<Vec<String>> = Vec::new();
    for i in 0..all_strings.len() {
        let mut dn = Vec::new();
        for j in 0..all_strings.len() {
            dn.push(if j < all_strings[i].len() && i < all_strings.len() {
                all_strings[j][i].clone()
            } else {
                language_names[j].clone()
            });
        }
        display_names.push(dn);
    }

    Ok(LangBase::new(all_strings, language_names))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::files::FileStore;

    fn write_minimal_language(dir: &tempfile::TempDir, file: &str, id: &str, strings: &str) {
        write(
            dir,
            file,
            &format!(
                r#"
id = "{id}"
name = "{id}"
[strings]
{strings}
"#
            ),
        );
    }

    #[test]
    fn rejects_missing_language_file() {
        let (store, dir) = make_files();
        write(
            &dir,
            "lang/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "nonexistent.toml"
"#,
        );
        assert!(load_languages(&store, "lang/manifest.toml").is_err());
    }

    #[test]
    fn rejects_invalid_string_key() {
        let (store, dir) = make_files();
        write(
            &dir,
            "lang/manifest.toml",
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
            "lang/english.toml",
            r#"
id = "english"
name = "English"
[strings]
abc = "bad"
"#,
        );
        let result = load_languages(&store, "lang/manifest.toml");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid string key"));
    }

    #[test]
    fn rejects_duplicate_language_id() {
        let (store, dir) = make_files();
        write(
            &dir,
            "lang/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "english.toml"

[[languages]]
id = "english"
file = "duplicate.toml"
"#,
        );
        write_minimal_language(&dir, "lang/english.toml", "english", r#"6 = "Yes""#);
        write_minimal_language(&dir, "lang/duplicate.toml", "english", r#"6 = "No""#);
        let result = load_languages(&store, "lang/manifest.toml");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Duplicate language id"));
    }

    #[test]
    fn rejects_id_mismatch() {
        let (store, dir) = make_files();
        write(
            &dir,
            "lang/manifest.toml",
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
            "lang/english.toml",
            r#"
id = "finnish"
name = "Finnish"
[strings]
6 = "Kyllä"
"#,
        );
        let result = load_languages(&store, "lang/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }

    #[test]
    fn rejects_wrong_format_version() {
        let (store, dir) = make_files();
        write(
            &dir,
            "lang/manifest.toml",
            r#"
format_version = 999
default = "english"

[[languages]]
id = "english"
file = "english.toml"
"#,
        );
        write_minimal_language(&dir, "lang/english.toml", "english", r#"6 = "Yes""#);

        let result = load_languages(&store, "lang/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }

    #[test]
    fn loads_real_assets() {
        let store = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let lang = load_languages(&store, "languages/manifest.toml").unwrap();
        assert!(!lang.languages.is_empty());
        assert_eq!(lang.lstr(1), "One");
    }
}

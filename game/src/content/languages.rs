use crate::parsers::langbase::LangBase;
use crate::save::files::FileStore;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

const NUM_STR: usize = 599;

#[derive(Debug, Deserialize)]
struct LanguageManifest {
    format_version: u32,
    default: String,
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

pub(crate) fn load_languages(files: &FileStore, manifest_path: &str) -> Result<LangBase, String> {
    let data = files
        .read(manifest_path)
        .map_err(|e| format!("Failed to read {manifest_path}: {e}"))?;
    let text =
        std::str::from_utf8(&data).map_err(|e| format!("Manifest is not valid UTF-8: {e}"))?;
    let manifest: LanguageManifest =
        toml::from_str(text).map_err(|e| format!("Failed to parse manifest: {e}"))?;

    if manifest.format_version != 1 {
        return Err(format!(
            "Unsupported manifest format_version: {}",
            manifest.format_version
        ));
    }
    if manifest.languages.is_empty() {
        return Err("Manifest has no languages".to_string());
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
        let data = files
            .read(&full_path)
            .map_err(|e| format!("Failed to read {full_path}: {e}"))?;
        let text = std::str::from_utf8(&data)
            .map_err(|e| format!("{full_path} is not valid UTF-8: {e}"))?;
        let lang_toml: LanguageToml =
            toml::from_str(text).map_err(|e| format!("Failed to parse {full_path}: {e}"))?;

        if lang_toml.id != entry.id {
            return Err(format!(
                "Language id mismatch in {full_path}: manifest has '{}', file has '{}'",
                entry.id, lang_toml.id
            ));
        }
        if lang_toml.name.is_empty() {
            return Err(format!(
                "Language '{}' has empty name in {full_path}",
                entry.id
            ));
        }
        if lang_toml.strings.is_empty() {
            return Err(format!(
                "Language '{}' has no strings in {full_path}",
                entry.id
            ));
        }

        if !seen_ids.insert(lang_toml.id.clone()) {
            return Err(format!("Duplicate language id '{}'", entry.id));
        }

        let mut strings: Vec<String> = (0..=NUM_STR).map(|_| "?".to_string()).collect();
        for (key, val) in &lang_toml.strings {
            let idx: usize = key
                .parse()
                .map_err(|_| format!("Invalid string key '{key}' in {full_path}"))?;
            if idx <= NUM_STR {
                strings[idx] = val.clone();
            }
        }

        language_names.push(lang_toml.name.clone());
        all_strings.push(strings);
    }

    if !seen_ids.contains(&manifest.default) {
        return Err(format!(
            "Default language '{}' not found in manifest",
            manifest.default
        ));
    }

    Ok(LangBase::new(all_strings, language_names))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::files::FileStore;
    use std::fs;
    use tempfile::tempdir;

    fn make_files() -> (FileStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf(), dir.path().to_path_buf());
        (store, dir)
    }

    fn write(dir: &tempfile::TempDir, path: &str, content: &str) {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
    }

    #[test]
    fn loads_minimal_languages() {
        let (store, dir) = make_files();
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "english.toml"

[[languages]]
id = "suomi"
file = "suomi.toml"
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
            "languages/suomi.toml",
            r#"
id = "suomi"
name = "Suomi"

[strings]
6 = "Kyllä"
7 = "Ei"
"#,
        );

        let langbase = load_languages(&store, "languages/manifest.toml").unwrap();
        assert_eq!(langbase.languages, vec!["English", "Suomi"]);
        assert_eq!(langbase.lstr(6), "Yes");
        assert_eq!(langbase.lstr(7), "No");

        langbase.selected.set(1);
        assert_eq!(langbase.lstr(6), "Kyllä");
        assert_eq!(langbase.lstr(7), "Ei");
    }

    #[test]
    fn missing_string_returns_question_mark() {
        let (store, dir) = make_files();
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

        let langbase = load_languages(&store, "languages/manifest.toml").unwrap();
        assert_eq!(langbase.lstr(999), "?");
        assert_eq!(langbase.lstr(5), "?");
    }

    #[test]
    fn rejects_invalid_string_key() {
        let (store, dir) = make_files();
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
"abc" = "bad"
"#,
        );

        let result = load_languages(&store, "languages/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid string key"));
    }

    #[test]
    fn rejects_duplicate_language_id() {
        let (store, dir) = make_files();
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "english"

[[languages]]
id = "english"
file = "english.toml"

[[languages]]
id = "english"
file = "english2.toml"
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
            "languages/english2.toml",
            r#"
id = "english"
name = "English 2"

[strings]
6 = "Yep"
"#,
        );

        let result = load_languages(&store, "languages/manifest.toml");
        assert!(result.is_err(), "expected error for duplicate language id");
        assert!(result.unwrap_err().contains("Duplicate language id"));
    }

    #[test]
    fn rejects_missing_default() {
        let (store, dir) = make_files();
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "missing"

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

        let result = load_languages(&store, "languages/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn rejects_empty_language_list() {
        let (store, dir) = make_files();
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "english"
languages = []
"#,
        );

        let result = load_languages(&store, "languages/manifest.toml");
        assert!(result.is_err(), "expected error for empty languages");
        let err = result.unwrap_err();
        assert!(
            err.contains("no languages") || err.contains("empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_wrong_format_version() {
        let (store, dir) = make_files();
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 999
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

        let result = load_languages(&store, "languages/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("format_version"));
    }

    #[test]
    fn rejects_id_mismatch() {
        let (store, dir) = make_files();
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
id = "suomi"
name = "Suomi"

[strings]
6 = "Kyllä"
"#,
        );

        let result = load_languages(&store, "languages/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mismatch"));
    }

    #[test]
    fn loads_with_subdirectory_manifest() {
        let (store, dir) = make_files();
        write(
            &dir,
            "languages/manifest.toml",
            r#"
format_version = 1
default = "test"

[[languages]]
id = "test"
file = "test.toml"
"#,
        );
        write(
            &dir,
            "languages/test.toml",
            r#"
id = "test"
name = "Test Lang"

[strings]
1 = "One"
2 = "Two"
"#,
        );

        let langbase = load_languages(&store, "languages/manifest.toml").unwrap();
        assert_eq!(langbase.languages, vec!["Test Lang"]);
        assert_eq!(langbase.lstr(1), "One");
        assert_eq!(langbase.lstr(2), "Two");
    }

    #[test]
    fn loads_real_assets_smoke_test() {
        let asset_dir = std::path::PathBuf::from("game/assets");
        if !asset_dir.join("languages/manifest.toml").exists() {
            return;
        }
        let save_dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(asset_dir, save_dir.path().to_path_buf());
        let langbase = load_languages(&files, "languages/manifest.toml").unwrap();

        assert!(
            langbase.languages.len() >= 2,
            "expected at least 2 languages"
        );
        assert_eq!(langbase.languages[0], "English");
        assert_eq!(langbase.languages[1], "Suomi");

        assert_eq!(langbase.lstr(6), "Yes");
        assert_eq!(langbase.lstr(7), "No");
        assert_eq!(langbase.lstr(9), "None");
        assert_eq!(langbase.lstr(175), "Setup Menu");

        langbase.selected.set(1);
        assert_eq!(langbase.lstr(6), "Kyllä");
        assert_eq!(langbase.lstr(7), "Ei");
    }
}

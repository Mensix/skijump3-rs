pub(crate) mod languages;
mod manifest;

use crate::parsers::langbase::LangBase;
use crate::save::files::FileStore;

#[derive(Debug)]
pub struct ContentStore {
    pub langbase: LangBase,
}

impl ContentStore {
    pub fn load(files: &FileStore, content_manifest_path: &str) -> Result<Self, String> {
        let cm = manifest::ContentManifest::load(files, content_manifest_path)?;
        let langbase_manifest = match cm.languages {
            Some(ref l) => &l.manifest,
            None => return Err("content.toml missing [languages] section".to_string()),
        };
        let langbase = languages::load_languages(files, langbase_manifest)?;
        Ok(Self { langbase })
    }
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
    fn loads_content_through_content_toml() {
        let (store, dir) = make_files();
        write(
            &dir,
            "content.toml",
            r#"format_version = 1

[languages]
manifest = "languages/manifest.toml"
"#,
        );
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
7 = "No"
"#,
        );

        let store = ContentStore::load(&store, "content.toml").unwrap();
        assert_eq!(store.langbase.languages, vec!["English"]);
        assert_eq!(store.langbase.lstr(6), "Yes");
        assert_eq!(store.langbase.lstr(7), "No");
    }

    #[test]
    fn content_toml_rejects_missing_languages() {
        let (store, dir) = make_files();
        write(
            &dir,
            "content.toml",
            r#"format_version = 1
"#,
        );

        let result = ContentStore::load(&store, "content.toml");
        assert!(result.is_err(), "expected error for missing [languages]");
        let err = result.unwrap_err();
        assert!(
            err.contains("missing [languages]"),
            "unexpected error: {err}"
        );
    }
}

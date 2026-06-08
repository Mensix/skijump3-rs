use crate::error::AssetError;
use crate::files::FileStore;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct NameCatalog {
    namesets: Vec<NameSet>,
}

#[derive(Debug, Clone)]
pub(crate) struct NameSet {
    #[allow(dead_code)]
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) names: Vec<String>,
    #[allow(dead_code)]
    pub(crate) teams: Vec<NameTeam>,
}

#[derive(Debug, Clone)]
pub(crate) struct NameTeam {
    #[allow(dead_code)]
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) members: Vec<usize>,
}

impl NameCatalog {
    pub(crate) fn names_for_config(&self, namenumber: i32) -> &[String] {
        if namenumber >= 0 && (namenumber as usize) < self.namesets.len() {
            &self.namesets[namenumber as usize].names
        } else {
            &self.namesets[0].names
        }
    }

    pub(crate) fn title_for_config(&self, namenumber: i32) -> &str {
        if namenumber >= 0 && (namenumber as usize) < self.namesets.len() {
            &self.namesets[namenumber as usize].title
        } else {
            &self.namesets[0].title
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.namesets.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.namesets.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct NameSetManifest {
    format_version: u32,
    default: String,
    namesets: Vec<NameSetEntry>,
}

#[derive(Debug, Deserialize)]
struct NameSetEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct NameSetToml {
    id: String,
    name: String,
    title: String,
    names: Vec<String>,
    teams: Option<Vec<NameTeamToml>>,
}

#[derive(Debug, Deserialize)]
struct NameTeamToml {
    name: String,
    members: Vec<usize>,
}

pub(crate) fn load_namesets(
    files: &FileStore,
    manifest_path: &str,
) -> Result<NameCatalog, AssetError> {
    let data = files
        .read(manifest_path)
        .map_err(|e| AssetError::io(manifest_path, e))?;
    let text = std::str::from_utf8(&data).map_err(|e| AssetError::utf8(manifest_path, e))?;
    let manifest: NameSetManifest =
        toml::from_str(text).map_err(|e| AssetError::toml(manifest_path, e))?;

    if manifest.format_version != 1 {
        return Err(AssetError::format_version(
            manifest_path,
            1,
            manifest.format_version,
        ));
    }
    if manifest.namesets.is_empty() {
        return Err(AssetError::Custom(
            "Nameset manifest has no namesets".to_string(),
        ));
    }

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut namesets: Vec<NameSet> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for entry in &manifest.namesets {
        let full_path = format!("{base_dir}{}", entry.file);
        let data = files
            .read(&full_path)
            .map_err(|e| AssetError::io(&full_path, e))?;
        let text = std::str::from_utf8(&data).map_err(|e| AssetError::utf8(&full_path, e))?;
        let ns: NameSetToml = toml::from_str(text).map_err(|e| AssetError::toml(&full_path, e))?;

        if ns.id != entry.id {
            return Err(AssetError::Custom(format!(
                "Nameset id mismatch in {full_path}: manifest has '{}', file has '{}'",
                entry.id, ns.id
            )));
        }
        if ns.name.is_empty() {
            return Err(AssetError::Custom(format!(
                "Nameset '{}' has empty name in {full_path}",
                entry.id
            )));
        }
        if ns.names.is_empty() {
            return Err(AssetError::Custom(format!(
                "Nameset '{}' has no names in {full_path}",
                entry.id
            )));
        }
        if ns.title.is_empty() {
            return Err(AssetError::Custom(format!(
                "Nameset '{}' has empty display_name in {full_path}",
                entry.id
            )));
        }

        if !seen_ids.insert(ns.id.clone()) {
            return Err(AssetError::Custom(format!(
                "Duplicate nameset id '{}'",
                entry.id
            )));
        }

        let teams = ns
            .teams
            .unwrap_or_default()
            .into_iter()
            .map(|t| NameTeam {
                name: t.name,
                members: t.members,
            })
            .collect();

        namesets.push(NameSet {
            id: ns.id,
            title: ns.title,
            names: ns.names,
            teams,
        });
    }

    if !seen_ids.contains(&manifest.default) {
        return Err(AssetError::Custom(format!(
            "Default nameset '{}' not found",
            manifest.default
        )));
    }

    Ok(NameCatalog { namesets })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::FileStore;
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

    fn names_100(s: &str) -> String {
        let names: Vec<String> = (0..100).map(|i| format!("{s}_{i}")).collect();
        names.join("\", \"")
    }

    #[test]
    fn loads_minimal_nameset() {
        let (store, dir) = make_files();
        write(
            &dir,
            "names/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "names/default.toml",
            &format!(
                r#"
id = "default"
name = "male"
title = "Default Names"
names = ["{}"]
"#,
                names_100("A")
            ),
        );

        let catalog = load_namesets(&store, "names/manifest.toml").unwrap();
        assert_eq!(catalog.len(), 1);
        assert_eq!(
            catalog.names_for_config(0),
            &names_100("A").split("\", \"").collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_missing_default() {
        let (store, dir) = make_files();
        write(
            &dir,
            "names/manifest.toml",
            r#"
format_version = 1
default = "nonexistent"

[[namesets]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "names/a.toml",
            &format!(
                r#"
id = "a"
name = "male"
title = "A"
names = ["{}"]
"#,
                names_100("A")
            ),
        );
        assert!(load_namesets(&store, "names/manifest.toml").is_err());
    }

    #[test]
    fn rejects_id_mismatch() {
        let (store, dir) = make_files();
        write(
            &dir,
            "names/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "names/default.toml",
            &format!(
                r#"
id = "other"
name = "male"
title = "Other"
names = ["{}"]
"#,
                names_100("O")
            ),
        );
        assert!(load_namesets(&store, "names/manifest.toml").is_err());
    }

    #[test]
    fn accepts_any_name_type() {
        let (store, dir) = make_files();
        write(
            &dir,
            "names/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "names/default.toml",
            &format!(
                r#"
id = "default"
name = "unknown"
title = "Any Names"
names = ["{}"]
"#,
                names_100("U")
            ),
        );
        let catalog = load_namesets(&store, "names/manifest.toml").unwrap();
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog.names_for_config(0).len(), 100);
    }

    #[test]
    fn rejects_empty_names() {
        let (store, dir) = make_files();
        write(
            &dir,
            "names/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "names/default.toml",
            r#"
id = "default"
name = "male"
title = "Empty Names"
names = []
"#,
        );
        let result = load_namesets(&store, "names/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no names"));
    }

    #[test]
    fn rejects_duplicate_nameset_id() {
        let (store, dir) = make_files();
        write(
            &dir,
            "names/manifest.toml",
            r#"
format_version = 1
default = "default"

[[namesets]]
id = "default"
file = "default.toml"

[[namesets]]
id = "default"
file = "other.toml"
"#,
        );
        write(
            &dir,
            "names/default.toml",
            &format!(
                r#"
id = "default"
name = "male"
title = "Default"
names = ["{}"]
"#,
                names_100("A")
            ),
        );
        write(
            &dir,
            "names/other.toml",
            &format!(
                r#"
id = "other"
name = "male"
title = "Other"
names = ["{}"]
"#,
                names_100("B")
            ),
        );
        assert!(load_namesets(&store, "names/manifest.toml").is_err());
    }

    #[test]
    fn loads_real_assets() {
        let store = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let catalog = load_namesets(&store, "namesets/manifest.toml").unwrap();
        assert!(!catalog.is_empty());
        let names = catalog.names_for_config(0);
        assert!(!names.is_empty());
        assert!(names.len() > 50);
    }
}

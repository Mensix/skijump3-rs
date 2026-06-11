use crate::error::AssetError;
use crate::files::FileStore;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct TeamDef {
    pub name: String,
    /// 1-indexed indices into the names list (75 names).
    pub members: [usize; 4],
}

#[derive(Debug, Clone)]
pub struct NameCatalog {
    namesets: Vec<NameSet>,
}

#[derive(Debug, Clone)]
pub(crate) struct NameSet {
    pub(crate) title: String,
    pub(crate) names: Vec<String>,
    pub(crate) teams: Vec<TeamDef>,
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

    pub(crate) fn teams_for_config(&self, namenumber: i32) -> &[TeamDef] {
        if namenumber >= 0 && (namenumber as usize) < self.namesets.len() {
            &self.namesets[namenumber as usize].teams
        } else {
            &self.namesets[0].teams
        }
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
struct TeamToml {
    name: String,
    #[serde(default)]
    members: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct NameSetToml {
    id: String,
    name: String,
    title: String,
    names: Vec<String>,
    #[serde(default)]
    teams: Vec<TeamToml>,
}

pub(crate) fn load_namesets(
    files: &FileStore,
    manifest_path: &str,
) -> Result<NameCatalog, AssetError> {
    let manifest: NameSetManifest = super::read_toml(files, manifest_path)?;

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
        let ns: NameSetToml = super::read_toml(files, &full_path)?;

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

        let teams: Vec<TeamDef> = ns
            .teams
            .iter()
            .map(|t| TeamDef {
                name: t.name.clone(),
                members: {
                    let m = &t.members;
                    if m.len() < 4 {
                        [0, 0, 0, 0]
                    } else {
                        [m[0], m[1], m[2], m[3]]
                    }
                },
            })
            .collect();

        namesets.push(NameSet {
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
    use super::super::test_support::*;
    use super::*;
    use crate::files::FileStore;

    fn names_100(s: &str) -> String {
        let names: Vec<String> = (0..100).map(|i| format!("{s}_{i}")).collect();
        names.join("\", \"")
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
        assert!(catalog.len() > 0);
        let names = catalog.names_for_config(0);
        assert!(!names.is_empty());
        assert!(names.len() > 50);
    }
}

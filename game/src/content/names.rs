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

pub(crate) fn load_namesets(files: &FileStore, manifest_path: &str) -> Result<NameCatalog, String> {
    let data = files
        .read(manifest_path)
        .map_err(|e| format!("Failed to read {manifest_path}: {e}"))?;
    let text = std::str::from_utf8(&data)
        .map_err(|e| format!("Nameset manifest is not valid UTF-8: {e}"))?;
    let manifest: NameSetManifest =
        toml::from_str(text).map_err(|e| format!("Failed to parse nameset manifest: {e}"))?;

    if manifest.format_version != 1 {
        return Err(format!(
            "Unsupported nameset manifest format_version: {}",
            manifest.format_version
        ));
    }
    if manifest.namesets.is_empty() {
        return Err("Nameset manifest has no namesets".to_string());
    }

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut namesets = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for entry in &manifest.namesets {
        let full_path = format!("{base_dir}{}", entry.file);
        let data = files
            .read(&full_path)
            .map_err(|e| format!("Failed to read {full_path}: {e}"))?;
        let text = std::str::from_utf8(&data)
            .map_err(|e| format!("{full_path} is not valid UTF-8: {e}"))?;
        let ns_toml: NameSetToml =
            toml::from_str(text).map_err(|e| format!("Failed to parse {full_path}: {e}"))?;

        if ns_toml.id != entry.id {
            return Err(format!(
                "Nameset id mismatch in {full_path}: manifest has '{}', file has '{}'",
                entry.id, ns_toml.id
            ));
        }
        if ns_toml.name.is_empty() {
            return Err(format!(
                "Nameset '{}' has empty name in {full_path}",
                entry.id
            ));
        }
        if ns_toml.title.is_empty() {
            return Err(format!(
                "Nameset '{}' has empty title in {full_path}",
                entry.id
            ));
        }
        if ns_toml.names.is_empty() {
            return Err(format!(
                "Nameset '{}' has no names in {full_path}",
                entry.id
            ));
        }

        if !seen_ids.insert(ns_toml.id.clone()) {
            return Err(format!("Duplicate nameset id '{}'", entry.id));
        }

        // Validate team members
        if let Some(ref teams) = ns_toml.teams {
            for team in teams {
                if team.name.is_empty() {
                    return Err(format!(
                        "Nameset '{}' has empty team name in {full_path}",
                        entry.id
                    ));
                }
                for &m in &team.members {
                    if m == 0 || m > ns_toml.names.len() {
                        return Err(format!(
                            "Nameset '{}' team '{}' has invalid member index {m} in {full_path} \
                             (valid: 1..={})",
                            entry.id,
                            team.name,
                            ns_toml.names.len()
                        ));
                    }
                }
            }
        }

        let teams = ns_toml
            .teams
            .unwrap_or_default()
            .into_iter()
            .map(|t| NameTeam {
                name: t.name,
                members: t.members,
            })
            .collect();

        namesets.push(NameSet {
            id: ns_toml.id,
            title: ns_toml.title,
            names: ns_toml.names,
            teams,
        });
    }

    if !seen_ids.contains(&manifest.default) {
        return Err(format!(
            "Default nameset '{}' not found in manifest",
            manifest.default
        ));
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

    #[test]
    fn loads_minimal_nameset() {
        let (store, dir) = make_files();
        write(
            &dir,
            "namesets/manifest.toml",
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
            "namesets/default.toml",
            r#"
id = "default"
name = "Default"
title = "Default Names"

names = ["Alice", "Bob", "Charlie"]
"#,
        );

        let catalog = load_namesets(&store, "namesets/manifest.toml").unwrap();
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog.names_for_config(0), &["Alice", "Bob", "Charlie"]);
        assert_eq!(catalog.title_for_config(0), "Default Names");
    }

    #[test]
    fn loads_with_teams() {
        let (store, dir) = make_files();
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "a"

[[namesets]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "namesets/a.toml",
            r#"
id = "a"
name = "A"
title = "Test A"

names = ["One", "Two", "Three", "Four", "Five"]

[[teams]]
name = "Team Alpha"
members = [1, 2, 3, 4]
"#,
        );

        let catalog = load_namesets(&store, "namesets/manifest.toml").unwrap();
        assert_eq!(catalog.namesets.len(), 1);
        assert_eq!(catalog.namesets[0].teams.len(), 1);
        assert_eq!(catalog.namesets[0].teams[0].name, "Team Alpha");
        assert_eq!(catalog.namesets[0].teams[0].members, vec![1, 2, 3, 4]);
    }

    #[test]
    fn invalid_nameset_index_falls_back_to_first() {
        let (store, dir) = make_files();
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "a"

[[namesets]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "namesets/a.toml",
            r#"
id = "a"
name = "A"
title = "Test A"
names = ["Only"]
"#,
        );

        let catalog = load_namesets(&store, "namesets/manifest.toml").unwrap();
        assert_eq!(catalog.names_for_config(999), &["Only"]);
        assert_eq!(catalog.names_for_config(-1), &["Only"]);
    }

    #[test]
    fn rejects_invalid_team_member_zero() {
        let (store, dir) = make_files();
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "a"

[[namesets]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "namesets/a.toml",
            r#"
id = "a"
name = "A"
title = "Test"
names = ["One"]

[[teams]]
name = "Bad"
members = [0]
"#,
        );

        let result = load_namesets(&store, "namesets/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid member"));
    }

    #[test]
    fn rejects_duplicate_nameset_id() {
        let (store, dir) = make_files();
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "a"

[[namesets]]
id = "a"
file = "a.toml"

[[namesets]]
id = "a"
file = "b.toml"
"#,
        );
        write(
            &dir,
            "namesets/a.toml",
            r#"
id = "a"
name = "A"
title = "Test"
names = ["One"]
"#,
        );
        write(
            &dir,
            "namesets/b.toml",
            r#"
id = "a"
name = "B"
title = "Test"
names = ["Two"]
"#,
        );

        let result = load_namesets(&store, "namesets/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate nameset id"));
    }

    #[test]
    fn rejects_id_mismatch() {
        let (store, dir) = make_files();
        write(
            &dir,
            "namesets/manifest.toml",
            r#"
format_version = 1
default = "a"

[[namesets]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "namesets/a.toml",
            r#"
id = "b"
name = "B"
title = "Test"
names = ["X"]
"#,
        );

        let result = load_namesets(&store, "namesets/manifest.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mismatch"));
    }

    #[test]
    fn load_real_assets_smoke_test() {
        let asset_dir = std::path::PathBuf::from("game/assets");
        if !asset_dir.join("namesets/manifest.toml").exists() {
            return;
        }
        let save_dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(asset_dir, save_dir.path().to_path_buf());
        let catalog = load_namesets(&files, "namesets/manifest.toml").unwrap();

        assert_eq!(catalog.len(), 3);
        assert_eq!(
            catalog.title_for_config(0),
            "Original names v3.11 - DO NOT EDIT"
        );
        assert_eq!(catalog.title_for_config(1), "The Cool Dudes List");
        assert_eq!(catalog.title_for_config(2), "1987/1988 season");
        assert_eq!(catalog.names_for_config(0)[0], "Roar Ljøkelsøy");
        assert_eq!(catalog.names_for_config(1)[1], "Mika Häkkinen");
        assert_eq!(catalog.names_for_config(2)[2], "Primoz Ulaga");
    }
}

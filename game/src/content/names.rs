use crate::files::FileStore;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct TeamDef {
    pub name: String,
    pub members: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
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
    pub(crate) fn names_for_config(&self, name_set_index: i32) -> &[String] {
        if name_set_index >= 0 && (name_set_index as usize) < self.namesets.len() {
            &self.namesets[name_set_index as usize].names
        } else {
            self.namesets
                .first()
                .map_or(&[], |nameset| nameset.names.as_slice())
        }
    }

    pub(crate) fn title_for_config(&self, name_set_index: i32) -> &str {
        if name_set_index >= 0 && (name_set_index as usize) < self.namesets.len() {
            &self.namesets[name_set_index as usize].title
        } else {
            self.namesets.first().map_or("?", |nameset| &nameset.title)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.namesets.len()
    }

    pub(crate) fn teams_for_config(&self, name_set_index: i32) -> &[TeamDef] {
        if name_set_index >= 0 && (name_set_index as usize) < self.namesets.len() {
            &self.namesets[name_set_index as usize].teams
        } else {
            self.namesets
                .first()
                .map_or(&[], |nameset| nameset.teams.as_slice())
        }
    }
}

#[derive(Debug, Default, Deserialize)]

struct NameSetManifest {
    namesets: Vec<NameSetEntry>,
}

#[derive(Debug, Default, Deserialize)]

struct NameSetEntry {
    file: String,
}

#[derive(Debug, Deserialize)]

struct TeamToml {
    name: String,
    #[serde(default)]
    members: Vec<usize>,
}

#[derive(Debug, Default, Deserialize)]
struct NameSetToml {
    title: String,
    names: Vec<String>,
    #[serde(default)]
    teams: Vec<TeamToml>,
}

pub(crate) fn load_namesets(files: &FileStore, manifest_path: &str) -> Result<NameCatalog, String> {
    let manifest: NameSetManifest = super::read_toml(files, manifest_path)?;

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut namesets: Vec<NameSet> = Vec::new();
    for entry in &manifest.namesets {
        let full_path = format!("{base_dir}{}", entry.file);
        let ns: NameSetToml = super::read_toml(files, &full_path)?;
        let teams = ns
            .teams
            .iter()
            .map(|t| TeamDef {
                name: t.name.clone(),
                members: t.members.iter().map(|&idx| idx.saturating_sub(1)).collect(),
            })
            .collect();
        namesets.push(NameSet {
            title: ns.title,
            names: ns.names,
            teams,
        });
    }

    Ok(NameCatalog { namesets })
}

use crate::files::FileStore;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct TeamDef {
    pub name: String,
    pub members: Vec<usize>,
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
    pub(crate) fn names_for_config(&self, name_set_index: i32) -> &[String] {
        if name_set_index >= 0 && (name_set_index as usize) < self.namesets.len() {
            &self.namesets[name_set_index as usize].names
        } else {
            &self.namesets[0].names
        }
    }

    pub(crate) fn title_for_config(&self, name_set_index: i32) -> &str {
        if name_set_index >= 0 && (name_set_index as usize) < self.namesets.len() {
            &self.namesets[name_set_index as usize].title
        } else {
            &self.namesets[0].title
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.namesets.len()
    }

    pub(crate) fn teams_for_config(&self, name_set_index: i32) -> &[TeamDef] {
        if name_set_index >= 0 && (name_set_index as usize) < self.namesets.len() {
            &self.namesets[name_set_index as usize].teams
        } else {
            &self.namesets[0].teams
        }
    }
}

#[derive(Debug, Deserialize)]

struct NameSetManifest {
    namesets: Vec<NameSetEntry>,
}

#[derive(Debug, Deserialize)]

struct NameSetEntry {
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
    title: String,
    names: Vec<String>,
    #[serde(default)]
    teams: Vec<TeamToml>,
}

pub(crate) fn load_namesets(
    files: &FileStore,
    manifest_path: &str,
) -> NameCatalog {
    let manifest: NameSetManifest = super::read_toml(files, manifest_path);

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut namesets: Vec<NameSet> = Vec::new();
    for entry in &manifest.namesets {
        let full_path = format!("{base_dir}{}", entry.file);
        let ns: NameSetToml = super::read_toml(files, &full_path);
        let teams = ns.teams.iter().map(|t| TeamDef {
            name: t.name.clone(),
            members: t.members.clone(),
        }).collect();
        namesets.push(NameSet {
            title: ns.title,
            names: ns.names,
            teams,
        });
    }

    NameCatalog { namesets }
}

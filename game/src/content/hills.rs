use crate::data::hill::{HillCatalog, HillInfo};
use crate::files::FileStore;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct HillsManifest {
    format_version: u32,
    default: String,
    catalogs: Vec<CatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct HillCatalogToml {
    id: String,
    name: String,
    hills: Vec<HillToml>,
}

#[derive(Debug, Deserialize)]
struct HillToml {
    id: String,
    name: String,
    #[serde(default)]
    terrain_index: Option<TerrainIndexToml>,
    kr: i64,
    front_index: String,
    back_index: String,
    back_brightness: i64,
    back_mirror: bool,
    vx_final: i64,
    pk_hundred: i64,
    pl_save_ten_thousand: i64,
    author: String,
    checksum: i64,
    profile_checksum: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TerrainIndexToml {
    String(String),
    Number(usize),
}

impl TerrainIndexToml {
    fn into_terrain_id(self) -> String {
        match self {
            Self::String(value) => value,
            Self::Number(value) => value.to_string(),
        }
    }
}

pub(crate) fn load_hills(
    files: &FileStore,
    manifest_path: &str,
) -> HillCatalog {
    let manifest: HillsManifest = super::read_toml(files, manifest_path);

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut all_hills = Vec::new();
    let mut seen_catalog_ids: HashSet<String> = HashSet::new();
    let mut seen_hill_ids: HashSet<String> = HashSet::new();

    for entry in &manifest.catalogs {
        let full_path = format!("{base_dir}{}", entry.file);
        let cat: HillCatalogToml = super::read_toml(files, &full_path);
        seen_catalog_ids.insert(cat.id.clone());
        append_catalog(&mut all_hills, &mut seen_hill_ids, &cat);
    }

    let mut custom_names = files.list_save_subdir_by_ext("custom_hills", "toml");
    custom_names.sort();
    for name in custom_names {
        let full_path = format!("custom_hills/{name}");
        let cat: HillCatalogToml = super::read_toml(files, &full_path);
        append_catalog(&mut all_hills, &mut seen_hill_ids, &cat);
    }

    HillCatalog::new(all_hills)
}

fn append_catalog(
    all_hills: &mut Vec<HillInfo>,
    seen_hill_ids: &mut HashSet<String>,
    cat: &HillCatalogToml,
) {
    for (idx, h) in cat.hills.iter().enumerate() {
        seen_hill_ids.insert(format!("{}:{}", cat.id, h.id));
        all_hills.push(HillInfo {
            name: h.name.clone(),
            kr: h.kr,
            front_index: h.front_index.clone(),
            back_index: h.back_index.clone(),
            back_brightness: h.back_brightness,
            back_mirror: i64::from(h.back_mirror),
            vx_final: h.vx_final,
            pk_hundred: h.pk_hundred,
            pl_save_ten_thousand: h.pl_save_ten_thousand,
            author: h.author.clone(),
            checksum: h.checksum,
            profile_checksum: h.profile_checksum,
            terrain_id: h
                .terrain_index
                .clone()
                .map(TerrainIndexToml::into_terrain_id)
                .unwrap_or_else(|| idx.to_string()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::files::FileStore;

    #[test]
    fn loads_real_assets() {
        let store = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let catalog = load_hills(&store, "hills/manifest.toml");
        assert!(catalog.hill(0).is_some());

        let kuopio = catalog.hill(0).unwrap();
        assert_eq!(kuopio.name, "kuopio");
        assert_eq!(kuopio.kr, 120);
        assert_eq!(kuopio.front_index, "1");
        assert_eq!(kuopio.back_index, "0");
        assert_eq!(kuopio.back_brightness, 90);
        assert_eq!(kuopio.back_mirror, 0);
        assert_eq!(kuopio.vx_final, 148);
        assert_eq!(kuopio.pk_hundred, 89);
        assert_eq!(kuopio.pl_save_ten_thousand, 3217);
        assert_eq!(kuopio.author, "SJ v3.00 Original");
        assert_eq!(kuopio.checksum, 932638);
        assert_eq!(kuopio.profile_checksum, 1009542);
        assert!((kuopio.pk() - 0.89).abs() < f64::EPSILON);
        assert!((kuopio.pl_save() - 0.3217).abs() < f64::EPSILON);
    }

    #[test]
    fn back_mirror_true_produces_1() {
        let (store, dir) = make_files();
        write(
            &dir,
            "hills/manifest.toml",
            r#"
format_version = 1
default = "default"

[[catalogs]]
id = "default"
file = "default.toml"
"#,
        );
        write(
            &dir,
            "hills/default.toml",
            r#"
id = "default"
name = "Default"
[[hills]]
id = "S"
name = "sapporo"
kr = 120
front_index = "16"
back_index = "4"
back_brightness = 95
back_mirror = true
vx_final = 151
pk_hundred = 88
pl_save_ten_thousand = 3220
author = "t"
checksum = 0
profile_checksum = 0
"#,
        );
        let catalog = load_hills(&store, "hills/manifest.toml");
        assert_eq!(catalog.hill(0).unwrap().back_mirror, 1);
    }
}

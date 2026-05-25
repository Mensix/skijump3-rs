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

pub(crate) fn load_hills(files: &FileStore, manifest_path: &str) -> Result<HillCatalog, String> {
    let data = files
        .read(manifest_path)
        .map_err(|e| format!("Failed to read {manifest_path}: {e}"))?;
    let text = std::str::from_utf8(&data)
        .map_err(|e| format!("Hills manifest is not valid UTF-8: {e}"))?;
    let manifest: HillsManifest =
        toml::from_str(text).map_err(|e| format!("Failed to parse hills manifest: {e}"))?;

    if manifest.format_version != 1 {
        return Err(format!(
            "Unsupported hills manifest format_version: {}",
            manifest.format_version
        ));
    }
    if manifest.catalogs.is_empty() {
        return Err("Hills manifest has no catalogs".to_string());
    }

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut all_hills = Vec::new();
    let mut seen_catalog_ids: HashSet<String> = HashSet::new();
    let mut seen_hill_ids: HashSet<String> = HashSet::new();

    for entry in &manifest.catalogs {
        let full_path = format!("{base_dir}{}", entry.file);
        let data = files
            .read(&full_path)
            .map_err(|e| format!("Failed to read {full_path}: {e}"))?;
        let text = std::str::from_utf8(&data)
            .map_err(|e| format!("{full_path} is not valid UTF-8: {e}"))?;
        let cat: HillCatalogToml =
            toml::from_str(text).map_err(|e| format!("Failed to parse {full_path}: {e}"))?;

        if cat.id != entry.id {
            return Err(format!(
                "Hill catalog id mismatch in {full_path}: manifest has '{}', file has '{}'",
                entry.id, cat.id
            ));
        }
        if cat.name.is_empty() {
            return Err(format!(
                "Hill catalog '{}' has empty name in {full_path}",
                entry.id
            ));
        }
        if cat.hills.is_empty() {
            return Err(format!(
                "Hill catalog '{}' has no hills in {full_path}",
                entry.id
            ));
        }

        if !seen_catalog_ids.insert(cat.id.clone()) {
            return Err(format!("Duplicate hill catalog id '{}'", entry.id));
        }

        for h in &cat.hills {
            if h.name.is_empty() {
                return Err(format!("Hill '{}' in {full_path} has empty name", h.id));
            }
            if h.front_index.is_empty() {
                return Err(format!(
                    "Hill '{}' in {full_path} has empty front_index",
                    h.id
                ));
            }
            if h.back_index.is_empty() {
                return Err(format!(
                    "Hill '{}' in {full_path} has empty back_index",
                    h.id
                ));
            }
            if h.author.is_empty() {
                return Err(format!("Hill '{}' in {full_path} has empty author", h.id));
            }
            if h.kr <= 0 {
                return Err(format!(
                    "Hill '{}' in {full_path} has non-positive kr ({})",
                    h.id, h.kr
                ));
            }
            if h.pk_hundred <= 0 {
                return Err(format!(
                    "Hill '{}' in {full_path} has non-positive pk_hundred ({})",
                    h.id, h.pk_hundred
                ));
            }
            if h.pl_save_ten_thousand <= 0 {
                return Err(format!(
                    "Hill '{}' in {full_path} has non-positive pl_save_ten_thousand ({})",
                    h.id, h.pl_save_ten_thousand
                ));
            }

            if !seen_hill_ids.insert(format!("{}:{}", cat.id, h.id)) {
                return Err(format!(
                    "Duplicate hill id '{}' in catalog '{}'",
                    h.id, cat.id
                ));
            }

            all_hills.push(HillInfo {
                name: h.name.clone(),
                kr: h.kr,
                front_index: h.front_index.clone(),
                back_index: h.back_index.clone(),
                back_brightness: h.back_brightness,
                back_mirror: if h.back_mirror { 1 } else { 0 },
                vx_final: h.vx_final,
                pk_hundred: h.pk_hundred,
                pl_save_ten_thousand: h.pl_save_ten_thousand,
                author: h.author.clone(),
                checksum: h.checksum,
                profile_checksum: h.profile_checksum,
            });
        }
    }

    if !seen_catalog_ids.contains(&manifest.default) {
        return Err(format!(
            "Default hill catalog '{}' not found in manifest",
            manifest.default
        ));
    }

    Ok(HillCatalog::new(all_hills))
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
    fn loads_minimal_hill_catalog() {
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
id = "A"
name = "testhill"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "test"
checksum = 0
profile_checksum = 0
"#,
        );

        let catalog = load_hills(&store, "hills/manifest.toml").unwrap();
        assert_eq!(catalog.len(), 1);
        let hill = catalog.hill(0).unwrap();
        assert_eq!(hill.name, "testhill");
        assert_eq!(hill.kr, 90);
        assert_eq!(hill.back_mirror, 0);
    }

    #[test]
    fn rejects_missing_catalog() {
        let (store, dir) = make_files();
        write(
            &dir,
            "hills/manifest.toml",
            r#"
format_version = 1
default = "nonexistent"

[[catalogs]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "hills/a.toml",
            r#"
id = "a"
name = "A"
[[hills]]
id = "H"
name = "h"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "t"
checksum = 0
profile_checksum = 0
"#,
        );
        assert!(load_hills(&store, "hills/manifest.toml").is_err());
    }

    #[test]
    fn rejects_empty_catalog_id() {
        let (store, dir) = make_files();
        write(
            &dir,
            "hills/manifest.toml",
            r#"
format_version = 1
default = "default"

[[catalogs]]
id = "a"
file = "a.toml"
"#,
        );
        write(
            &dir,
            "hills/a.toml",
            r#"
id = "b"
name = "A"
[[hills]]
id = "H"
name = "h"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "t"
checksum = 0
profile_checksum = 0
"#,
        );
        assert!(load_hills(&store, "hills/manifest.toml").is_err());
    }

    #[test]
    fn rejects_duplicate_hill_ids() {
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
id = "A"
name = "h1"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "t"
checksum = 0
profile_checksum = 0

[[hills]]
id = "A"
name = "h2"
kr = 90
front_index = "1"
back_index = "0"
back_brightness = 90
back_mirror = false
vx_final = 130
pk_hundred = 85
pl_save_ten_thousand = 3200
author = "t"
checksum = 0
profile_checksum = 0
"#,
        );
        assert!(load_hills(&store, "hills/manifest.toml").is_err());
    }

    #[test]
    fn loads_real_assets() {
        let store = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let catalog = load_hills(&store, "hills/manifest.toml").unwrap();
        assert!(catalog.len() >= 1);

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
        let catalog = load_hills(&store, "hills/manifest.toml").unwrap();
        assert_eq!(catalog.hill(0).unwrap().back_mirror, 1);
    }
}

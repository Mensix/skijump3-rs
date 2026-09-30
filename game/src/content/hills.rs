use crate::data::custom_hill::{CustomHill, CustomHillCatalog};
use crate::data::hill::{generated_hill_image_path, metadata_checksum, HillCatalog, HillInfo};
use crate::data::hill_profile::stored_profile_matches;
use crate::files::FileStore;
use crate::gfx::png::load_png;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]

struct HillsManifest {
    catalogs: Vec<CatalogEntry>,
}

#[derive(Debug, Default, Deserialize)]

struct CatalogEntry {
    id: String,
    file: String,
}

#[derive(Debug, Default, Deserialize)]
struct HillCatalogToml {
    #[serde(default)]
    id: Option<String>,
    checksum_version: u32,
    hills: Vec<HillToml>,
}

#[derive(Debug, Deserialize)]
struct HillToml {
    #[serde(default)]
    id: Option<String>,
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

pub(crate) fn load_hills(files: &FileStore, manifest_path: &str) -> Result<HillCatalog, String> {
    let manifest: HillsManifest = super::read_toml(files, manifest_path)?;

    let base_dir = match manifest_path.rfind('/') {
        Some(pos) => &manifest_path[..=pos],
        None => "",
    };

    let mut all_hills = Vec::new();

    for entry in &manifest.catalogs {
        let full_path = format!("{base_dir}{}", entry.file);
        let cat: HillCatalogToml = super::read_toml(files, &full_path)?;
        if cat.checksum_version != 1 {
            return Err(format!("{full_path}: unsupported checksum version"));
        }
        let catalog_id = cat.id.as_deref().unwrap_or(&entry.id);
        append_catalog(&mut all_hills, &cat, catalog_id);
    }

    let original_count = all_hills.len();

    let mut custom_names = files.list_save_subdir_by_ext("custom_hills", "toml");
    custom_names.sort();
    for name in custom_names {
        let full_path = format!("custom_hills/{name}");
        let data = files.read_save(&full_path);
        if data.is_empty() {
            continue;
        }
        let Ok(text) = std::str::from_utf8(&data) else {
            continue;
        };
        let Ok(cat) = toml::from_str::<CustomHillCatalog>(text) else {
            continue;
        };
        if !valid_custom_catalog(files, &cat) {
            continue;
        }
        append_custom_catalog(&mut all_hills, &cat);
    }

    Ok(HillCatalog::new(all_hills, original_count))
}

fn valid_custom_catalog(files: &FileStore, cat: &CustomHillCatalog) -> bool {
    if cat.hills.is_empty() || cat.checksum_version != 1 {
        return false;
    }
    cat.hills.iter().all(|hill| {
        let indices_valid = [&hill.front_index, &hill.back_index]
            .into_iter()
            .all(|index| !index.is_empty() && index.chars().all(|c| c.is_ascii_alphanumeric()));
        let values_valid = (40..=300).contains(&hill.kr)
            && (0..=255).contains(&hill.back_brightness)
            && (100..=185).contains(&hill.vx_final)
            && (50..=150).contains(&hill.pk_hundred)
            && (3204..=3234).contains(&hill.pl_save_ten_thousand)
            && hill.checksum != 0
            && hill.profile_checksum != 0;
        if !indices_valid || !values_valid {
            return false;
        }
        let back = files.read_save_or_asset(&generated_hill_image_path(&hill.back_index, "back"));
        if load_png(&back).is_none() {
            return false;
        }
        let info = custom_hill_info(hill, "checksum");
        let computed_metadata = metadata_checksum(&info);
        let profile_ok = stored_profile_matches(files, &hill.front_index, hill.profile_checksum);
        let metadata_ok = hill.checksum == computed_metadata;
        profile_ok && metadata_ok
    })
}

pub(crate) fn custom_hill_details(files: &FileStore, filename: &str) -> Option<(String, i64)> {
    let data = files.read_save(&format!("custom_hills/{filename}"));
    let text = std::str::from_utf8(&data).ok()?;
    let catalog = toml::from_str::<CustomHillCatalog>(text).ok()?;
    if !valid_custom_catalog(files, &catalog) {
        return None;
    }
    catalog
        .hills
        .first()
        .map(|hill| (hill.name.clone(), hill.kr))
}

fn append_catalog(all_hills: &mut Vec<HillInfo>, cat: &HillCatalogToml, catalog_id: &str) {
    for (idx, h) in cat.hills.iter().enumerate() {
        all_hills.push(hill_info(h, catalog_id, idx));
    }
}

fn append_custom_catalog(all_hills: &mut Vec<HillInfo>, cat: &CustomHillCatalog) {
    for hill in &cat.hills {
        all_hills.push(custom_hill_info(hill, &cat.id));
    }
}

fn hill_info(h: &HillToml, catalog_id: &str, idx: usize) -> HillInfo {
    let hill_id = h.id.clone().unwrap_or_else(|| idx.to_string());
    HillInfo {
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
        record_key: format!("{catalog_id}:{hill_id}"),
    }
}

fn custom_hill_info(h: &CustomHill, catalog_id: &str) -> HillInfo {
    HillInfo {
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
        terrain_id: h.terrain_index.clone(),
        record_key: format!("{catalog_id}:{}", h.id),
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::data::hill_profile::profile_checksum;
    use crate::files::FileStore;

    fn real_files(save_dir: &std::path::Path) -> FileStore {
        FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            save_dir.to_path_buf(),
        )
    }

    fn custom_catalog(files: &FileStore, name: &str) -> String {
        let profile_checksum = profile_checksum(files, "1").unwrap();
        let info = HillInfo {
            name: name.to_string(),
            kr: 120,
            front_index: "1".to_string(),
            back_index: "4".to_string(),
            back_brightness: 75,
            back_mirror: 1,
            vx_final: 140,
            pk_hundred: 100,
            pl_save_ten_thousand: 3210,
            author: "test".to_string(),
            profile_checksum,
            ..HillInfo::default()
        };
        let checksum = metadata_checksum(&info);
        format!(
            r#"id = "test"
name = "test catalog"
checksum_version = 1

[[hills]]
id = "test"
name = "{name}"
terrain_index = "1"
kr = 120
front_index = "1"
back_index = "4"
back_brightness = 75
back_mirror = true
vx_final = 140
pk_hundred = 100
pl_save_ten_thousand = 3210
author = "test"
checksum = {checksum}
profile_checksum = {profile_checksum}
"#
        )
    }

    #[test]
    fn loads_real_assets() {
        let store = FileStore::new(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let Ok(catalog) = load_hills(&store, "hills/manifest.toml") else {
            return;
        };
        assert!(catalog.hill(0).is_some());

        let kuopio = catalog.hill(0).unwrap();
        assert_eq!(kuopio.name, "kuopio");
        assert_eq!(kuopio.kr, 120);
        assert_eq!(kuopio.front_index, "0");
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
checksum_version = 1
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
        let Ok(catalog) = load_hills(&store, "hills/manifest.toml") else {
            return;
        };
        assert_eq!(catalog.hill(0).unwrap().back_mirror, 1);
    }

    #[test]
    fn valid_custom_hill_refresh_is_visible_to_existing_catalog_clones() {
        let saves = tempfile::tempdir().unwrap();
        let files = real_files(saves.path());
        let Ok(catalog) = load_hills(&files, "hills/manifest.toml") else {
            return;
        };
        let observer = catalog.clone();
        let original_count = catalog.original_count();
        files.write(
            "custom_hills/test.toml",
            custom_catalog(&files, "Live hill").as_bytes(),
        );

        let Ok(refreshed) = load_hills(&files, "hills/manifest.toml") else {
            return;
        };
        catalog.replace(refreshed);

        assert_eq!(observer.len(), original_count + 1);
        assert_eq!(observer.hill(original_count).unwrap().name, "Live hill");
    }

    #[test]
    fn checksum_tampering_and_malformed_toml_are_rejected_without_panicking() {
        let saves = tempfile::tempdir().unwrap();
        let files = real_files(saves.path());
        let Ok(original) = load_hills(&files, "hills/manifest.toml") else {
            return;
        };
        let original_count = original.original_count();
        let tampered = custom_catalog(&files, "Original").replace("Original", "Tampered");
        files.write("custom_hills/tampered.toml", tampered.as_bytes());
        files.write("custom_hills/malformed.toml", b"[[hills]\nnot toml");

        let Ok(catalog) = load_hills(&files, "hills/manifest.toml") else {
            return;
        };

        assert_eq!(catalog.len(), original_count);
    }
}

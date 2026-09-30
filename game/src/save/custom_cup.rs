use crate::competition::types::{CustomCupScoring, MAX_CUSTOM_CUP_HILLS};
use crate::data::hill::HillCatalog;
use crate::data::records::Hiscore;
use crate::files::FileStore;
use crate::save::parse_toml;
use serde::{Deserialize, Serialize};

pub(crate) const CUSTOM_CUP_DIR: &str = "custom_cups";
pub(crate) const FORMAT_VERSION: u32 = 1;
pub(crate) const MAX_CUSTOM_CUP_NAME_LEN: usize = 8;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CustomCupFile {
    pub(crate) format_version: u32,
    pub(crate) hill_refs: Vec<String>,
    pub(crate) scoring: CustomCupScoring,
    pub(crate) records: Vec<Hiscore>,
}

pub(crate) fn normalize_custom_cup_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_graphic() && !matches!(ch, '/' | '\\' | '.'))
        .take(MAX_CUSTOM_CUP_NAME_LEN)
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

pub(crate) fn custom_cup_path(name: &str) -> String {
    format!("{CUSTOM_CUP_DIR}/{}.toml", normalize_custom_cup_name(name))
}

pub(crate) fn named_custom_cup_source(name: &str) -> Option<String> {
    let name = normalize_custom_cup_name(name);
    (!name.is_empty() && name != "TEMP").then_some(name)
}

pub(crate) fn resolve_custom_cup_hills(
    file: &CustomCupFile,
    hills: &HillCatalog,
) -> Option<Vec<usize>> {
    if file.hill_refs.len() > MAX_CUSTOM_CUP_HILLS {
        return None;
    }
    file.hill_refs
        .iter()
        .map(|key| hills.index_by_record_key(key))
        .collect()
}

pub(crate) fn load_custom_cup_file(files: &FileStore, name: &str) -> Option<CustomCupFile> {
    let path = custom_cup_path(name);
    if !files.exists_save(&path) {
        return None;
    }
    let file = parse_toml::<CustomCupFile>(&files.read_save(&path)).ok()?;
    (file.format_version == FORMAT_VERSION).then_some(file)
}

pub(crate) fn save_custom_cup_file(files: &FileStore, name: &str, file: &CustomCupFile) -> bool {
    let Ok(bytes) = toml::to_string(file).map(String::into_bytes) else {
        return false;
    };
    files.write(&custom_cup_path(name), &bytes)
}

pub(crate) fn discover_custom_cup_files(files: &FileStore) -> Vec<(String, CustomCupFile)> {
    let mut cups: Vec<_> = files
        .list_save_subdir_by_ext(CUSTOM_CUP_DIR, "toml")
        .into_iter()
        .filter_map(|filename| {
            let name = filename.trim_end_matches(".toml").to_string();
            load_custom_cup_file(files, &name).map(|file| (name, file))
        })
        .collect();
    cups.sort_by(|a, b| a.0.cmp(&b.0));
    cups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::hill::HillInfo;
    use std::path::PathBuf;

    #[test]
    fn legacy_custom_cup_file_is_rejected() {
        let file = toml::from_str::<CustomCupFile>("format_version = 1\nhills = [0, 4, 2]\n");

        assert!(file.is_err());
    }

    #[test]
    fn custom_cup_file_round_trips_aggregate_scoring() {
        let file = CustomCupFile {
            format_version: 1,
            hill_refs: vec!["builtin:1".into(), "builtin:3".into()],
            scoring: CustomCupScoring::AggregateJumpPoints,
            records: vec![],
        };

        let saved = toml::to_string(&file).unwrap();
        let loaded: CustomCupFile = toml::from_str(&saved).unwrap();

        assert_eq!(loaded.hill_refs, file.hill_refs);
        assert_eq!(loaded.scoring, CustomCupScoring::AggregateJumpPoints);
    }

    #[test]
    fn discovery_finds_valid_saved_pages_and_skips_malformed_files() {
        let dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(PathBuf::from("/nonexistent"), dir.path().to_path_buf());
        let valid = CustomCupFile {
            format_version: 1,
            hill_refs: vec!["builtin:0".into(), "builtin:2".into()],
            scoring: CustomCupScoring::WorldCupPoints,
            records: vec![],
        };
        files.write(
            &custom_cup_path("CURRENT"),
            toml::to_string(&valid).unwrap().as_bytes(),
        );
        files.write(
            &custom_cup_path("OLDSET"),
            b"format_version = 1\nhills = [0, 2]\n",
        );
        files.write(&custom_cup_path("BROKEN"), b"not toml [[[");

        let pages = discover_custom_cup_files(&files);

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].0, "CURRENT");
        assert!(pages[0].1.records.is_empty());
    }

    #[test]
    fn records_are_persisted_in_toml() {
        let dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(PathBuf::from("/nonexistent"), dir.path().to_path_buf());
        let file = CustomCupFile {
            format_version: FORMAT_VERSION,
            hill_refs: vec!["builtin:0".into()],
            scoring: CustomCupScoring::WorldCupPoints,
            records: vec![Hiscore {
                name: "Player".into(),
                pos: 1,
                score: 100.0,
                time: "now".into(),
                is_computer: false,
            }],
        };

        assert!(save_custom_cup_file(&files, "SCORES", &file));
        assert_eq!(
            load_custom_cup_file(&files, "SCORES").unwrap().records,
            file.records
        );
    }

    #[test]
    fn hill_resolution_rejects_unregistered_references_instead_of_dropping_them() {
        let hills = HillCatalog::new(
            vec![HillInfo {
                record_key: "builtin:0".into(),
                ..Default::default()
            }],
            1,
        );
        let file = CustomCupFile {
            format_version: FORMAT_VERSION,
            hill_refs: vec!["builtin:0".into(), "missing:1".into()],
            scoring: CustomCupScoring::WorldCupPoints,
            records: vec![],
        };

        assert!(resolve_custom_cup_hills(&file, &hills).is_none());
    }

    #[test]
    fn hill_resolution_rejects_more_than_the_supported_number_of_hills() {
        let hills = HillCatalog::default();
        let file = CustomCupFile {
            format_version: FORMAT_VERSION,
            hill_refs: vec!["missing".into(); MAX_CUSTOM_CUP_HILLS + 1],
            scoring: CustomCupScoring::WorldCupPoints,
            records: vec![],
        };

        assert!(resolve_custom_cup_hills(&file, &hills).is_none());
    }

    #[test]
    fn temp_is_not_a_persisted_custom_cup_source() {
        assert_eq!(named_custom_cup_source("TEMP"), None);
        assert_eq!(named_custom_cup_source("saved"), Some("SAVED".into()));
    }
}

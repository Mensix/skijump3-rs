use serde::{Deserialize, Serialize};

use crate::data::records::RecordStore;
use crate::save::parse_toml;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RecordsFile {
    format_version: u32,
    #[serde(flatten)]
    store: RecordStore,
}

const FORMAT_VERSION: u32 = 1;

impl RecordStore {
    pub fn from_toml_bytes(data: &[u8]) -> Result<Self, String> {
        let file: RecordsFile = parse_toml(data)?;
        if file.format_version != FORMAT_VERSION {
            return Err(format!(
                "unsupported records format {}",
                file.format_version
            ));
        }
        if file.store.top.is_empty()
            || file.store.hill_records.is_empty()
            || file.store.hill_goals.is_empty()
            || file.store.top.len() > 41
            || file.store.top.iter().any(|record| {
                !record.score.is_finite() || record.score < 0.0 || record.pos > 10_000
            })
            || file
                .store
                .hill_records
                .values()
                .any(|record| !record.len.is_finite() || record.len < 0.0)
            || file
                .store
                .hill_goals
                .values()
                .any(|goal| !goal.is_finite() || *goal < 0.0)
        {
            return Err("record values are out of range".into());
        }
        Ok(file.store)
    }

    pub fn to_toml_bytes(&self) -> Result<Vec<u8>, String> {
        let file = RecordsFile {
            format_version: FORMAT_VERSION,
            store: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled_store() -> RecordStore {
        let path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/hiscores.toml");
        let data = std::fs::read(&path).unwrap();
        RecordStore::from_toml_bytes(&data).unwrap()
    }

    #[test]
    fn toml_roundtrip_preserves_records() {
        let store = bundled_store();
        let toml_bytes = match store.to_toml_bytes() {
            Ok(bytes) => bytes,
            Err(error) => {
                assert!(error.is_empty());
                return;
            }
        };
        let reparsed = RecordStore::from_toml_bytes(&toml_bytes).unwrap();

        assert_eq!(store.top, reparsed.top);
        assert_eq!(store.hill_records, reparsed.hill_records);
        assert_eq!(store.hill_goals, reparsed.hill_goals);
    }

    #[test]
    fn loads_bundled_hiscores() {
        let store = bundled_store();
        assert!(!store.top.is_empty());
        assert!(store.top.len() <= 41);
        assert!(!store.hill_records.is_empty());
        assert_eq!(store.hill_records.len(), 20);
        assert_eq!(store.hill_goals.len(), 20);

        let first = store.top(0);
        assert!(first.is_some());
        assert!(first.unwrap().is_computer);
    }
}

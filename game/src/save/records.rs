use serde::{Deserialize, Serialize};

use crate::data::records::RecordStore;
use crate::save::parse_toml;

#[derive(Debug, Deserialize, Serialize)]
struct RecordsFile {
    #[serde(flatten)]
    store: RecordStore,
}

impl RecordStore {
    pub fn from_toml_bytes(data: &[u8]) -> Self {
        let file: RecordsFile = parse_toml(data);
        file.store
    }

    pub fn to_toml_bytes(&self) -> Vec<u8> {
        let file = RecordsFile {
            store: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn bundled_store() -> RecordStore {
        let data = include_bytes!("../../assets/hiscores.toml");
        RecordStore::from_toml_bytes(data)
    }

    #[test]
    fn toml_roundtrip_preserves_records() {
        let store = bundled_store();
        let toml_bytes = store.to_toml_bytes();
        let reparsed = RecordStore::from_toml_bytes(&toml_bytes);

        assert_eq!(store.top.len(), reparsed.top.len());
        assert_eq!(store.hill_records.len(), reparsed.hill_records.len());

        for i in 0..store.top.len() {
            assert_eq!(store.top[i].name, reparsed.top[i].name);
            assert_eq!(store.top[i].pos, reparsed.top[i].pos);
            assert_eq!(store.top[i].score, reparsed.top[i].score);
            assert_eq!(store.top[i].time, reparsed.top[i].time);
        }

        for i in 0..store.hill_records.len() {
            assert_eq!(store.hill_records[i].name, reparsed.hill_records[i].name);
            assert_eq!(store.hill_records[i].len, reparsed.hill_records[i].len);
            assert_eq!(store.hill_records[i].time, reparsed.hill_records[i].time);
        }
    }

    #[test]
    fn loads_bundled_hiscores() {
        let store = bundled_store();
        assert!(!store.top.is_empty());
        assert!(store.top.len() <= 41);
        assert!(!store.hill_records.is_empty());
        assert!(store.hill_records.len() <= 20);

        let first = store.top(0);
        assert!(first.is_some());
    }
}

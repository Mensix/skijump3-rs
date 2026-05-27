use serde::{Deserialize, Serialize};

use crate::data::records::RecordStore;

/// TOML wrapper — mirrors save/config.rs and save/players.rs pattern.
#[derive(Debug, Deserialize, Serialize)]
struct RecordsFile {
    format_version: u32,
    #[serde(flatten)]
    store: RecordStore,
}

impl RecordStore {
    pub fn from_toml_bytes(data: &[u8]) -> Result<Self, String> {
        let text =
            std::str::from_utf8(data).map_err(|e| format!("Invalid UTF-8 in hiscores: {e}"))?;
        let file: RecordsFile =
            toml::from_str(text).map_err(|e| format!("Failed to parse hiscores: {e}"))?;
        if file.format_version != 1 {
            return Err(format!(
                "Unsupported hiscores format_version: {}",
                file.format_version
            ));
        }

        let store = &file.store;

        if store.top.len() > 41 {
            return Err(format!(
                "hiscores.toml has {} top records (max 41)",
                store.top.len()
            ));
        }
        if store.hill_records.len() > 20 {
            return Err(format!(
                "hiscores.toml has {} hill records (max 20)",
                store.hill_records.len()
            ));
        }

        Ok(file.store)
    }

    pub fn to_toml_bytes(&self) -> Result<Vec<u8>, String> {
        let file = RecordsFile {
            format_version: 1,
            store: self.clone(),
        };
        toml::to_string(&file)
            .map(std::string::String::into_bytes)
            .map_err(|e| format!("Failed to serialize hiscores: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses the bundled hiscores.toml to verify it loads correctly.
    fn bundled_store() -> RecordStore {
        let data = include_bytes!("../../assets/hiscores.toml");
        RecordStore::from_toml_bytes(data).expect("bundled hiscores.toml should be valid")
    }

    #[test]
    fn toml_roundtrip_preserves_records() {
        let store = bundled_store();
        let toml_bytes = store.to_toml_bytes().expect("serialize");
        let reparsed = RecordStore::from_toml_bytes(&toml_bytes).expect("re-parse");

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
    fn rejects_bad_format_version() {
        let bad = br#"format_version = 99
[[top]]
name = "x"
pos = 1
score = 1
time = ""
"#;
        let result = RecordStore::from_toml_bytes(bad);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("format_version"));
    }

    #[test]
    fn rejects_too_many_top_records() {
        let mut toml = String::from("format_version = 1\n");
        for i in 0..50 {
            toml.push_str("[[top]]\n");
            toml.push_str(&format!(
                "name = \"{i}\"\npos = 1\nscore = 1\ntime = \"\"\n"
            ));
        }
        let result = RecordStore::from_toml_bytes(toml.as_bytes());
        let err = result.unwrap_err();
        assert!(
            err.contains("top records"),
            "expected 'top records' in error, got: {err}"
        );
    }

    #[test]
    fn rejects_too_many_hill_records() {
        let mut toml = String::from("format_version = 1\n");
        for i in 0..30 {
            toml.push_str("[[hill_records]]\n");
            toml.push_str(&format!("name = \"{i}\"\nlen = 100\ntime = \"\"\n"));
        }
        let result = RecordStore::from_toml_bytes(toml.as_bytes());
        let err = result.unwrap_err();
        assert!(
            err.contains("hill records"),
            "expected 'hill records' in error, got: {err}"
        );
    }

    #[test]
    fn rejects_invalid_toml() {
        let result = RecordStore::from_toml_bytes(b"not valid toml {{{");
        assert!(result.is_err());
    }

    #[test]
    fn loads_bundled_hiscores() {
        let store = bundled_store();
        assert!(!store.top.is_empty());
        assert!(store.top.len() <= 41);
        assert!(!store.hill_records.is_empty());
        assert!(store.hill_records.len() <= 20);

        let first = store.top(0).expect("first top record");
        assert!(!first.name.is_empty());
        assert!(first.pos > 0);
        assert!(first.score > 0);

        let first_hill = store.hill_record(0).expect("first hill record");
        assert!(!first_hill.name.is_empty());
        assert!(first_hill.len > 0);
    }
}

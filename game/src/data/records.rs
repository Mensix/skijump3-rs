use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Hiscore {
    pub name: String,
    pub pos: usize,
    pub score: f64,
    pub time: String,
    #[serde(default)]
    pub is_computer: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HillRecord {
    pub name: String,
    pub len: f64,
    pub time: String,
    #[serde(default)]
    pub is_computer: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordStore {
    #[serde(default)]
    pub top: Vec<Hiscore>,
    #[serde(default)]
    pub hill_records: BTreeMap<String, HillRecord>,
    #[serde(default)]
    pub hill_goals: BTreeMap<String, f64>,
}

impl RecordStore {
    pub fn bundled_default() -> Self {
        let data = include_bytes!("../../assets/hiscores.toml");
        Self::from_toml_bytes(data)
    }

    pub fn cleared_default() -> Self {
        let mut records = Self::bundled_default();
        for top in &mut records.top {
            top.pos = 0;
            top.score = 0.0;
        }
        for record in records.hill_records.values_mut() {
            record.len = 0.0;
        }
        records
    }

    pub fn top(&self, idx: usize) -> Option<&Hiscore> {
        self.top.get(idx)
    }

    /// Look up a hill record by its stable `record_key`.
    pub fn hill_record(&self, record_key: &str) -> Option<&HillRecord> {
        self.hill_records.get(record_key)
    }

    /// Set a hill record by its stable `record_key`.
    pub fn set_hill_record(&mut self, record_key: &str, record: HillRecord) {
        self.hill_records.insert(record_key.to_string(), record);
    }

    /// Look up a hill goal by its stable `record_key`.
    pub fn hill_goal(&self, record_key: &str) -> Option<&f64> {
        self.hill_goals.get(record_key)
    }

    /// Set a hill goal by its stable `record_key`.
    pub fn set_hill_goal(&mut self, record_key: &str, goal: f64) {
        self.hill_goals.insert(record_key.to_string(), goal);
    }
}

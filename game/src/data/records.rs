use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Hiscore {
    pub name: String,
    pub pos: usize,
    pub score: f64,
    pub time: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HillRecord {
    pub name: String,
    pub len: f64,
    pub time: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordStore {
    #[serde(default)]
    pub top: Vec<Hiscore>,
    #[serde(default)]
    pub hill_records: Vec<HillRecord>,
    #[serde(default)]
    pub hill_goals: Vec<f64>,
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
        for hill in &mut records.hill_records {
            hill.len = 0.0;
        }
        records
    }

    pub fn top(&self, idx: usize) -> Option<&Hiscore> {
        self.top.get(idx)
    }

    pub fn hill_record(&self, idx: usize) -> Option<&HillRecord> {
        self.hill_records.get(idx)
    }
}

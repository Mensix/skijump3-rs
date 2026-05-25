use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hiscore {
    pub name: String,
    pub pos: usize,
    pub score: i64,
    pub time: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HillRecord {
    pub name: String,
    pub len: i64,
    pub time: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordStore {
    pub top: Vec<Hiscore>,
    pub hill_records: Vec<HillRecord>,
}

impl RecordStore {
    #[must_use]
    pub const fn new(top: Vec<Hiscore>, hill_records: Vec<HillRecord>) -> Self {
        Self { top, hill_records }
    }

    #[must_use]
    pub fn top(&self, idx: usize) -> Option<&Hiscore> {
        self.top.get(idx)
    }

    #[must_use]
    pub fn hill_record(&self, idx: usize) -> Option<&HillRecord> {
        self.hill_records.get(idx)
    }
}

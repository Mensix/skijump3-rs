#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hiscore {
    pub name: String,
    pub pos: usize,
    pub score: i64,
    pub time: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HillRecord {
    pub name: String,
    pub len: i64,
    pub time: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordStore {
    top: Vec<Hiscore>,
    hill_records: Vec<HillRecord>,
}

impl RecordStore {
    #[must_use] 
    pub const fn new(top: Vec<Hiscore>, hill_records: Vec<HillRecord>) -> Self {
        Self { top, hill_records }
    }

    #[must_use] 
    pub fn top(&self, pascal_index: usize) -> Option<&Hiscore> {
        pascal_index
            .checked_sub(1)
            .and_then(|idx| self.top.get(idx))
    }

    #[must_use] 
    pub fn hill_record(&self, pascal_index: usize) -> Option<&HillRecord> {
        pascal_index
            .checked_sub(1)
            .and_then(|idx| self.hill_records.get(idx))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HillInfo {
    pub name: String,
    pub kr: i64,
    pub front_index: String,
    pub back_index: String,
    pub back_brightness: i64,
    pub back_mirror: i64,
    pub vx_final: i64,
    pub pk_hundred: i64,
    pub pl_save_ten_thousand: i64,
    pub author: String,
    pub checksum: i64,
    pub profile_checksum: i64,
}

impl HillInfo {
    #[must_use] 
    pub fn pk(&self) -> f64 {
        self.pk_hundred as f64 / 100.0
    }

    #[must_use] 
    pub fn pl_save(&self) -> f64 {
        self.pl_save_ten_thousand as f64 / 10_000.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HillCatalog {
    hills: Vec<HillInfo>,
}

impl HillCatalog {
    #[must_use] 
    pub const fn new(hills: Vec<HillInfo>) -> Self {
        Self { hills }
    }

    #[must_use] 
    pub const fn len(&self) -> usize {
        self.hills.len()
    }

    #[must_use] 
    pub const fn is_empty(&self) -> bool {
        self.hills.is_empty()
    }

    #[must_use] 
    pub fn hill(&self, pascal_index: usize) -> Option<&HillInfo> {
        pascal_index
            .checked_sub(1)
            .and_then(|idx| self.hills.get(idx))
    }
}

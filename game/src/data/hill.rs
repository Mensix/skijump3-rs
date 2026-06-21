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
    pub terrain_id: String,
}

impl HillInfo {
    pub fn pk(&self) -> f64 {
        self.pk_hundred as f64 / 100.0
    }

    pub fn pl_save(&self) -> f64 {
        self.pl_save_ten_thousand as f64 / 10_000.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HillCatalog {
    hills: Vec<HillInfo>,
    /// Number of built-in hills (before any user-created custom hills).
    original_count: usize,
}

impl HillCatalog {
    /// `original_count` is the number of built-in hills; hills beyond that
    /// are user-created custom hills and are excluded from competitions.
    pub const fn new(hills: Vec<HillInfo>, original_count: usize) -> Self {
        Self {
            hills,
            original_count,
        }
    }

    pub const fn len(&self) -> usize {
        self.hills.len()
    }

    /// Number of built-in hills (excludes custom hills).
    pub const fn original_count(&self) -> usize {
        self.original_count
    }

    pub fn hill(&self, idx: usize) -> Option<&HillInfo> {
        self.hills.get(idx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandingEntry {
    pub rank: usize,
    pub name: String,
    pub primary_score: i32,
    pub secondary_score: Option<i32>,
    pub is_human: bool,
}

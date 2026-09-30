#[derive(Debug, Clone, PartialEq)]
pub struct StandingEntry {
    pub rank: usize,
    pub name: String,
    pub primary_score: f64,
    pub secondary_score: Option<f64>,
    pub is_human: bool,
}

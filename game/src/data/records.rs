use crate::files::FileStore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Range;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Hiscore {
    pub name: String,
    pub pos: usize,
    pub score: f64,
    pub time: String,
    pub is_computer: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HillRecord {
    pub name: String,
    pub len: f64,
    pub time: String,
    pub is_computer: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordStore {
    pub top: Vec<Hiscore>,
    pub hill_records: BTreeMap<String, HillRecord>,
    pub hill_goals: BTreeMap<String, f64>,
}

pub(crate) fn insert_ranked_hiscore(
    records: &mut [Hiscore],
    range: Range<usize>,
    candidate: Hiscore,
) -> bool {
    let Some(table) = records.get_mut(range) else {
        return false;
    };
    if table.is_empty() {
        return false;
    }

    let previous = table
        .iter()
        .filter(|record| record.name == candidate.name)
        .max_by(|a, b| compare_hiscores(a, b))
        .cloned();
    let inserted = previous
        .as_ref()
        .is_none_or(|record| compare_hiscores(&candidate, record).is_gt());
    let entry = if inserted {
        candidate
    } else if let Some(previous) = previous {
        previous
    } else {
        return false;
    };

    let mut ranked: Vec<_> = table
        .iter()
        .filter(|record| record.name != entry.name)
        .cloned()
        .collect();
    let insert_at = ranked
        .iter()
        .position(|record| compare_hiscores(&entry, record).is_gt())
        .unwrap_or(ranked.len());

    if insert_at >= table.len() {
        return false;
    }

    ranked.insert(insert_at, entry);
    ranked.truncate(table.len());
    ranked.resize_with(table.len(), Hiscore::default);
    table.clone_from_slice(&ranked);
    inserted
}

fn compare_hiscores(a: &Hiscore, b: &Hiscore) -> std::cmp::Ordering {
    a.score
        .total_cmp(&b.score)
        .then_with(|| match (a.pos, b.pos) {
            (0, 0) => std::cmp::Ordering::Equal,
            (0, _) => std::cmp::Ordering::Less,
            (_, 0) => std::cmp::Ordering::Greater,
            _ => b.pos.cmp(&a.pos),
        })
}

impl RecordStore {
    pub fn bundled_default(files: &FileStore) -> Result<Self, String> {
        Self::from_toml_bytes(&files.read("hiscores.toml"))
    }

    pub fn cleared_default(files: &FileStore) -> Result<Self, String> {
        let mut records = Self::bundled_default(files)?;
        for top in &mut records.top {
            top.pos = 0;
            top.score = 0.0;
        }
        for record in records.hill_records.values_mut() {
            record.len = 0.0;
        }
        Ok(records)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn hiscore(name: &str, pos: usize, score: f64, time: &str) -> Hiscore {
        Hiscore {
            name: name.to_string(),
            pos,
            score,
            time: time.to_string(),
            is_computer: false,
        }
    }

    #[test]
    fn ranked_insertion_shifts_only_the_selected_table() {
        let mut records = vec![
            hiscore("outside", 9, 999.0, "outside"),
            hiscore("first", 1, 100.0, "old-1"),
            hiscore("second", 2, 80.0, "old-2"),
            hiscore("third", 3, 60.0, "old-3"),
            hiscore("after", 8, 1.0, "after"),
        ];

        insert_ranked_hiscore(&mut records, 1..4, hiscore("new", 7, 90.0, "new-date"));

        assert_eq!(records[0].name, "outside");
        assert_eq!(records[4].name, "after");
        assert_eq!(
            records[1..4]
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "new", "second"]
        );
        assert_eq!(records[2].pos, 7);
        assert_eq!(records[2].time, "new-date");
        assert_eq!(records[3].time, "old-2");
    }

    #[test]
    fn weaker_repeat_keeps_better_result_and_original_date_without_duplicate() {
        let mut records = vec![
            hiscore("player", 2, 100.0, "best-date"),
            hiscore("other", 1, 90.0, "other-date"),
            hiscore("player", 5, 70.0, "duplicate-date"),
        ];

        insert_ranked_hiscore(&mut records, 0..3, hiscore("player", 8, 80.0, "new-date"));

        let player: Vec<_> = records.iter().filter(|r| r.name == "player").collect();
        assert_eq!(player.len(), 1);
        assert_eq!(player[0].score, 100.0);
        assert_eq!(player[0].pos, 2);
        assert_eq!(player[0].time, "best-date");
    }

    #[test]
    fn stronger_repeat_replaces_old_result() {
        let mut records = vec![
            hiscore("leader", 1, 120.0, "leader-date"),
            hiscore("player", 4, 80.0, "old-date"),
            hiscore("last", 7, 20.0, "last-date"),
        ];

        insert_ranked_hiscore(&mut records, 0..3, hiscore("player", 2, 110.0, "new-date"));

        assert_eq!(
            records.iter().map(|r| r.score).collect::<Vec<_>>(),
            vec![120.0, 110.0, 20.0]
        );
        assert_eq!(records[1].pos, 2);
        assert_eq!(records[1].time, "new-date");
    }

    #[test]
    fn equal_score_prefers_better_placement_and_reports_insertion() {
        let mut records = vec![
            hiscore("leader", 2, 100.0, "old"),
            hiscore("other", 1, 90.0, "other"),
        ];

        assert!(insert_ranked_hiscore(
            &mut records,
            0..2,
            hiscore("player", 1, 100.0, "new")
        ));
        assert_eq!(records[0].name, "player");
        assert!(!insert_ranked_hiscore(
            &mut records,
            0..2,
            hiscore("player", 3, 100.0, "weaker")
        ));
        assert_eq!(records[0].pos, 1);
        assert_eq!(records[0].time, "new");
    }

    #[test]
    fn bundled_and_cleared_defaults_include_original_goals_by_stable_key() {
        use std::path::PathBuf;
        let files = FileStore::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let expected = [
            132.5, 107.0, 209.5, 136.0, 146.5, 109.5, 145.0, 107.5, 139.5, 137.0, 130.0, 144.5,
            217.0, 131.5, 81.5, 144.5, 108.5, 139.5, 226.5, 136.5,
        ];

        let bundled = RecordStore::bundled_default(&files).expect("bundled hiscores");
        let cleared = RecordStore::cleared_default(&files).expect("cleared hiscores");
        for store in [bundled, cleared] {
            assert_eq!(store.hill_goals.len(), expected.len());
            for (index, goal) in expected.iter().enumerate() {
                let key = format!("original:{}", char::from(b'A' + index as u8));
                assert_eq!(store.hill_goal(&key), Some(goal));
            }
        }
    }
}

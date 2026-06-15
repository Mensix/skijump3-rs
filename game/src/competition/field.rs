use crate::competition::core::ranking::ranked_order;
use crate::competition::types::{CompetitionPhase, Participant, QualificationStatus};

/// Criterion for sorting the participant list.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    WcPoints,
    FourHillsPoints,
    EventPoints,
}

/// The full competition field: 50 jumpers with ordering arrays.
///
/// Maintains two orderings independently:
/// - `master_order` — sorted by season points (World Cup / Four Hills)
/// - `event_order`  — sorted by current event points
#[derive(Debug, Clone)]
pub struct CompetitionField {
    participants: Vec<Participant>,
    pub master_order: Vec<usize>,
    pub event_order: Vec<usize>,
}

impl CompetitionField {
    #[must_use]
    pub fn new(participants: Vec<Participant>) -> Self {
        let count = participants.len();
        let master_order: Vec<usize> = (0..count).collect();
        let event_order: Vec<usize> = (0..count).collect();
        Self {
            participants,
            master_order,
            event_order,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.participants.len()
    }

    #[must_use]
    pub fn get(&self, idx: usize) -> &Participant {
        &self.participants[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Participant {
        &mut self.participants[idx]
    }

    pub fn iter(&self) -> impl Iterator<Item = &Participant> {
        self.participants.iter()
    }

    fn score(&self, by: SortBy, idx: usize) -> f64 {
        match by {
            SortBy::WcPoints => f64::from(self.participants[idx].wc_points),
            SortBy::FourHillsPoints => self.participants[idx].four_hills_points,
            SortBy::EventPoints => self.participants[idx].points.unwrap_or(f64::NEG_INFINITY),
        }
    }

    /// Sort participants by score descending into the appropriate ordering array.
    pub fn sort_field(&mut self, by: SortBy) {
        let ranked = ranked_order(0..self.participants.len(), |i| self.score(by, i));
        let order: Vec<usize> = ranked.iter().map(|r| r.item).collect();

        let target = match by {
            SortBy::WcPoints | SortBy::FourHillsPoints => &mut self.master_order,
            SortBy::EventPoints => &mut self.event_order,
        };
        *target = order;
        for ranked in ranked {
            self.participants[ranked.item].rank = ranked.rank;
        }
    }

    /// Build an ordered start list for the given phase.
    ///
    /// - **Qualification**: reverse `master_order`, only non-injured,
    ///   skipping `PreQualified`.
    /// - **Round 1**: reverse `event_order`, only qualified.
    /// - **Round 2**: reverse `event_order`, only qualified.
    /// - **Training**: reverse `master_order`, only non-injured.
    #[must_use]
    pub fn build_start_list(&self, phase: CompetitionPhase) -> Vec<usize> {
        match phase {
            CompetitionPhase::Training(_) => self
                .master_order
                .iter()
                .rev()
                .copied()
                .filter(|&idx| self.participants[idx].injury == 0)
                .collect(),

            CompetitionPhase::Qualification => self
                .master_order
                .iter()
                .rev()
                .copied()
                .filter(|&idx| self.participants[idx].injury == 0)
                .collect(),

            CompetitionPhase::Round1
                if self
                    .participants
                    .iter()
                    .any(|p| matches!(p.qual, QualificationStatus::KoSeed(_))) =>
            {
                let mut seeded: Vec<_> = self
                    .participants
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, p)| match p.qual {
                        QualificationStatus::KoSeed(seed) if p.injury == 0 => Some((seed, idx)),
                        _ => None,
                    })
                    .collect();
                seeded.sort_by_key(|&(seed, _)| seed);
                let count = seeded.len().min(50);
                let mut list = Vec::with_capacity(count);
                for pair in (1..=count / 2).rev() {
                    list.push(seeded[count - pair].1);
                    list.push(seeded[pair - 1].1);
                }
                list
            }

            CompetitionPhase::Round1 => self
                .master_order
                .iter()
                .rev()
                .copied()
                .filter(|&idx| {
                    self.participants[idx].injury == 0 && self.participants[idx].qual.can_jump()
                })
                .collect(),

            CompetitionPhase::Round2 => self
                .event_order
                .iter()
                .rev()
                .copied()
                .filter(|&idx| {
                    self.participants[idx].injury == 0 && self.participants[idx].qual.can_jump()
                })
                .collect(),

            _ => Vec::new(),
        }
    }

    /// Reset per-event state for all participants.
    pub fn reset_event(&mut self) {
        for p in &mut self.participants {
            p.reset_event();
        }
    }

    /// Decrement injury counters. Called at the start of each event.
    pub fn tick_injuries(&mut self) {
        for p in &mut self.participants {
            if p.injury > 0 {
                p.injury -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_from_wc(scores: &[i32]) -> Vec<Participant> {
        scores
            .iter()
            .enumerate()
            .map(|(i, &s)| Participant {
                wc_points: s,
                ..Participant::computer(i, i, format!("Jumper {i}"))
            })
            .collect()
    }

    #[test]
    fn new_creates_identity_order() {
        let f = CompetitionField::new(make_from_wc(&[100, 80, 60]));
        assert_eq!(f.len(), 3);
        assert_eq!(f.master_order, vec![0, 1, 2]);
    }

    #[test]
    fn sort_field_by_wc_points_descending() {
        let mut f = CompetitionField::new(make_from_wc(&[10, 50, 30, 80]));
        f.sort_field(SortBy::WcPoints);
        assert_eq!(f.master_order, vec![3, 1, 2, 0]);
        assert_eq!(f.get(3).rank, 1);
        assert_eq!(f.get(0).rank, 4);
    }

    #[test]
    fn ties_get_same_rank() {
        let mut f = CompetitionField::new(make_from_wc(&[50, 50, 30]));
        f.sort_field(SortBy::WcPoints);
        assert_eq!(f.master_order, vec![0, 1, 2]);
        assert_eq!(f.get(0).rank, 1);
        assert_eq!(f.get(1).rank, 1);
        assert_eq!(f.get(2).rank, 3);
    }

    #[test]
    fn event_order_separate_from_master() {
        let mut participants = make_from_wc(&[100, 0]);
        participants[0].points = Some(10.0);
        participants[1].points = Some(999.0);
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);
        assert_eq!(f.master_order, vec![0, 1], "master by wc");
        f.sort_field(SortBy::EventPoints);
        assert_eq!(f.event_order, vec![1, 0], "event by points");
    }

    #[test]
    fn qualification_includes_all_non_injured() {
        let mut participants = make_from_wc(&[100, 90, 80, 70, 60]);
        participants[0].qual = QualificationStatus::PreQualified;
        participants[1].qual = QualificationStatus::PreQualified;
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);

        let list = f.build_start_list(CompetitionPhase::Qualification);
        // PreQualified are included; Pascal still runs hyppy for AI pre-qualified.
        assert_eq!(list, vec![4, 3, 2, 1, 0]);
    }

    #[test]
    fn round1_uses_master_order_qualified_only() {
        let mut participants = make_from_wc(&[50, 40, 30, 20, 10]);
        participants[3].qual = QualificationStatus::PreQualified;
        participants[4].qual = QualificationStatus::Qualified;
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);

        let list = f.build_start_list(CompetitionPhase::Round1);
        // master_order = [0,1,2,3,4] (wc_points sorted), reversed = [4,3,2,1,0]
        // filter by can_jump: 4(Qualified) and 3(PreQualified) can jump
        assert_eq!(list, vec![4, 3]);
    }

    #[test]
    fn tick_injuries_decrements_all() {
        let mut f = CompetitionField::new(make_from_wc(&[0; 3]));
        f.participants[0].injury = 2;
        f.participants[1].injury = 1;
        f.tick_injuries();
        assert_eq!(f.participants[0].injury, 1);
        assert_eq!(f.participants[1].injury, 0);
        f.tick_injuries();
        assert_eq!(f.participants[0].injury, 0);
    }

    #[test]
    fn reset_event_zeroes_all() {
        let mut f = CompetitionField::new(make_from_wc(&[0; 2]));
        f.participants[0].points = Some(500.0);
        f.participants[0].qual = QualificationStatus::Qualified;
        f.reset_event();
        assert_eq!(f.participants[0].points, None);
        assert_eq!(f.participants[0].qual, QualificationStatus::NotQualified);
    }

    #[test]
    fn training_start_list_all_active() {
        let mut participants = make_from_wc(&[100, 90, 80]);
        participants[1].injury = 1;
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);

        let list = f.build_start_list(CompetitionPhase::Training(1));
        assert_eq!(list, vec![2, 0]);
    }

    #[test]
    fn qualification_start_order_reverse_wc_rank() {
        let wc_scores: Vec<i32> = (0..75).map(|i| 100 - i).collect();
        let mut participants = make_from_wc(&wc_scores);
        let human_idx = 9;
        participants[human_idx].is_computer = false;

        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);

        let list = f.build_start_list(CompetitionPhase::Qualification);
        assert_eq!(list[65], human_idx, "human 10th WC at position 65");
        for (i, &idx) in list[..65].iter().enumerate() {
            assert!(f.get(idx).is_computer, "pos {i} should be AI");
        }
    }

    #[test]
    fn qualification_starts_with_lowest_wc() {
        let participants = make_from_wc(&[50, 100, 30, 80, 10]);
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);
        let list = f.build_start_list(CompetitionPhase::Qualification);
        assert_eq!(list[0], 4, "lowest WC first");
        assert_eq!(list[4], 1, "highest WC last");
    }
}

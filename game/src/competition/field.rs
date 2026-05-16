use crate::competition::types::{CompetitionPhase, Participant, QualificationStatus};

/// Criterion for sorting the participant list.
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
pub struct CompetitionField {
    participants: Vec<Participant>,
    pub master_order: Vec<usize>,
    pub event_order: Vec<usize>,
}

impl CompetitionField {
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

    pub fn len(&self) -> usize {
        self.participants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.participants.is_empty()
    }

    pub fn get(&self, idx: usize) -> &Participant {
        &self.participants[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Participant {
        &mut self.participants[idx]
    }

    pub fn iter(&self) -> impl Iterator<Item = &Participant> {
        self.participants.iter()
    }

    fn score(&self, by: SortBy, idx: usize) -> i32 {
        match by {
            SortBy::WcPoints => self.participants[idx].wc_points,
            SortBy::FourHillsPoints => self.participants[idx].four_hills_points,
            SortBy::EventPoints => self.participants[idx].points,
        }
    }

    /// Sort participants by score descending into the appropriate ordering array.
    pub fn sort_field(&mut self, by: SortBy) {
        let scores: Vec<i32> = (0..self.participants.len())
            .map(|i| self.score(by, i))
            .collect();

        let mut order: Vec<usize> = (0..self.participants.len()).collect();
        order.sort_by(|&a, &b| scores[b].cmp(&scores[a]));

        let target = match by {
            SortBy::WcPoints | SortBy::FourHillsPoints => &mut self.master_order,
            SortBy::EventPoints => &mut self.event_order,
        };
        *target = order;
        self.calculate_ranks_from(&scores);
    }

    fn calculate_ranks_from(&mut self, scores: &[i32]) {
        let order = &self.master_order;
        let mut rank = 1;
        for i in 0..order.len() {
            if i > 0 && scores[order[i]] < scores[order[i - 1]] {
                rank = i + 1;
            }
            self.participants[order[i]].rank = rank;
        }
    }

    /// Number of non-injured participants.
    pub fn num_active(&self) -> usize {
        self.participants.iter().filter(|p| p.injury == 0).count()
    }

    /// How many have `QualificationStatus` that lets them jump the current round.
    pub fn num_qualified(&self) -> usize {
        self.participants
            .iter()
            .filter(|p| p.qual.can_jump())
            .count()
    }

    /// Build an ordered start list for the given phase.
    ///
    /// - **Qualification**: reverse `master_order`, only non-injured,
    ///   skipping PreQualified.
    /// - **Round 1**: reverse `event_order`, only qualified.
    /// - **Round 2**: reverse `event_order`, only qualified.
    /// - **Training**: reverse `master_order`, only non-injured.
    pub fn build_start_list(&self, phase: CompetitionPhase) -> Vec<usize> {
        match phase {
            CompetitionPhase::Training(_) | CompetitionPhase::Setup => self
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
                .filter(|&idx| {
                    self.participants[idx].injury == 0
                        && self.participants[idx].qual
                            != QualificationStatus::PreQualified
                })
                .collect(),

            CompetitionPhase::Round1 | CompetitionPhase::Round2 => self
                .event_order
                .iter()
                .rev()
                .copied()
                .filter(|&idx| {
                    self.participants[idx].injury == 0
                        && self.participants[idx].qual.can_jump()
                })
                .collect(),

            CompetitionPhase::Results
            | CompetitionPhase::EventComplete
            | CompetitionPhase::SeasonComplete => Vec::new(),
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
                ..Participant::computer(i, format!("Jumper {i}"))
            })
            .collect()
    }

    fn make_from_event(scores: &[i32]) -> Vec<Participant> {
        scores
            .iter()
            .enumerate()
            .map(|(i, &s)| Participant {
                points: s,
                ..Participant::computer(i, format!("Jumper {i}"))
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
        participants[0].points = 10;
        participants[1].points = 999;
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);
        assert_eq!(f.master_order, vec![0, 1], "master by wc");
        f.sort_field(SortBy::EventPoints);
        assert_eq!(f.event_order, vec![1, 0], "event by points");
    }

    #[test]
    fn qualification_skips_pre_qualified() {
        let mut participants = make_from_wc(&[100, 90, 80, 70, 60]);
        participants[0].qual = QualificationStatus::PreQualified;
        participants[1].qual = QualificationStatus::PreQualified;
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::WcPoints);

        let list = f.build_start_list(CompetitionPhase::Qualification);
        assert_eq!(list, vec![4, 3, 2]);
    }

    #[test]
    fn round1_uses_event_order_qualified_only() {
        let mut participants = make_from_event(&[0; 5]);
        participants[0].points = 300;
        participants[1].points = 200;
        participants[2].points = 100;
        participants[3].qual = QualificationStatus::PreQualified;
        participants[4].qual = QualificationStatus::Qualified;
        let mut f = CompetitionField::new(participants);
        f.sort_field(SortBy::EventPoints);

        let list = f.build_start_list(CompetitionPhase::Round1);
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
        f.participants[0].points = 500;
        f.participants[0].qual = QualificationStatus::Qualified;
        f.reset_event();
        assert_eq!(f.participants[0].points, 0);
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
}

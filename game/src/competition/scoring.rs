use crate::competition::core::scoring::PointsTable;
use crate::competition::field::CompetitionField;

/// World Cup points awarded to top 30 finishers (1-indexed: position 1 → 100 pts).
pub const WC_POINTS: PointsTable<30> = PointsTable::new([
    100, 80, 60, 50, 45, 40, 36, 32, 29, 26, 24, 22, 20, 18, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7,
    6, 5, 4, 3, 2, 1,
]);

/// Look up WC points awarded for a given rank (1-indexed).
/// Rank 1 → 100, rank 30 → 1, out-of-range → 0.
#[must_use]
pub fn wc_points_for_rank(rank: usize) -> i32 {
    WC_POINTS.points_for_rank(rank)
}

/// Add World Cup points to each participant based on their rank in the current event.
///
/// Only ranks 1..=30 receive points. Rank 1 gets 100, rank 2 gets 80, ..., rank 30 gets 1.
/// Points are added to `participant.wc_points`.
pub fn award_wc_points(field: &mut CompetitionField) {
    for idx in 0..field.len() {
        let rank = field.get(idx).rank;
        let pts = wc_points_for_rank(rank);
        field.get_mut(idx).wc_points += pts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::field::SortBy;
    use crate::competition::types::Participant;

    fn make_field(scores: &[i32]) -> CompetitionField {
        let participants: Vec<Participant> = scores
            .iter()
            .enumerate()
            .map(|(i, &s)| Participant {
                points: Some(s),
                ..Participant::computer(i, i, format!("J {i}"))
            })
            .collect();
        CompetitionField::new(participants)
    }

    #[test]
    fn wc_points_distribution() {
        let scores = [1000, 900, 800, 700, 600, 500, 400, 300, 200, 100];
        let mut field = make_field(&scores);
        field.sort_field(SortBy::EventPoints);

        let top3: Vec<Option<i32>> = (0..3).map(|i| field.get(i).points).collect();
        assert_eq!(
            top3,
            [Some(1000), Some(900), Some(800)],
            "sorted descending"
        );

        award_wc_points(&mut field);
        assert_eq!(field.get(0).wc_points, 100);
        assert_eq!(field.get(1).wc_points, 80);
        assert_eq!(field.get(2).wc_points, 60);
        assert_eq!(field.get(9).wc_points, 26); // rank 10
    }

    #[test]
    fn beyond_30_get_no_points() {
        let mut field = make_field(&[0i32; 35]);
        for i in 0..35 {
            field.get_mut(i).points = Some((35 - i) as i32);
        }
        field.sort_field(SortBy::EventPoints);
        award_wc_points(&mut field);

        assert_eq!(field.get(29).wc_points, 1); // rank 30 → 1 point
        assert_eq!(field.get(30).wc_points, 0); // rank 31 → 0
        assert_eq!(field.get(34).wc_points, 0); // rank 35 → 0
    }

    #[test]
    fn table_has_30_entries() {
        assert_eq!(WC_POINTS.len(), 30);
        let points = WC_POINTS.points();
        assert_eq!(points[0], 100);
        assert_eq!(points[29], 1);
    }

    #[test]
    fn wc_points_for_rank_returns_correct_values() {
        assert_eq!(wc_points_for_rank(1), 100);
        assert_eq!(wc_points_for_rank(2), 80);
        assert_eq!(wc_points_for_rank(30), 1);
    }

    #[test]
    fn wc_points_for_rank_out_of_range_returns_zero() {
        assert_eq!(wc_points_for_rank(0), 0);
        assert_eq!(wc_points_for_rank(31), 0);
        assert_eq!(wc_points_for_rank(99), 0);
    }
}

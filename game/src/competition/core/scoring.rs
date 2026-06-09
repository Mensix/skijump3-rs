#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointsTable<const N: usize> {
    points: [i32; N],
}

impl<const N: usize> PointsTable<N> {
    #[must_use]
    pub const fn new(points: [i32; N]) -> Self {
        Self { points }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        N
    }

    #[must_use]
    pub const fn points(&self) -> [i32; N] {
        self.points
    }

    #[must_use]
    pub fn points_for_rank(&self, rank: usize) -> i32 {
        if (1..=N).contains(&rank) {
            self.points[rank - 1]
        } else {
            0
        }
    }
}

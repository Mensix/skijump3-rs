use crate::competition::core::ranking::ranked_order;
use crate::competition::core::scoring::PointsTable;
use crate::competition::team_cup::types::TeamCupTeam;

pub const TEAM_LEG_POINTS: PointsTable<8> =
    PointsTable::new([400, 350, 300, 250, 200, 150, 100, 50]);

pub fn team_points_for_rank(rank: usize) -> i32 {
    TEAM_LEG_POINTS.points_for_rank(rank)
}

pub fn calculate_team_leg_score(team: &TeamCupTeam, leg_idx: usize) -> f64 {
    team.members
        .iter()
        .flat_map(|m| m.jumps.iter())
        .filter(|j| j.leg == leg_idx)
        .map(|j| j.score)
        .sum()
}

pub fn award_leg_points(teams: &mut [TeamCupTeam], leg_idx: usize) {
    for ranked in ranked_order(0..teams.len(), |idx| {
        calculate_team_leg_score(&teams[idx], leg_idx)
    }) {
        let pts = team_points_for_rank(ranked.rank);
        teams[ranked.item].cup_points += pts;
    }
}

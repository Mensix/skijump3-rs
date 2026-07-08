pub(crate) fn scoreboard_leg_header(
    leg: usize,
    total_legs: usize,
    hill: &str,
    round: usize,
) -> String {
    format!(
        "SCOREBOARD - LEG {} OF {} - {} - R{}",
        leg, total_legs, hill, round
    )
}

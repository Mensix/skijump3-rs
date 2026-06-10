use super::scoring::{award_leg_points, calculate_team_leg_score};
use super::types::{
    TeamCupJumpContext, TeamCupPhase, TeamCupResultsKind, TeamCupRuntime, TeamCupStandingsKind,
    TeamCupTeam, MEMBERS_PER_TEAM, NUM_LEGS,
};
use crate::competition::core::ranking::ranked_order;
use crate::competition::core::standings::StandingEntry;
use crate::competition::runtime::{CompetitionDecision, CompetitionRuntime};
use crate::jump::types::{JumpOutcome, DEFAULT_START_GATE};

impl TeamCupRuntime {
    pub fn new(
        teams: Vec<TeamCupTeam>,
        schedule: Vec<usize>,
        human_teams: usize,
        team_order: Vec<usize>,
    ) -> Self {
        Self {
            teams,
            schedule,
            current_leg: 0,
            current_round: 0,
            current_jumper_slot: 0,
            current_team_order_pos: 0,
            team_order,
            phase: TeamCupPhase::Setup,
            human_teams,
        }
    }

    pub fn decide_next(&mut self) -> CompetitionDecision<TeamCupJumpContext, TeamCupResultsKind> {
        loop {
            match self.phase {
                TeamCupPhase::Setup => {
                    self.phase = TeamCupPhase::Jumping;
                }

                TeamCupPhase::Jumping => {
                    if self.current_leg >= NUM_LEGS {
                        self.phase = TeamCupPhase::Complete;
                        continue;
                    }
                    if self.current_round >= 2 {
                        self.phase = TeamCupPhase::LegResults;
                        continue;
                    }
                    if self.current_jumper_slot >= MEMBERS_PER_TEAM {
                        self.current_jumper_slot = 0;
                        self.current_round += 1;
                        continue;
                    }
                    if self.current_team_order_pos >= self.team_order.len() {
                        self.current_team_order_pos = 0;
                        self.current_jumper_slot += 1;
                        continue;
                    }

                    let team_idx = self.team_order[self.current_team_order_pos];
                    let member = &self.teams[team_idx].members[self.current_jumper_slot];
                    let hill_idx = self.schedule[self.current_leg];
                    let is_human = self.teams[team_idx].is_human_team;
                    let is_new_event = self.current_round == 0
                        && self.current_jumper_slot == 0
                        && self.current_team_order_pos == 0;

                    let participant = member.to_jump_participant();

                    return CompetitionDecision::Jump {
                        participant,
                        hill_idx,
                        context: TeamCupJumpContext {
                            leg_idx: self.current_leg,
                            round_idx: self.current_round,
                            team_idx,
                            member_idx: self.current_jumper_slot,
                            team_name: self.teams[team_idx].name.clone(),
                            jumper_name: member.competitor.name.clone(),
                            jumper_in_team: self.current_jumper_slot + 1,
                        },
                        is_human,
                        is_new_event,
                    };
                }

                TeamCupPhase::LegResults => {
                    return CompetitionDecision::ShowResults(TeamCupResultsKind::LegResults);
                }

                TeamCupPhase::TeamCupStandings => {
                    return CompetitionDecision::ShowResults(TeamCupResultsKind::Standings);
                }

                TeamCupPhase::Complete => {
                    return CompetitionDecision::Done;
                }
            }
        }
    }

    pub fn record_jump(&mut self, distance: i32, score: i32, gate: u8) {
        let team_idx = self.team_order[self.current_team_order_pos];
        let member = &mut self.teams[team_idx].members[self.current_jumper_slot];

        member.jumps.push(super::types::TeamCupJumpResult {
            leg: self.current_leg,
            round: self.current_round,
            distance,
            score,
            gate,
        });

        self.teams[team_idx].leg_score =
            calculate_team_leg_score(&self.teams[team_idx], self.current_leg);

        self.current_team_order_pos += 1;
    }

    pub fn advance_after_results(&mut self) {
        match self.phase {
            TeamCupPhase::LegResults => {
                award_leg_points(&mut self.teams, self.current_leg);
                self.phase = TeamCupPhase::TeamCupStandings;
            }
            TeamCupPhase::TeamCupStandings => {
                for team in &mut self.teams {
                    team.leg_score = 0;
                }
                self.current_leg += 1;
                self.current_round = 0;
                self.current_jumper_slot = 0;
                self.current_team_order_pos = 0;
                self.phase = TeamCupPhase::Jumping;
            }
            TeamCupPhase::Complete => {}
            _ => {}
        }
    }

    pub fn current_leg_standings(&self) -> Vec<StandingEntry> {
        ranked_order(0..self.teams.len(), |team_idx| {
            self.teams[team_idx].leg_score
        })
        .into_iter()
        .map(|ranked| {
            let team_idx = ranked.item;
            StandingEntry {
                rank: ranked.rank,
                name: self.teams[team_idx].name.clone(),
                primary_score: self.teams[team_idx].leg_score,
                secondary_score: Some(self.teams[team_idx].cup_points),
                is_human: self.teams[team_idx].is_human_team,
            }
        })
        .collect()
    }

    pub fn overall_standings(&self) -> Vec<StandingEntry> {
        ranked_order(0..self.teams.len(), |team_idx| {
            self.teams[team_idx].cup_points
        })
        .into_iter()
        .map(|ranked| {
            let team_idx = ranked.item;
            StandingEntry {
                rank: ranked.rank,
                name: self.teams[team_idx].name.clone(),
                primary_score: self.teams[team_idx].cup_points,
                secondary_score: None,
                is_human: self.teams[team_idx].is_human_team,
            }
        })
        .collect()
    }

    pub fn is_human_current(&self) -> bool {
        if self.current_team_order_pos >= self.team_order.len() {
            return false;
        }
        let team_idx = self.team_order[self.current_team_order_pos];
        self.teams.get(team_idx).is_some_and(|t| t.is_human_team)
    }

    pub fn current_jump_context(&self) -> TeamCupJumpContext {
        let team_idx = if self.current_team_order_pos < self.team_order.len() {
            self.team_order[self.current_team_order_pos]
        } else {
            0
        };
        let member = &self.teams[team_idx].members[self.current_jumper_slot];
        TeamCupJumpContext {
            leg_idx: self.current_leg,
            round_idx: self.current_round,
            team_idx,
            member_idx: self.current_jumper_slot,
            team_name: self.teams[team_idx].name.clone(),
            jumper_name: member.competitor.name.clone(),
            jumper_in_team: self.current_jumper_slot + 1,
        }
    }

    pub fn current_hill_idx(&self) -> usize {
        if self.current_leg < self.schedule.len() {
            self.schedule[self.current_leg]
        } else {
            0
        }
    }
}

impl CompetitionRuntime for TeamCupRuntime {
    type Context = TeamCupJumpContext;
    type ResultsKind = TeamCupResultsKind;
    type StandingsKind = TeamCupStandingsKind;

    fn decide_next_runtime(&mut self) -> CompetitionDecision<Self::Context, Self::ResultsKind> {
        self.decide_next()
    }

    fn record_jump_runtime(&mut self, _context: &Self::Context, outcome: JumpOutcome) {
        self.record_jump(outcome.distance, outcome.score, DEFAULT_START_GATE as u8);
    }

    fn advance_results_runtime(&mut self, _kind: Self::ResultsKind) {
        self.advance_after_results();
    }

    fn is_complete_runtime(&self) -> bool {
        self.phase == TeamCupPhase::Complete
    }

    fn standings_runtime(&self, kind: Self::StandingsKind) -> Vec<StandingEntry> {
        match kind {
            TeamCupStandingsKind::Leg => self.current_leg_standings(),
            TeamCupStandingsKind::Overall => self.overall_standings(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::core::competitor::Competitor;
    use crate::competition::team_cup::types::TeamCupMember;

    fn make_member(name: &str, is_computer: bool, ai_id: usize) -> TeamCupMember {
        TeamCupMember {
            competitor: if is_computer {
                Competitor::computer(ai_id, ai_id, name.to_string(), None)
            } else {
                Competitor {
                    id: ai_id,
                    ai_id,
                    name: name.to_string(),
                    real_name: String::new(),
                    suit_color: 0,
                    ski_color: 0,
                    team: None,
                    is_computer: false,
                    profile_idx: None,
                }
            },
            jumps: Vec::new(),
        }
    }

    fn make_team(id: usize, name: &str, is_human: bool, member_names: &[&str]) -> TeamCupTeam {
        let members: Vec<TeamCupMember> = member_names
            .iter()
            .enumerate()
            .map(|(i, n)| make_member(n, !is_human || i > 0, id * 4 + i))
            .collect();
        TeamCupTeam {
            id,
            name: name.to_string(),
            members,
            leg_score: 0,
            cup_points: 0,
            is_human_team: is_human,
        }
    }

    fn make_test_runtime() -> TeamCupRuntime {
        let names: Vec<String> = (0..60).map(|i| format!("J{i}")).collect();
        let teams: Vec<TeamCupTeam> = (0..15)
            .map(|i| {
                let m: Vec<&str> = (0..4).map(|j| names[i * 4 + j].as_str()).collect();
                make_team(i, &format!("Team{i}"), i < 2, &m)
            })
            .collect();
        let schedule = vec![0, 1, 2, 3, 4, 5];
        let team_order: Vec<usize> = (0..teams.len()).collect();
        TeamCupRuntime::new(teams, schedule, 2, team_order)
    }

    fn simulate_leg(runtime: &mut TeamCupRuntime, leg_base_score: i32) {
        for _ in 0..120 {
            match runtime.decide_next() {
                CompetitionDecision::Jump { .. } => runtime.record_jump(100, leg_base_score, 15),
                other => panic!("expected Jump during simulation, got {other:?}"),
            }
        }
        match runtime.decide_next() {
            CompetitionDecision::ShowResults(TeamCupResultsKind::LegResults) => {}
            other => panic!("expected ShowLegResults, got {other:?}"),
        }
        runtime.advance_after_results();
        match runtime.decide_next() {
            CompetitionDecision::ShowResults(TeamCupResultsKind::Standings) => {}
            other => panic!("expected ShowTeamCupStandings, got {other:?}"),
        }
        runtime.advance_after_results();
    }

    fn simulate_all_jumps(runtime: &mut TeamCupRuntime) {
        for leg in 0..6 {
            simulate_leg(runtime, 200 + leg as i32);
        }
        match runtime.decide_next() {
            CompetitionDecision::Done => {}
            other => panic!("expected Done, got {other:?}"),
        }
    }

    #[test]
    fn starts_with_jump_decision() {
        let mut r = make_test_runtime();
        let d = r.decide_next();
        match d {
            CompetitionDecision::Jump {
                hill_idx,
                is_human,
                is_new_event,
                ..
            } => {
                assert_eq!(hill_idx, 0);
                assert!(is_human);
                assert!(is_new_event);
            }
            _ => panic!("expected Jump, got {d:?}"),
        }
    }

    #[test]
    fn first_jumper_is_team0_member0_in_human_team() {
        let mut r = make_test_runtime();
        let d = r.decide_next();
        match d {
            CompetitionDecision::Jump {
                context, is_human, ..
            } => {
                assert_eq!(context.team_idx, 0);
                assert_eq!(context.member_idx, 0);
                assert!(is_human);
                assert_eq!(context.jumper_in_team, 1);
            }
            _ => panic!("expected Jump"),
        }
    }

    #[test]
    fn all_15_teams_jump_before_repeating_jumper_slot() {
        let mut r = make_test_runtime();
        let mut teams_seen = Vec::new();
        for _ in 0..15 {
            match r.decide_next() {
                CompetitionDecision::Jump { context, .. } => {
                    let key = (context.team_idx, context.member_idx, context.round_idx);
                    teams_seen.push(key);
                    r.record_jump(100, 200, 15);
                }
                _ => panic!("expected Jump"),
            }
        }
        assert_eq!(teams_seen.len(), 15);
        assert_eq!(teams_seen[0], (0, 0, 0));
        assert_eq!(teams_seen[14], (14, 0, 0));
    }

    #[test]
    fn after_15_jumps_next_team_member_is_used() {
        let mut r = make_test_runtime();
        for _ in 0..15 {
            match r.decide_next() {
                CompetitionDecision::Jump { .. } => r.record_jump(100, 200, 15),
                _ => panic!("expected Jump"),
            }
        }
        // Next jumper should be team 0, member 1 (second jumper slot)
        let d = r.decide_next();
        match d {
            CompetitionDecision::Jump { context, .. } => {
                assert_eq!(context.team_idx, 0);
                assert_eq!(context.member_idx, 1);
            }
            _ => panic!("expected Jump, got {d:?}"),
        }
    }

    #[test]
    fn leg_completes_after_120_jumps() {
        let mut r = make_test_runtime();
        // 15 teams × 4 members × 2 rounds = 120 jumps
        for _ in 0..120 {
            match r.decide_next() {
                CompetitionDecision::Jump { .. } => r.record_jump(100, 200, 15),
                other => panic!("expected Jump, got {other:?}"),
            }
        }
        let d = r.decide_next();
        assert_eq!(
            d,
            CompetitionDecision::ShowResults(TeamCupResultsKind::LegResults)
        );
    }

    #[test]
    fn round_transition_after_60_jumps() {
        let mut r = make_test_runtime();
        // 15 teams × 4 members = 60 jumps in round 0
        for _ in 0..60 {
            match r.decide_next() {
                CompetitionDecision::Jump { context, .. } => {
                    assert_eq!(context.round_idx, 0);
                    r.record_jump(100, 200, 15);
                }
                other => panic!("expected Jump, got {other:?}"),
            }
        }
        // Now round 1 should start
        match r.decide_next() {
            CompetitionDecision::Jump { context, .. } => {
                assert_eq!(context.round_idx, 1);
                assert_eq!(context.team_idx, 0);
                assert_eq!(context.member_idx, 0);
            }
            other => panic!("expected Jump for round 1, got {other:?}"),
        }
    }

    #[test]
    fn leg_results_advances_to_team_cup_standings() {
        let mut r = make_test_runtime();
        for _ in 0..120 {
            match r.decide_next() {
                CompetitionDecision::Jump { .. } => r.record_jump(100, 200, 15),
                other => panic!("expected Jump, got {other:?}"),
            }
        }
        assert_eq!(
            r.decide_next(),
            CompetitionDecision::ShowResults(TeamCupResultsKind::LegResults)
        );
        r.advance_after_results();
        assert_eq!(
            r.decide_next(),
            CompetitionDecision::ShowResults(TeamCupResultsKind::Standings)
        );
    }

    #[test]
    fn team_cup_standings_advances_to_next_leg() {
        let mut r = make_test_runtime();
        for _ in 0..120 {
            match r.decide_next() {
                CompetitionDecision::Jump { .. } => r.record_jump(100, 200, 15),
                other => panic!("expected Jump, got {other:?}"),
            }
        }
        match r.decide_next() {
            CompetitionDecision::ShowResults(TeamCupResultsKind::LegResults) => {}
            other => panic!("expected ShowLegResults, got {other:?}"),
        }
        r.advance_after_results();
        match r.decide_next() {
            CompetitionDecision::ShowResults(TeamCupResultsKind::Standings) => {}
            other => panic!("expected ShowTeamCupStandings, got {other:?}"),
        }
        r.advance_after_results();
        assert_eq!(r.current_leg, 1);
    }

    #[test]
    fn completes_after_6_legs() {
        let mut r = make_test_runtime();
        for leg in 0..6 {
            simulate_leg(&mut r, 200 + leg);
        }
        assert_eq!(r.decide_next(), CompetitionDecision::Done);
    }

    #[test]
    fn leg_score_is_sum_of_8_scores() {
        let mut r = make_test_runtime();
        for _ in 0..120 {
            match r.decide_next() {
                CompetitionDecision::Jump { context, .. } => {
                    let member_idx = context.member_idx;
                    let round = context.round_idx;
                    // Each jumper scores 100 + member_idx*10 + round*5
                    let score = 100 + member_idx as i32 * 10 + round as i32 * 5;
                    r.record_jump(100, score, 15);
                }
                other => panic!("expected Jump, got {other:?}"),
            }
        }
        // Team 0 leg score: members 0..3 each jump rounds 0,1
        // member 0: round0=100, round1=105  = 205
        // member 1: round0=110, round1=115  = 225
        // member 2: round0=120, round1=125  = 245
        // member 3: round0=130, round1=135  = 265
        // total = 205+225+245+265 = 940
        assert_eq!(r.teams[0].leg_score, 940);
    }

    #[test]
    fn top_8_teams_receive_cup_points() {
        let mut r = make_test_runtime();
        // Team N scores higher per leg: team 0 = worst, team 14 = best
        for _ in 0..120 {
            match r.decide_next() {
                CompetitionDecision::Jump { context, .. } => {
                    let score = context.team_idx as i32 * 10 + 100;
                    r.record_jump(100, score, 15);
                }
                other => panic!("expected Jump, got {other:?}"),
            }
        }
        // Consume LegResults and advance to TeamCupStandings (awards points)
        match r.decide_next() {
            CompetitionDecision::ShowResults(TeamCupResultsKind::LegResults) => {}
            other => panic!("expected ShowLegResults, got {other:?}"),
        }
        r.advance_after_results();
        assert_eq!(r.teams[14].cup_points, 400); // 1st place
        assert_eq!(r.teams[13].cup_points, 350); // 2nd place
        assert_eq!(r.teams[7].cup_points, 50); // 8th place
        assert_eq!(r.teams[6].cup_points, 0); // 9th place (outside top 8)
        assert_eq!(r.teams[0].cup_points, 0); // 15th place
    }

    #[test]
    fn overall_standings_ordered_by_cup_points() {
        let mut r = make_test_runtime();
        for _leg in 0..3 {
            for _ in 0..120 {
                match r.decide_next() {
                    CompetitionDecision::Jump { context, .. } => {
                        // Team 0 always scores highest
                        r.record_jump(100, 1000 - context.team_idx as i32 * 50, 15);
                    }
                    other => panic!("expected Jump, got {other:?}"),
                }
            }
            match r.decide_next() {
                CompetitionDecision::ShowResults(TeamCupResultsKind::LegResults) => {}
                other => panic!("expected ShowLegResults, got {other:?}"),
            }
            r.advance_after_results();
            match r.decide_next() {
                CompetitionDecision::ShowResults(TeamCupResultsKind::Standings) => {}
                other => panic!("expected ShowTeamCupStandings, got {other:?}"),
            }
            r.advance_after_results();
        }
        let overall = r.overall_standings();
        assert_eq!(overall[0].name, "Team0");
        assert!(overall[0].primary_score > overall[1].primary_score);
    }

    #[test]
    fn leg_standings_ordered_by_leg_score() {
        let mut r = make_test_runtime();
        for _ in 0..120 {
            match r.decide_next() {
                CompetitionDecision::Jump { context, .. } => {
                    // Team N scores higher per leg: team 0 = worst, team 14 = best
                    let score = context.team_idx as i32 * 10 + 100;
                    r.record_jump(100, score, 15);
                }
                other => panic!("expected Jump, got {other:?}"),
            }
        }
        let standings = r.current_leg_standings();
        assert_eq!(standings.len(), 15);
        assert_eq!(standings[0].name, "Team14"); // highest scoring team
        assert_eq!(
            standings[0].primary_score,
            (14 * 10 + 100) * 8 // 8 jumps per team
        );
        assert_eq!(standings[14].name, "Team0"); // lowest scoring team
    }

    #[test]
    fn human_team_jumps_are_flagged() {
        let mut r = make_test_runtime();
        // Team 0 and team 1 are human teams (set up in make_test_runtime)
        for _idx in 0..15 {
            let d = r.decide_next();
            match d {
                CompetitionDecision::Jump {
                    is_human, context, ..
                } => {
                    assert_eq!(is_human, context.team_idx < 2);
                    r.record_jump(100, 200, 15);
                }
                _ => panic!("expected Jump, got Jump?"),
            }
        }
    }

    #[test]
    fn each_jump_stores_correct_result() {
        let mut r = make_test_runtime();
        match r.decide_next() {
            CompetitionDecision::Jump { .. } => r.record_jump(120, 250, 14),
            other => panic!("expected Jump, got {other:?}"),
        }
        let team = &r.teams[0];
        let member = &team.members[0];
        assert_eq!(member.jumps.len(), 1);
        assert_eq!(member.jumps[0].distance, 120);
        assert_eq!(member.jumps[0].score, 250);
        assert_eq!(member.jumps[0].gate, 14);
        assert_eq!(member.jumps[0].leg, 0);
        assert_eq!(member.jumps[0].round, 0);
    }

    #[test]
    fn total_jumps_across_all_legs_is_720() {
        let mut r = make_test_runtime();
        simulate_all_jumps(&mut r);
        let total: usize = r
            .teams
            .iter()
            .flat_map(|t| t.members.iter())
            .map(|m| m.jumps.len())
            .sum();
        assert_eq!(total, 720); // 15 teams × 4 members × 2 rounds × 6 legs
    }
}

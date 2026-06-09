use crate::competition::core::competitor::{active_profiles, Competitor};
use crate::competition::core::schedule::random_unique_schedule;
use crate::competition::team_cup::types::{
    TeamCupMember, TeamCupRuntime, TeamCupTeam, MEMBERS_PER_TEAM, NUM_LEGS, NUM_TEAMS,
};
use crate::content::names::TeamDef;
use crate::data::profile::ProfileStore;
use crate::rng::Random;

#[must_use]
pub fn build_team_cup(
    names: &[String],
    teams_def: &[TeamDef],
    profiles: &ProfileStore,
    human_team_count: usize,
    hill_count: usize,
    rng: &mut Random,
) -> TeamCupRuntime {
    let teams = build_teams(names, teams_def, profiles, human_team_count);
    let schedule = build_schedule(hill_count, rng);

    TeamCupRuntime::new(teams, schedule, human_team_count)
}

fn build_teams(
    names: &[String],
    teams_def: &[TeamDef],
    profiles: &ProfileStore,
    human_team_count: usize,
) -> Vec<TeamCupTeam> {
    let max_teams = teams_def.len().min(NUM_TEAMS);
    let active_profiles = active_profiles(profiles);
    let mut profile_ptr = 0;
    let mut teams = Vec::with_capacity(max_teams);

    for ti in 0..max_teams {
        let td = &teams_def[ti];
        let is_human = ti < human_team_count;
        let mut members = Vec::with_capacity(MEMBERS_PER_TEAM);

        for mi in 0..MEMBERS_PER_TEAM {
            let name_idx = if mi < td.members.len() {
                td.members[mi]
            } else {
                1
            };
            let jumper_name = if name_idx > 0 && name_idx <= names.len() {
                names[name_idx - 1].clone()
            } else {
                format!("Jumper {}", ti * MEMBERS_PER_TEAM + mi + 1)
            };

            let id = ti * MEMBERS_PER_TEAM + mi;
            let member = if is_human && profile_ptr < active_profiles.len() {
                let (profile_idx, p) = active_profiles[profile_ptr];
                profile_ptr += 1;
                TeamCupMember {
                    competitor: Competitor::from_profile(id, profile_idx, p, Some(ti)),
                    jumps: Vec::new(),
                }
            } else {
                TeamCupMember {
                    competitor: Competitor::computer(id, id, jumper_name, Some(ti)),
                    jumps: Vec::new(),
                }
            };
            members.push(member);
        }

        teams.push(TeamCupTeam {
            id: ti,
            name: td.name.clone(),
            members,
            leg_score: 0,
            cup_points: 0,
            is_human_team: is_human,
        });
    }

    teams
}

fn build_schedule(hill_count: usize, rng: &mut Random) -> Vec<usize> {
    random_unique_schedule(hill_count, NUM_LEGS, rng)
}

use crate::competition::core::competitor::{active_profiles, Competitor};
use crate::competition::core::schedule::{random_unique_schedule, sequential_schedule};
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
    let team_order = shuffle_team_order(teams.len(), rng);

    TeamCupRuntime::new(teams, schedule, team_order)
}

fn shuffle_team_order(num_teams: usize, rng: &mut Random) -> Vec<usize> {
    let mut order: Vec<usize> = (0..num_teams).rev().collect();
    for _ in 0..3 {
        for i in 0..num_teams.saturating_sub(1) {
            if rng.random_i32(2) == 0 {
                order.swap(i, i + 1);
            }
        }
    }
    order
}

fn build_teams(
    names: &[String],
    teams_def: &[TeamDef],
    profiles: &ProfileStore,
    human_team_count: usize,
) -> Vec<TeamCupTeam> {
    let active_profiles = active_profiles(profiles);
    let mut profile_ptr = 0;
    let mut teams = Vec::with_capacity(NUM_TEAMS);

    // AI teams occupy the first slots; human teams are at the end (Pascal: jnimet[15], jnimet[14])
    let ai_count = NUM_TEAMS.saturating_sub(human_team_count);

    for ti in 0..NUM_TEAMS {
        let is_human = ti >= ai_count;
        let td = teams_def.get(ti).or_else(|| teams_def.last());

        let name = if is_human {
            // Will be overwritten by GetTeam; keep a placeholder
            format!("Team {}", ti + 1)
        } else if let Some(t) = td {
            t.name.clone()
        } else {
            format!("AI Team {}", ti + 1)
        };

        let members: Vec<TeamCupMember> = (0..MEMBERS_PER_TEAM)
            .map(|mi| {
                let id = ti * MEMBERS_PER_TEAM + mi;

                let name_idx = td.and_then(|t| t.members.get(mi).copied()).unwrap_or(1);
                let jumper_name = if name_idx > 0 && name_idx <= names.len() {
                    names[name_idx - 1].clone()
                } else {
                    format!("Jumper {}", id + 1)
                };

                if is_human && profile_ptr < active_profiles.len() {
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
                }
            })
            .collect();

        teams.push(TeamCupTeam {
            name,
            members,
            leg_score: 0.0,
            cup_points: 0,
            is_human_team: is_human,
        });
    }

    teams
}

fn build_schedule(hill_count: usize, rng: &mut Random) -> Vec<usize> {
    if hill_count > 0 {
        random_unique_schedule(hill_count, NUM_LEGS, rng)
    } else {
        sequential_schedule(hill_count, NUM_LEGS)
    }
}

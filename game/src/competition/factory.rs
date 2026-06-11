use crate::competition::builder::{build_competition, build_custom_competition};
use crate::competition::team_cup::builder::build_team_cup;
use crate::competition::types::CupStyle;
use crate::competition::ActiveCompetition;
use crate::content::names::TeamDef;
use crate::data::profile::ProfileStore;
use crate::rng::Random;

pub fn world_cup(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    trainrounds: usize,
) -> ActiveCompetition {
    ActiveCompetition::individual(build_competition(
        CupStyle::WorldCup,
        profiles,
        computer_names,
        hill_count,
        trainrounds,
    ))
}

pub fn four_hills(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    trainrounds: usize,
) -> ActiveCompetition {
    ActiveCompetition::individual(build_competition(
        CupStyle::FourHills,
        profiles,
        computer_names,
        hill_count,
        trainrounds,
    ))
}

pub fn custom_cup(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_order: Vec<usize>,
    trainrounds: usize,
) -> ActiveCompetition {
    ActiveCompetition::individual(build_custom_competition(
        profiles,
        computer_names,
        hill_order,
        trainrounds,
    ))
}

pub fn team_cup(
    names: &[String],
    teams_def: &[TeamDef],
    profiles: &ProfileStore,
    human_teams: usize,
    hill_count: usize,
    rng: &mut Random,
) -> ActiveCompetition {
    ActiveCompetition::team_cup(build_team_cup(
        names,
        teams_def,
        profiles,
        human_teams,
        hill_count,
        rng,
    ))
}

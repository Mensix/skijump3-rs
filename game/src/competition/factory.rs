use crate::competition::builder::{build_competition, build_custom_competition};
use crate::competition::koth::builder::build_koth;
use crate::competition::team_cup::builder::build_team_cup;
use crate::competition::types::CupStyle;
use crate::competition::ActiveCompetition;
use crate::content::names::TeamDef;
use crate::data::profile::ProfileStore;
use crate::rng::Random;
use crate::save::config::Config;

pub fn world_cup(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    training_rounds: usize,
    no_same_name: bool,
    ko_system: bool,
) -> ActiveCompetition {
    ActiveCompetition::from_individual(build_competition(
        CupStyle::WorldCup,
        profiles,
        computer_names,
        hill_count,
        training_rounds,
        no_same_name,
        ko_system,
    ))
}

pub fn four_hills(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    training_rounds: usize,
    no_same_name: bool,
    ko_system: bool,
) -> ActiveCompetition {
    ActiveCompetition::from_individual(build_competition(
        CupStyle::FourHills,
        profiles,
        computer_names,
        hill_count,
        training_rounds,
        no_same_name,
        ko_system,
    ))
}

pub fn custom_cup(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_order: Vec<usize>,
    training_rounds: usize,
    no_same_name: bool,
    ko_system: bool,
) -> ActiveCompetition {
    ActiveCompetition::from_individual(build_custom_competition(
        profiles,
        computer_names,
        hill_order,
        training_rounds,
        no_same_name,
        ko_system,
    ))
}

pub const fn training() -> ActiveCompetition {
    ActiveCompetition::training()
}

pub fn koth(
    config: &Config,
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    no_same_name: bool,
    rng: Random,
) -> ActiveCompetition {
    ActiveCompetition::from_koth(build_koth(
        config,
        profiles,
        computer_names,
        hill_count,
        no_same_name,
        rng,
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
    ActiveCompetition::from_team_cup(build_team_cup(
        names,
        teams_def,
        profiles,
        human_teams,
        hill_count,
        rng,
    ))
}

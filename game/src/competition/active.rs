use crate::competition::machine::Competition;
use crate::competition::team_cup::types::TeamCupRuntime;

#[derive(Debug, Clone)]
pub enum ActiveCompetition {
    Training,
    Individual(Competition),
    TeamCup(TeamCupRuntime),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveCompetitionKind {
    Training,
    Individual,
    TeamCup,
}

impl ActiveCompetition {
    pub fn from_individual(comp: Competition) -> Self {
        Self::Individual(comp)
    }

    pub const fn training() -> Self {
        Self::Training
    }

    pub fn from_team_cup(comp: TeamCupRuntime) -> Self {
        Self::TeamCup(comp)
    }

    pub const fn kind(&self) -> ActiveCompetitionKind {
        match self {
            Self::Training => ActiveCompetitionKind::Training,
            Self::Individual(_) => ActiveCompetitionKind::Individual,
            Self::TeamCup(_) => ActiveCompetitionKind::TeamCup,
        }
    }

    pub const fn individual(&self) -> Option<&Competition> {
        match self {
            Self::Training => None,
            Self::Individual(comp) => Some(comp),
            Self::TeamCup(_) => None,
        }
    }

    pub const fn individual_mut(&mut self) -> Option<&mut Competition> {
        match self {
            Self::Training => None,
            Self::Individual(comp) => Some(comp),
            Self::TeamCup(_) => None,
        }
    }

    pub const fn team_cup_runtime(&self) -> Option<&TeamCupRuntime> {
        match self {
            Self::Training => None,
            Self::Individual(_) => None,
            Self::TeamCup(comp) => Some(comp),
        }
    }

    pub const fn team_cup_runtime_mut(&mut self) -> Option<&mut TeamCupRuntime> {
        match self {
            Self::Training => None,
            Self::Individual(_) => None,
            Self::TeamCup(comp) => Some(comp),
        }
    }
}

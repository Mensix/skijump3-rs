use crate::competition::machine::Competition;
use crate::competition::team_cup::types::TeamCupRuntime;

#[derive(Debug, Clone)]
pub enum ActiveCompetition {
    Training,
    Individual(Competition),
    TeamCup(TeamCupRuntime),
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

    pub const fn is_team_cup(&self) -> bool {
        matches!(self, Self::TeamCup(_))
    }

    pub const fn is_training(&self) -> bool {
        matches!(self, Self::Training)
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

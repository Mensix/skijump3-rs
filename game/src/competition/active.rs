use crate::competition::koth::types::KothRuntime;
use crate::competition::machine::Competition;
use crate::competition::team_cup::types::TeamCupRuntime;
use serde::{Deserialize, Serialize};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActiveCompetition {
    Training,
    Individual(Competition),
    TeamCup(TeamCupRuntime),
    Koth(KothRuntime),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveCompetitionKind {
    Training,
    Individual,
    TeamCup,
    Koth,
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

    pub fn from_koth(comp: KothRuntime) -> Self {
        Self::Koth(comp)
    }

    pub const fn kind(&self) -> ActiveCompetitionKind {
        match self {
            Self::Training => ActiveCompetitionKind::Training,
            Self::Individual(_) => ActiveCompetitionKind::Individual,
            Self::TeamCup(_) => ActiveCompetitionKind::TeamCup,
            Self::Koth(_) => ActiveCompetitionKind::Koth,
        }
    }

    pub const fn individual(&self) -> Option<&Competition> {
        match self {
            Self::Training => None,
            Self::Individual(comp) => Some(comp),
            Self::TeamCup(_) => None,
            Self::Koth(_) => None,
        }
    }

    pub const fn individual_mut(&mut self) -> Option<&mut Competition> {
        match self {
            Self::Training => None,
            Self::Individual(comp) => Some(comp),
            Self::TeamCup(_) => None,
            Self::Koth(_) => None,
        }
    }

    pub const fn team_cup_runtime(&self) -> Option<&TeamCupRuntime> {
        match self {
            Self::Training => None,
            Self::Individual(_) => None,
            Self::TeamCup(comp) => Some(comp),
            Self::Koth(_) => None,
        }
    }

    pub const fn team_cup_runtime_mut(&mut self) -> Option<&mut TeamCupRuntime> {
        match self {
            Self::Training => None,
            Self::Individual(_) => None,
            Self::TeamCup(comp) => Some(comp),
            Self::Koth(_) => None,
        }
    }

    pub const fn koth_runtime(&self) -> Option<&KothRuntime> {
        match self {
            Self::Koth(comp) => Some(comp),
            _ => None,
        }
    }

    pub const fn koth_runtime_mut(&mut self) -> Option<&mut KothRuntime> {
        match self {
            Self::Koth(comp) => Some(comp),
            _ => None,
        }
    }
}

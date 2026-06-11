use crate::competition::machine::Competition;
use crate::competition::team_cup::types::TeamCupRuntime;

#[derive(Debug, Clone)]
pub enum ActiveCompetition {
    Individual(Competition),
    TeamCup(TeamCupRuntime),
}

impl ActiveCompetition {
    pub fn individual(comp: Competition) -> Self {
        Self::Individual(comp)
    }

    pub fn team_cup(comp: TeamCupRuntime) -> Self {
        Self::TeamCup(comp)
    }
}

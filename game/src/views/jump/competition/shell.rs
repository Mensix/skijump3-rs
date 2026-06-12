use crate::competition::active::ActiveCompetitionKind;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::koth::KothJumpView;
use crate::views::jump::team_cup::TeamCupJumpView;
use crate::views::jump::training_jump::TrainingJumpView;
use crate::views::jump::world_cup::WorldCupJumpView;
use engine::ui::{BackgroundMode, Element, Event, View};

pub(crate) enum CompetitionJumpView {
    Training(TrainingJumpView),
    Individual(WorldCupJumpView),
    TeamCup(TeamCupJumpView),
    Koth(KothJumpView),
}

impl CompetitionJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        match store
            .with_active(|active| active.kind())
            .unwrap_or(ActiveCompetitionKind::Individual)
        {
            ActiveCompetitionKind::Training => {
                Self::Training(TrainingJumpView::new(resources, store))
            }
            ActiveCompetitionKind::Individual => {
                Self::Individual(WorldCupJumpView::new(resources, store))
            }
            ActiveCompetitionKind::TeamCup => Self::TeamCup(TeamCupJumpView::new(resources, store)),
            ActiveCompetitionKind::Koth => Self::Koth(KothJumpView::new(resources, store)),
        }
    }
}

impl View<RouteTarget> for CompetitionJumpView {
    fn update(&mut self) {
        match self {
            Self::Training(view) => view.update(),
            Self::Individual(view) => view.update(),
            Self::TeamCup(view) => view.update(),
            Self::Koth(view) => view.update(),
        }
    }

    fn elements(&self) -> Vec<Element> {
        match self {
            Self::Training(view) => view.elements(),
            Self::Individual(view) => view.elements(),
            Self::TeamCup(view) => view.elements(),
            Self::Koth(view) => view.elements(),
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self {
            Self::Training(view) => view.handle_event(event),
            Self::Individual(view) => view.handle_event(event),
            Self::TeamCup(view) => view.handle_event(event),
            Self::Koth(view) => view.handle_event(event),
        }
    }

    fn gpu_background(&self) -> BackgroundMode {
        match self {
            Self::Training(view) => view.gpu_background(),
            Self::Individual(view) => view.gpu_background(),
            Self::TeamCup(view) => view.gpu_background(),
            Self::Koth(view) => view.gpu_background(),
        }
    }
}

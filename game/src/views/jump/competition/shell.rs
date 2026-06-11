use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::team_cup::TeamCupJumpView;
use crate::views::jump::training_jump::TrainingJumpView;
use crate::views::jump::world_cup::WorldCupJumpView;
use engine::ui::{BackgroundMode, Element, Event, View};

pub(crate) enum CompetitionJumpView {
    Training(TrainingJumpView),
    Individual(WorldCupJumpView),
    TeamCup(TeamCupJumpView),
}

impl CompetitionJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        if store
            .with_active(|active| active.is_training())
            .unwrap_or(false)
        {
            Self::Training(TrainingJumpView::new(resources, store))
        } else if store
            .with_active(|active| active.is_team_cup())
            .unwrap_or(false)
        {
            Self::TeamCup(TeamCupJumpView::new(resources, store))
        } else {
            Self::Individual(WorldCupJumpView::new(resources, store))
        }
    }
}

impl View<RouteTarget> for CompetitionJumpView {
    fn update(&mut self) {
        match self {
            Self::Training(view) => view.update(),
            Self::Individual(view) => view.update(),
            Self::TeamCup(view) => view.update(),
        }
    }

    fn elements(&self) -> Vec<Element> {
        match self {
            Self::Training(view) => view.elements(),
            Self::Individual(view) => view.elements(),
            Self::TeamCup(view) => view.elements(),
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match self {
            Self::Training(view) => view.handle_event(event),
            Self::Individual(view) => view.handle_event(event),
            Self::TeamCup(view) => view.handle_event(event),
        }
    }

    fn gpu_background(&self) -> BackgroundMode {
        match self {
            Self::Training(view) => view.gpu_background(),
            Self::Individual(view) => view.gpu_background(),
            Self::TeamCup(view) => view.gpu_background(),
        }
    }
}

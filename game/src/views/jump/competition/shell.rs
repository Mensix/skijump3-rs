use crate::competition::active::ActiveCompetitionKind;
use crate::route::RouteTarget;
use crate::store::{GameStateRef, ResourcesRef};
use crate::views::jump::koth::KothJumpView;
use crate::views::jump::team_cup::TeamCupJumpView;
use crate::views::jump::training_jump::TrainingJumpView;
use crate::views::jump::world_cup::WorldCupJumpView;
use engine::oxide::{PaintCx, Screen, ScreenBackground, ScreenEventCx, UiEvent};

pub(crate) enum CompetitionJumpView {
    Training(TrainingJumpView),
    Individual(WorldCupJumpView),
    TeamCup(TeamCupJumpView),
    Koth(KothJumpView),
}

impl CompetitionJumpView {
    pub(crate) fn new(resources: ResourcesRef, store: GameStateRef) -> Self {
        let kind = store
            .borrow()
            .active_competition
            .as_ref()
            .map(|a| a.kind())
            .unwrap_or(ActiveCompetitionKind::Individual);
        match kind {
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

impl Screen<RouteTarget> for CompetitionJumpView {
    fn update(&mut self) {
        match self {
            Self::Training(view) => Screen::update(view),
            Self::Individual(view) => Screen::update(view),
            Self::TeamCup(view) => Screen::update(view),
            Self::Koth(view) => Screen::update(view),
        }
    }

    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match self {
            Self::Training(view) => Screen::event(view, cx, event),
            Self::Individual(view) => Screen::event(view, cx, event),
            Self::TeamCup(view) => Screen::event(view, cx, event),
            Self::Koth(view) => Screen::event(view, cx, event),
        }
    }

    fn paint(&self, cx: &mut PaintCx<'_>) {
        match self {
            Self::Training(view) => Screen::paint(view, cx),
            Self::Individual(view) => Screen::paint(view, cx),
            Self::TeamCup(view) => Screen::paint(view, cx),
            Self::Koth(view) => Screen::paint(view, cx),
        }
    }

    fn background(&self) -> ScreenBackground {
        match self {
            Self::Training(view) => Screen::background(view),
            Self::Individual(view) => Screen::background(view),
            Self::TeamCup(view) => Screen::background(view),
            Self::Koth(view) => Screen::background(view),
        }
    }
}

use crate::competition::active::ActiveCompetitionKind;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::views::jump::koth::KothJumpView;
use crate::views::jump::team_cup::TeamCupJumpView;
use crate::views::jump::training_jump::TrainingJumpView;
use crate::views::jump::world_cup::WorldCupJumpView;
use engine::oxide::{PaintCx, ScreenBackground, ScreenEventCx, UiEvent};

pub(crate) enum CompetitionJumpView {
    Training(TrainingJumpView),
    Individual(WorldCupJumpView),
    TeamCup(TeamCupJumpView),
    Koth(KothJumpView),
}

impl CompetitionJumpView {
    pub(crate) fn new(
        resources: ResourcesRef,
        save_manager: SaveRef,
        state: &mut GameState,
        kind: ActiveCompetitionKind,
    ) -> Self {
        match kind {
            ActiveCompetitionKind::Training => {
                Self::Training(TrainingJumpView::new(resources, state, save_manager))
            }
            ActiveCompetitionKind::Individual => {
                Self::Individual(WorldCupJumpView::new(resources, save_manager))
            }
            ActiveCompetitionKind::TeamCup => {
                Self::TeamCup(TeamCupJumpView::new(resources, save_manager))
            }
            ActiveCompetitionKind::Koth => Self::Koth(KothJumpView::new(resources, save_manager)),
        }
    }
}

impl GameScreen for CompetitionJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        // On first update, check if we need to create the correct view variant
        // based on the actual active competition kind in state
        match self {
            Self::Training(view) => view.update(cx),
            Self::Individual(view) => view.update(cx),
            Self::TeamCup(view) => view.update(cx),
            Self::Koth(view) => view.update(cx),
        }
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match self {
            Self::Training(view) => view.event(cx, nav, event),
            Self::Individual(view) => view.event(cx, nav, event),
            Self::TeamCup(view) => view.event(cx, nav, event),
            Self::Koth(view) => view.event(cx, nav, event),
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        match self {
            Self::Training(view) => view.paint(cx, paint),
            Self::Individual(view) => view.paint(cx, paint),
            Self::TeamCup(view) => view.paint(cx, paint),
            Self::Koth(view) => view.paint(cx, paint),
        }
    }

    fn background(&self) -> ScreenBackground {
        match self {
            Self::Training(view) => view.background(),
            Self::Individual(view) => view.background(),
            Self::TeamCup(view) => view.background(),
            Self::Koth(view) => view.background(),
        }
    }
}

use crate::competition::active::ActiveCompetitionKind;
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::{ScreenBackground, ScreenEventCx, UiEvent};
use crate::views::jump::koth::KothJumpView;
use crate::views::jump::team_cup::TeamCupJumpView;
use crate::views::jump::training_jump::TrainingJumpView;
use crate::views::jump::world_cup::WorldCupJumpView;

pub(crate) enum CompetitionJumpView {
    Training(CompetitionShell<TrainingJumpView>),
    Individual(CompetitionShell<WorldCupJumpView>),
    TeamCup(CompetitionShell<TeamCupJumpView>),
    Koth(CompetitionShell<KothJumpView>),
}

pub(crate) struct CompetitionShell<T> {
    inner: T,
}

impl<T> CompetitionShell<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl CompetitionJumpView {
    pub(crate) fn new(
        resources: ResourcesRef,
        state: &mut GameState,
        kind: ActiveCompetitionKind,
    ) -> Self {
        match kind {
            ActiveCompetitionKind::Training => Self::Training(CompetitionShell::new(
                TrainingJumpView::new(resources, state),
            )),
            ActiveCompetitionKind::Individual => Self::Individual(CompetitionShell::new(
                WorldCupJumpView::new(resources, state),
            )),
            ActiveCompetitionKind::TeamCup => Self::TeamCup(CompetitionShell::new(
                TeamCupJumpView::new(resources, state),
            )),
            ActiveCompetitionKind::Koth => {
                Self::Koth(CompetitionShell::new(KothJumpView::new(resources)))
            }
        }
    }
}

impl<T: GameScreen> CompetitionShell<T> {
    fn paused_update(&mut self, cx: &mut GameCx<'_>) {
        self.inner.update(cx);
    }
}

impl GameScreen for CompetitionJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        match self {
            Self::Training(view) => view.paused_update(cx),
            Self::Individual(view) => view.paused_update(cx),
            Self::TeamCup(view) => view.paused_update(cx),
            Self::Koth(view) => view.paused_update(cx),
        }
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match self {
            Self::Training(view) => view.inner.event(cx, nav, event),
            Self::Individual(view) => view.inner.event(cx, nav, event),
            Self::TeamCup(view) => view.inner.event(cx, nav, event),
            Self::Koth(view) => view.inner.event(cx, nav, event),
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        match self {
            Self::Training(view) => shell_paint(view, cx, paint),
            Self::Individual(view) => shell_paint(view, cx, paint),
            Self::TeamCup(view) => shell_paint(view, cx, paint),
            Self::Koth(view) => shell_paint(view, cx, paint),
        }
    }

    fn background(&self) -> ScreenBackground {
        match self {
            Self::Training(view) => view.inner.background(),
            Self::Individual(view) => view.inner.background(),
            Self::TeamCup(view) => view.inner.background(),
            Self::Koth(view) => view.inner.background(),
        }
    }

    fn has_modal(&self) -> bool {
        match self {
            Self::Training(view) => view.inner.has_modal(),
            Self::Individual(view) => view.inner.has_modal(),
            Self::TeamCup(view) => view.inner.has_modal(),
            Self::Koth(view) => view.inner.has_modal(),
        }
    }
}

fn shell_paint<T: GameScreen>(
    shell: &mut CompetitionShell<T>,
    cx: &mut GameCx<'_>,
    paint: &mut dyn UiCanvas,
) {
    shell.inner.paint(cx, paint);
}

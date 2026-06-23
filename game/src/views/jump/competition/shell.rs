use crate::competition::active::ActiveCompetitionKind;
use crate::components::modal::alert_prompt;
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::views::jump::koth::KothJumpView;
use crate::views::jump::team_cup::TeamCupJumpView;
use crate::views::jump::training_jump::TrainingJumpView;
use crate::views::jump::world_cup::WorldCupJumpView;
use engine::oxide::{Key, PaintCx, ScreenBackground, ScreenEventCx, UiEvent};

pub(crate) enum CompetitionJumpView {
    Training(CompetitionShell<TrainingJumpView>),
    Individual(CompetitionShell<WorldCupJumpView>),
    TeamCup(CompetitionShell<TeamCupJumpView>),
    Koth(CompetitionShell<KothJumpView>),
}

pub(crate) struct CompetitionShell<T> {
    inner: T,
    save_prompt: bool,
}

impl<T> CompetitionShell<T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
            save_prompt: false,
        }
    }
}

impl CompetitionJumpView {
    pub(crate) fn new(
        resources: ResourcesRef,
        save_manager: SaveRef,
        state: &mut GameState,
        kind: ActiveCompetitionKind,
    ) -> Self {
        match kind {
            ActiveCompetitionKind::Training => Self::Training(CompetitionShell::new(
                TrainingJumpView::new(resources, state, save_manager),
            )),
            ActiveCompetitionKind::Individual => Self::Individual(CompetitionShell::new(
                WorldCupJumpView::new(resources, save_manager, state),
            )),
            ActiveCompetitionKind::TeamCup => Self::TeamCup(CompetitionShell::new(
                TeamCupJumpView::new(resources, save_manager),
            )),
            ActiveCompetitionKind::Koth => Self::Koth(CompetitionShell::new(KothJumpView::new(
                resources,
                save_manager,
            ))),
        }
    }
}

impl<T: GameScreen> CompetitionShell<T> {
    fn paused_update(&mut self, cx: &mut GameCx<'_>) {
        if !self.save_prompt {
            self.inner.update(cx);
        }
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
            Self::Training(view) => shell_event(view, false, cx, nav, event),
            Self::Individual(view) => shell_event(view, true, cx, nav, event),
            Self::TeamCup(view) => shell_event(view, true, cx, nav, event),
            Self::Koth(view) => shell_event(view, true, cx, nav, event),
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
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
}

fn shell_event<T: GameScreen>(
    shell: &mut CompetitionShell<T>,
    has_cup: bool,
    cx: &mut GameCx<'_>,
    nav: &mut ScreenEventCx<RouteTarget>,
    event: UiEvent,
) {
    if shell.save_prompt {
        match event {
            UiEvent::Text(c) if is_yes(c, cx) => {
                if let Some(active) = cx.state.active_competition.as_ref() {
                    let filename = cx.state.active_cup_filename.as_deref();
                    let new_name = cx.save_manager.save_active_cup(active, filename);
                    cx.state.active_cup_filename = Some(new_name);
                }
                nav.back();
            }
            UiEvent::Text(c) if is_no(c, cx) => nav.back(),
            UiEvent::KeyDown(Key::Escape) => {
                shell.save_prompt = false;
                nav.consume();
            }
            UiEvent::KeyDown(_) | UiEvent::Text(_) => nav.consume(),
            _ => {}
        }
        return;
    }

    if has_cup && matches!(event, UiEvent::KeyDown(Key::Escape)) {
        shell.save_prompt = true;
        nav.consume();
        return;
    }

    shell.inner.event(cx, nav, event);
}

fn is_yes(c: char, cx: &GameCx<'_>) -> bool {
    let lang = &cx.layout.langbase;
    let localized = lang
        .tr(6)
        .chars()
        .next()
        .unwrap_or('Y')
        .to_ascii_uppercase();
    c.to_ascii_uppercase() == localized || c.eq_ignore_ascii_case(&'Y')
}

fn is_no(c: char, cx: &GameCx<'_>) -> bool {
    let lang = &cx.layout.langbase;
    let localized = lang
        .tr(7)
        .chars()
        .next()
        .unwrap_or('N')
        .to_ascii_uppercase();
    c.to_ascii_uppercase() == localized || c.eq_ignore_ascii_case(&'N')
}

fn shell_paint<T: GameScreen>(
    shell: &mut CompetitionShell<T>,
    cx: &mut GameCx<'_>,
    paint: &mut PaintCx<'_>,
) {
    let lang = &cx.layout.langbase;
    shell.inner.paint(cx, paint);
    if shell.save_prompt {
        alert_prompt(paint, lang.tr(521), lang.tr(522), true);
    }
}

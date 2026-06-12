use crate::competition::active::ActiveCompetitionKind;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::koth::KothJumpView;
use crate::views::jump::team_cup::TeamCupJumpView;
use crate::views::jump::training_jump::TrainingJumpView;
use crate::views::jump::world_cup::WorldCupJumpView;
use engine::oxide::legacy::commands_to_elements;
use engine::oxide::{CommandBuffer, NavAction, PaintCx, Screen, ScreenEventCx, UiEvent};
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
            Self::Training(view) => View::update(view),
            Self::Individual(view) => View::update(view),
            Self::TeamCup(view) => View::update(view),
            Self::Koth(view) => View::update(view),
        }
    }

    fn elements(&self) -> Vec<Element> {
        let mut commands = CommandBuffer::new();
        let mut cx = PaintCx::new(&mut commands);
        Screen::paint(self, &mut cx);
        commands_to_elements(&commands)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        let mut cx = ScreenEventCx::default();
        Screen::event(self, &mut cx, event.into());
        match cx.take_action() {
            NavAction::Navigate(route) => Some(route),
            NavAction::Back => Some(RouteTarget::Back),
            NavAction::Quit => Some(RouteTarget::Quit),
            NavAction::None => None,
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

impl Screen<RouteTarget> for CompetitionJumpView {
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
}

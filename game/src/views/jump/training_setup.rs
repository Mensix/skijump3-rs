use crate::competition::factory;
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::{EventCx, Key, ScreenEventCx, UiEvent};
use crate::views::jump::hill_list::{HillListPicker, HillPick};

pub struct TrainingSetupView {
    picker: HillListPicker,
}

impl TrainingSetupView {
    pub fn new(resources: ResourcesRef, state: &GameState) -> Self {
        Self {
            picker: HillListPicker::new(resources, state.practice_hill),
        }
    }

    fn confirm(&mut self, state: &mut GameState, pick: HillPick) -> Option<RouteTarget> {
        match pick {
            HillPick::Exit => Some(RouteTarget::MainMenu),
            HillPick::More => None,
            HillPick::Hill(hill_idx) => {
                state.practice_hill = hill_idx;
                state.start_active(factory::training());
                Some(RouteTarget::CompetitionJump)
            }
        }
    }

    fn paint_content(&self, cx: &mut dyn UiCanvas) {
        self.picker.paint(cx, 154);
    }

    fn handle_input(
        &mut self,
        ecx: &mut EventCx,
        state: &mut GameState,
        event: UiEvent,
    ) -> Option<RouteTarget> {
        self.picker
            .confirm(ecx, event)
            .and_then(|pick| self.confirm(state, pick))
    }
}

impl GameScreen for TrainingSetupView {
    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::Quit | UiEvent::Tick => return,
            UiEvent::KeyDown(Key::Escape | Key::F10) => {
                nav.back();
                return;
            }
            _ => {}
        }
        let mut ecx = EventCx::default();
        if let Some(route) = self.handle_input(&mut ecx, cx.state, event) {
            nav.navigate(route);
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.paint_content(paint);
    }
}

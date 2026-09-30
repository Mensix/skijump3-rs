use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen, Persistence};
use crate::store::{GameState, ResourcesRef};
use crate::ui::UiCanvas;
use crate::ui::{EventCx, Key, ScreenEventCx, UiEvent};
use crate::views::jump::hill_list::{HillListPicker, HillPick};

pub struct KothHillPickerView {
    picker: HillListPicker,
}

impl KothHillPickerView {
    pub fn new(resources: ResourcesRef) -> Self {
        Self {
            picker: HillListPicker::new(resources, 0),
        }
    }

    fn select_hill(&mut self, state: &mut GameState, cx: &Persistence, pick: HillPick) -> bool {
        match pick {
            HillPick::Exit => {
                state.config.koth_pack = 0;
                state.config.koth_hill = -1;
                let cfg = state.config.clone();
                cx.save_config(&cfg);
                true
            }
            HillPick::More => false,
            HillPick::Hill(hill_idx) => {
                state.config.koth_pack = 0;
                state.config.koth_hill = hill_idx as i32;
                let cfg = state.config.clone();
                cx.save_config(&cfg);
                true
            }
        }
    }

    fn confirm_event(
        &mut self,
        ecx: &mut EventCx,
        event: UiEvent,
        state: &mut GameState,
        cx: &Persistence,
    ) -> bool {
        if let Some(pick) = self.picker.confirm(ecx, event) {
            self.select_hill(state, cx, pick)
        } else {
            false
        }
    }
}

impl GameScreen for KothHillPickerView {
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
        let persistence = cx.persistence();
        if self.confirm_event(&mut ecx, event, cx.state, &persistence) {
            nav.back();
        }
        if ecx.is_consumed() {
            nav.consume();
        }
    }

    fn paint(&mut self, _: &mut GameCx<'_>, paint: &mut dyn UiCanvas) {
        self.picker.paint(paint, 155);
    }
}

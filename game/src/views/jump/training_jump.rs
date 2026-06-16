use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::store::{GameStateRef, ResourcesRef};
use crate::views::jump::input::JumpInputAction;
use crate::views::jump::scene::JumpScene;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
pub struct TrainingJumpView {
    store: GameStateRef,
    scene: JumpScene,
}

impl TrainingJumpView {
    pub fn new(resources: ResourcesRef, store: GameStateRef, _save_manager: SaveRef) -> Self {
        let hill_idx = store.borrow().practice_hill;
        let participant = JumpParticipant::trainee();
        let start_gate = store.borrow().practice_start_gate;
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            GameStateRef::clone(&store),
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::training(),
        );

        Self { store, scene }
    }

    fn handle_jump_event(&mut self, event: UiEvent) -> Option<RouteTarget> {
        let action = self.scene.handle_jump_input(event);
        match action {
            JumpInputAction::None => None,
            JumpInputAction::RouteBack => Some(RouteTarget::Back),
            JumpInputAction::SaveReplay => {
                self.scene.open_save_dialog();
                None
            }
            JumpInputAction::ResetWind => {
                self.store.borrow_mut().reset_practice_wind();
                None
            }
            JumpInputAction::ResetJump => {
                let _ = self.scene.outcome();
                let _ = self.scene.replay_trace();
                self.scene
                    .reset_state(self.store.borrow().practice_start_gate);
                None
            }
            JumpInputAction::PersistStartGate(start_gate) => {
                self.store.borrow_mut().practice_start_gate = start_gate;
                None
            }
        }
    }

    fn paint_content(&mut self, cx: &mut PaintCx<'_>) {
        self.scene.render(cx);
    }

    fn handle_input(&mut self, event: UiEvent) -> Option<RouteTarget> {
        if self.scene.is_save_dialog_active() {
            self.scene.handle_save_dialog_event(&event);
            None
        } else {
            self.handle_jump_event(event)
        }
    }
}

impl Screen<RouteTarget> for TrainingJumpView {
    fn update(&mut self) {
        self.scene.update();
    }

    fn event(&mut self, cx: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::Quit | UiEvent::Tick => return,
            _ => {}
        }
        if let Some(route) = self.handle_input(event) {
            if route == RouteTarget::Back {
                cx.back();
            } else {
                cx.navigate(route);
            }
        } else {
            cx.consume();
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

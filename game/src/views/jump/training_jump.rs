use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::save::SaveRef;
use crate::screen::{GameCx, GameScreen};
use crate::store::{GameState, ResourcesRef};
use crate::views::jump::input::JumpInputAction;
use crate::views::jump::scene::JumpScene;
use engine::oxide::{PaintCx, ScreenEventCx, UiEvent};
pub struct TrainingJumpView {
    scene: JumpScene,
}

impl TrainingJumpView {
    pub fn new(resources: ResourcesRef, state: &mut GameState, _save_manager: SaveRef) -> Self {
        let hill_idx = state.practice_hill;
        let participant = JumpParticipant::trainee();
        let start_gate = state.practice_start_gate;
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            state,
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::training(),
        );

        Self { scene }
    }

    fn handle_jump_event(&mut self, state: &mut GameState, event: UiEvent) -> Option<RouteTarget> {
        let action = self.scene.handle_jump_input(state, event);
        match action {
            JumpInputAction::None => None,
            JumpInputAction::SaveReplay => {
                self.scene.open_save_dialog(state);
                None
            }
            JumpInputAction::ResetWind => {
                state.reset_practice_wind();
                None
            }
            JumpInputAction::ResetJump => {
                let _ = self.scene.outcome();
                let _ = self.scene.replay_trace();
                self.scene.reset_state(state, state.practice_start_gate);
                None
            }
            JumpInputAction::PersistStartGate(start_gate) => {
                state.practice_start_gate = start_gate;
                None
            }
        }
    }

    fn paint_content(&mut self, cx: &mut PaintCx<'_>, state: &GameState) {
        self.scene.render(cx, state);
    }

    fn handle_input(&mut self, state: &mut GameState, event: UiEvent) -> Option<RouteTarget> {
        if self.scene.is_save_dialog_active() {
            self.scene.handle_save_dialog_event(&event);
            None
        } else {
            self.handle_jump_event(state, event)
        }
    }
}

impl GameScreen for TrainingJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        self.scene.update(cx.state);
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        match event {
            UiEvent::Quit | UiEvent::Tick => return,
            _ => {}
        }
        if let Some(route) = self.handle_input(cx.state, event) {
            if route == RouteTarget::Back {
                nav.back();
            } else {
                nav.navigate(route);
            }
        } else {
            nav.consume();
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        self.paint_content(paint, cx.state);
    }
}

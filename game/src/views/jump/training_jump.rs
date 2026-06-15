use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::jump::input::JumpInputAction;
use crate::views::jump::scene::JumpScene;
use engine::oxide::{PaintCx, Screen, ScreenEventCx, UiEvent};
use std::cell::RefCell;

pub struct TrainingJumpView {
    store: StoreRef,
    scene: RefCell<JumpScene>,
}

impl TrainingJumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = store.practice_hill();
        let participant = JumpParticipant::trainee();
        let start_gate = store.practice_start_gate();
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            StoreRef::clone(&store),
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::training(),
        );

        Self {
            store,
            scene: RefCell::new(scene),
        }
    }

    fn handle_jump_event(&self, event: UiEvent) -> Option<RouteTarget> {
        let action = self.scene.borrow().handle_jump_input(event);
        match action {
            JumpInputAction::None => None,
            JumpInputAction::RouteBack => Some(RouteTarget::Back),
            JumpInputAction::SaveReplay => {
                self.scene.borrow().open_save_dialog();
                None
            }
            JumpInputAction::ResetWind => {
                self.store.reset_practice_wind();
                None
            }
            JumpInputAction::ResetJump => {
                let _ = self.scene.borrow().outcome();
                let _ = self.scene.borrow().replay_trace();
                self.scene
                    .borrow()
                    .reset_state(self.store.practice_start_gate());
                None
            }
            JumpInputAction::PersistStartGate(start_gate) => {
                self.store.set_practice_start_gate(start_gate);
                self.store.set_start_gate(start_gate);
                None
            }
        }
    }

    fn paint_content(&self, cx: &mut PaintCx<'_>) {
        self.scene.borrow().render(cx);
    }

    fn handle_input(&self, event: UiEvent) -> Option<RouteTarget> {
        let scene = self.scene.borrow();
        if scene.is_save_dialog_active() {
            scene.handle_save_dialog_event(&event);
            None
        } else {
            drop(scene);
            self.handle_jump_event(event)
        }
    }
}

impl Screen<RouteTarget> for TrainingJumpView {
    fn update(&mut self) {
        self.scene.borrow_mut().update();
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

    fn paint(&self, cx: &mut PaintCx<'_>) {
        self.paint_content(cx);
    }
}

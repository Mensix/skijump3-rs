use crate::components::save_replay_dialog::{SaveAction, SaveReplayDialog};
use crate::controllers::jump_input::{JumpInputAction, JumpInputController};
use crate::controllers::jump_scene::JumpScene;
use crate::jump::{JumpParticipant, JumpPolicy};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::palette::Palette;
use engine::ui::{Component, Element, Event, View};
use std::cell::RefCell;

pub struct TrainingJumpView {
    resources: ResourcesRef,
    store: StoreRef,
    scene: RefCell<JumpScene>,
    save_dialog: SaveReplayDialog,
}

impl TrainingJumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = store.practice.hill.get();
        let participant = JumpParticipant::trainee();
        let start_gate = store.practice.start_gate.get();
        JumpScene::setup_event(&store);
        let scene = JumpScene::new(
            ResourcesRef::clone(&resources),
            StoreRef::clone(&store),
            hill_idx,
            start_gate,
            participant,
            JumpPolicy::training(),
        );
        let save_dialog = SaveReplayDialog::new(ResourcesRef::clone(&resources));

        Self {
            resources,
            store,
            scene: RefCell::new(scene),
            save_dialog,
        }
    }

    fn handle_jump_event(&mut self, event: Event) -> Option<RouteTarget> {
        let action = {
            let scene = self.scene.borrow_mut();
            let mut session = scene.session_mut();
            JumpInputController.handle_event(event, &mut session)
        };
        match action {
            JumpInputAction::None => None,
            JumpInputAction::RouteBack => Some(RouteTarget::Back),
            JumpInputAction::ResetWind => {
                self.store.jump_runtime.reset_practice_wind();
                None
            }
            JumpInputAction::ResetJump => {
                let _ = self.scene.borrow_mut().outcome();
                let _ = self.scene.borrow().replay_trace();
                self.scene
                    .borrow_mut()
                    .reset_state(self.store.practice.start_gate.get());
                None
            }
            JumpInputAction::PersistStartGate(start_gate) => {
                self.store.practice.start_gate.set(start_gate);
                self.store.start_gate.set(start_gate);
                None
            }
            JumpInputAction::SaveReplay => {
                self.enter_save_dialog();
                None
            }
        }
    }

    fn enter_save_dialog(&mut self) {
        let distance = self
            .scene
            .borrow()
            .outcome()
            .map(|o| format!("{:.1}", f64::from(o.distance) / 10.0))
            .unwrap_or_default();
        let hill_name = self
            .resources
            .hills
            .hill(self.scene.borrow().hill_idx())
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let pb = self.store.profiles.borrow();
        let author_name = pb
            .active_order
            .first()
            .and_then(|&idx| {
                let p = pb.profiles.get(idx)?;
                Some(if p.real_name.is_empty() {
                    p.name.clone()
                } else {
                    p.real_name.clone()
                })
            })
            .unwrap_or_default();
        self.save_dialog.open(
            author_name,
            format!("Huge Jump in {hill_name}"),
            distance,
            hill_name,
        );
    }

    fn do_save_replay(&mut self) {
        if let Some(trace) = self.scene.borrow().replay_trace() {
            self.save_dialog.write_replay(&trace);
        }
    }
}

impl View<RouteTarget> for TrainingJumpView {
    fn update(&mut self) {
        if !self.save_dialog.is_active() {
            self.scene.borrow_mut().update();
        }
    }

    fn elements(&self) -> Vec<Element> {
        if self.save_dialog.is_active() {
            return self.save_dialog.elements();
        }
        self.scene.borrow().elements()
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.save_dialog.is_active() {
            match self.save_dialog.handle_event(&event) {
                Some(SaveAction::SaveReplay) => self.do_save_replay(),
                Some(SaveAction::Consumed) | None => {}
            }
            None
        } else {
            self.handle_jump_event(event)
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if !self.save_dialog.is_active() {
            self.scene.borrow_mut().render_snow(framebuffer);
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if !self.save_dialog.is_active() {
            self.scene.borrow().apply_palette(palette);
        }
    }

    fn requires_legacy_framebuffer(&self) -> bool {
        true
    }
}

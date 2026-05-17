use crate::components::save_replay_dialog::{SaveAction, SaveReplayDialog};
use crate::controllers::jump_environment::{new_runner_with_env, runner_elements};
use crate::controllers::training_jump::{TrainingJumpAction, TrainingJumpController};
use crate::jump::{JumpParticipant, JumpPolicy, JumpRunner};
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::palette::Palette;
use engine::ui::{Element, Event, View};
use std::cell::RefCell;

pub struct JumpView {
    resources: ResourcesRef,
    store: StoreRef,
    runner: RefCell<JumpRunner>,
    save_dialog: SaveReplayDialog,
}

impl JumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = store.practice.selected_hill.get();
        store.first_event.set(true);

        let runner = new_runner_with_env(
            hill_idx,
            store.practice.start_gate.get(),
            JumpParticipant::trainee(),
            JumpPolicy::training(),
            &resources,
            &store,
        );

        Self {
            resources: ResourcesRef::clone(&resources),
            store,
            runner: RefCell::new(runner),
            save_dialog: SaveReplayDialog::new(resources),
        }
    }

    fn enter_save_dialog(&self) {
        let hill_name = self
            .resources
            .hills
            .hill(self.runner.borrow().hill_idx())
            .map(|h| h.name.clone())
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
        self.save_dialog
            .open(author_name, format!("Huge Jump in {hill_name}"));
    }

    fn do_save_replay(&self) {
        let runner = self.runner.borrow();
        if let Some(trace) = runner.replay_trace() {
            self.save_dialog.write_replay(&trace);
        } else {
            // Session vanished; just close the dialog
        }
    }

    fn reset_jump_state(&self) {
        // Pascal: wind continues between jumps, NOT re-initialized (only F5 resets it)
        let hill_idx = self.runner.borrow().hill_idx();
        let record_distance = self
            .store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map_or(0, |r| r.len as i32);
        self.runner
            .borrow_mut()
            .reset_state(self.store.practice.start_gate.get(), record_distance);
    }

    fn reset_wind(&self) {
        {
            let mut rng = self.store.rng.borrow_mut();
            let mut wind = self.store.wind.borrow_mut();
            wind.initialize(&mut rng, self.store.wind_place.get());
        }
    }

    fn handle_jump_event(&self, event: Event) -> Option<RouteTarget> {
        let action = {
            let mut runner = self.runner.borrow_mut();
            TrainingJumpController.handle_event(event, runner.session_mut())
        };
        match action {
            TrainingJumpAction::None => None,
            TrainingJumpAction::RouteBack => Some(RouteTarget::Back),
            TrainingJumpAction::ResetWind => {
                self.reset_wind();
                None
            }
            TrainingJumpAction::ResetJump => {
                let _ = self.runner.borrow().outcome();
                let _ = self.runner.borrow().replay_trace();
                self.reset_jump_state();
                None
            }
            TrainingJumpAction::PersistStartGate(start_gate) => {
                self.store.practice.start_gate.set(start_gate);
                self.store.start_gate.set(start_gate);
                None
            }
            TrainingJumpAction::SaveReplay => {
                self.enter_save_dialog();
                None
            }
        }
    }
}

impl View<RouteTarget> for JumpView {
    fn elements(&self) -> Vec<Element> {
        if self.save_dialog.is_active() {
            let distance = self
                .runner
                .borrow()
                .outcome()
                .map(|o| format!("{:.1}", f64::from(o.distance) / 10.0))
                .unwrap_or_default();
            let hill_name = self
                .resources
                .hills
                .hill(self.runner.borrow().hill_idx())
                .map(|h| format!("{} K{}", h.name, h.kr))
                .unwrap_or_default();
            return self.save_dialog.elements(&distance, &hill_name);
        }
        runner_elements(&mut self.runner.borrow_mut(), &self.resources, &self.store)
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        if self.save_dialog.is_active() {
            match self.save_dialog.handle_event(event) {
                SaveAction::SaveReplay => self.do_save_replay(),
                SaveAction::Consumed => {}
            }
            None
        } else {
            self.handle_jump_event(event)
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if self.save_dialog.is_active() {
            return;
        }
        if let Ok(mut runner) = self.runner.try_borrow_mut() {
            let wind = self.store.wind.borrow().value;
            runner.render_snow(framebuffer, wind);
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if self.save_dialog.is_active() {
            return;
        }
        if let Ok(runner) = self.runner.try_borrow() {
            runner.apply_palette(palette);
        }
    }
}

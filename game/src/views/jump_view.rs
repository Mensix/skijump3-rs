use crate::components::save_replay_dialog::{SaveAction, SaveReplayDialog};
use crate::jump::{JumpConfig, JumpParticipant, JumpPolicy, JumpRunner};
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::training_jump_controller::{TrainingJumpAction, TrainingJumpController};
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
        let hill = resources.hills.hill(hill_idx).cloned();
        let terrain = resources.hill_terrain(hill_idx).map(|t| (*t).clone());

        let mut snow = SnowSystem::new();

        // Pascal: each new practice/competition round resets eka=true
        store.eka.set(true);

        if terrain.is_ok() && hill.is_some() {
            let mut rng = store.rng.borrow_mut();
            let mut wind = store.wind.borrow_mut();
            wind.initialize(&mut rng, store.wind_place.get());

            if store.eka.get() {
                // Pascal lines 1127-1131: snow LMaara calc + VieLmaara on first jump
                let lmaara = rng.random_i32(2) * rng.random_i32(256);
                let lmaara = if lmaara > 0 && lmaara < 40 {
                    lmaara + rng.random_i32(150)
                } else {
                    lmaara
                };
                let lmaara = if lmaara > 0 && rng.random_i32(4) == 0 {
                    lmaara + 1000
                } else {
                    lmaara
                };
                snow.set_count(lmaara as u16, &mut rng);

                // Pascal line 1149: Tuuli.Hae inside eka block (first wind shift)
                wind.sample(&mut rng);

                store.eka.set(false);
            } else {
                // Pascal: on subsequent jumps, snow persists (no re-init).
                // Initialize with fixed count so snow stays visible.
                snow.set_count(50, &mut rng);
            }
        }

        let record_distance = store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map(|r| r.len as i32)
            .unwrap_or(0);
        let config = JumpConfig {
            hill_idx,
            hill,
            terrain,
            start_gate: store.practice.start_gate.get(),
            snow,
            participant: JumpParticipant::trainee(),
            policy: JumpPolicy::training(),
            record_distance,
        };

        Self {
            resources: ResourcesRef::clone(&resources),
            store,
            runner: RefCell::new(JumpRunner::new(config)),
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
            .open(author_name, format!("Huge Jump in {}", hill_name));
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
            .map(|r| r.len as i32)
            .unwrap_or(0);
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

    fn handle_jump_event(&mut self, event: Event) -> Option<RouteTarget> {
        let action = {
            let mut runner = self.runner.borrow_mut();
            TrainingJumpController.handle_event(event, runner.session_mut())
        };
        match action {
            TrainingJumpAction::None => None,
            TrainingJumpAction::RoutePractice => Some(RouteTarget::Practice),
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
                .map(|o| format!("{:.1}", o.distance as f64 / 10.0))
                .unwrap_or_default();
            let hill_name = self
                .resources
                .hills
                .hill(self.runner.borrow().hill_idx())
                .map(|h| format!("{} K{}", h.name, h.kr))
                .unwrap_or_default();
            return self.save_dialog.elements(&distance, &hill_name);
        }
        self.runner
            .borrow_mut()
            .elements(&self.resources, &self.store)
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

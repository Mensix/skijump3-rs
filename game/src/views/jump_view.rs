use crate::components::save_replay_dialog::{SaveAction, SaveReplayDialog};
use crate::data::hill_profile::HillTerrain;
use crate::jump::presentation;
use crate::jump::{JumpPolicy, JumpPresentationContext, JumpSession, WindGaugePosition};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use crate::views::training_jump_controller::{TrainingJumpAction, TrainingJumpController};
use engine::consts::{HEIGHT, WIDTH};
use engine::palette::Palette;
use engine::ui::{Element, Event, View};
use std::cell::RefCell;

pub struct JumpView {
    resources: ResourcesRef,
    store: StoreRef,
    hill_idx: usize,
    session: RefCell<JumpSession>,
    jumper_name: String,
    save_dialog: SaveReplayDialog,
}

impl JumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = store.selected_hill.get();
        let hill = resources.hills.hill(hill_idx);
        let terrain = hill
            .ok_or_else(|| format!("Hill {} not found", hill_idx))
            .and_then(HillTerrain::load);

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

        let jumper_name = "TRAINEE".to_string();
        let record_distance = store
            .records
            .borrow()
            .hill_record(hill_idx)
            .map(|r| r.len as i32)
            .unwrap_or(0);
        let session = JumpSession::new(
            terrain,
            hill,
            hill_idx,
            store.start_gate.get(),
            snow,
            jumper_name.clone(),
            JumpPolicy::training(),
            record_distance,
        );

        Self {
            resources: ResourcesRef::clone(&resources),
            store,
            hill_idx,
            session: RefCell::new(session),
            jumper_name,
            save_dialog: SaveReplayDialog::new(resources),
        }
    }

    fn enter_save_dialog(&self) {
        let hill_name = self
            .resources
            .hills
            .hill(self.hill_idx)
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
        let session = self.session.borrow();
        if let Some(trace) = session.replay_trace() {
            self.save_dialog.write_replay(&trace);
        } else {
            // Session vanished; just close the dialog
        }
    }

    fn reset_jump_state(&self) {
        if let Some(hill) = self.resources.hills.hill(self.hill_idx) {
            // Pascal: wind continues between jumps, NOT re-initialized (only F5 resets it)
            let record_distance = self
                .store
                .records
                .borrow()
                .hill_record(self.hill_idx)
                .map(|r| r.len as i32)
                .unwrap_or(0);
            self.session.borrow_mut().reset_state(
                hill,
                self.store.start_gate.get(),
                record_distance,
            );
        }
    }

    fn reset_wind(&self) {
        {
            let mut rng = self.store.rng.borrow_mut();
            let mut wind = self.store.wind.borrow_mut();
            wind.initialize(&mut rng, self.store.wind_place.get());
        }
    }

    fn handle_jump_event(&mut self, event: Event) -> Option<RouteTarget> {
        match TrainingJumpController.handle_event(event, self.session.get_mut()) {
            TrainingJumpAction::None => None,
            TrainingJumpAction::RoutePractice => Some(RouteTarget::Practice),
            TrainingJumpAction::ResetWind => {
                self.reset_wind();
                None
            }
            TrainingJumpAction::ResetJump => {
                let _ = self.session.get_mut().outcome();
                let _ = self.session.get_mut().replay_trace();
                self.reset_jump_state();
                None
            }
            TrainingJumpAction::PersistStartGate(start_gate) => {
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

impl JumpView {
    #[allow(unused_mut)]
    fn build_elements(&self) -> Vec<Element> {
        let mut session = self.session.borrow_mut();
        let Err(err) = session.terrain() else {
            if session.state().is_some() {
                return self.elements_for_loaded_session(&mut session);
            }
            let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
            els.push(Element::text_color(
                "jump state not available",
                20,
                80,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
            return els;
        };

        let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
        els.push(Element::text_color(err, 20, 80, FONT_DEFAULT));
        els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
        els
    }
}

impl JumpView {
    fn elements_for_loaded_session(&self, session: &mut JumpSession) -> Vec<Element> {
        if session.phase().is_none() {
            let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
            els.push(Element::text_color(
                "jump state not available",
                20,
                80,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
            return els;
        }

        let mut rng = self.store.rng.borrow_mut();
        let mut wind_store = self.store.wind.borrow_mut();
        let wind = session.tick_with_wind(&mut rng, &mut wind_store);
        drop(wind_store);
        drop(rng);

        let hill_name_k = self
            .resources
            .hills
            .hill(self.hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();
        let wind_pos = self.store.wind.borrow().position();
        let records = self.store.records.borrow();
        let frame = session
            .render_frame(wind, WIDTH, HEIGHT)
            .expect("loaded jump render frame");
        let ctx = JumpPresentationContext {
            font: &self.resources.font,
            langbase: &self.resources.langbase,
            jumper_name: &self.jumper_name,
            hill_name_k: &hill_name_k,
            hill_record: records.hill_record(self.hill_idx),
            wind_position: WindGaugePosition {
                x: wind_pos.x,
                y: wind_pos.y,
            },
        };
        presentation::elements(&frame, &ctx)
    }
}

impl View<RouteTarget> for JumpView {
    fn elements(&self) -> Vec<Element> {
        if self.save_dialog.is_active() {
            let distance = self
                .session
                .borrow()
                .outcome()
                .map(|o| format!("{:.1}", o.distance as f64 / 10.0))
                .unwrap_or_default();
            let hill_name = self
                .resources
                .hills
                .hill(self.hill_idx)
                .map(|h| format!("{} K{}", h.name, h.kr))
                .unwrap_or_default();
            return self.save_dialog.elements(&distance, &hill_name);
        }
        self.build_elements()
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
        if let Ok(mut session) = self.session.try_borrow_mut() {
            let wind = self.store.wind.borrow().value;
            let draw = session.draws_snow();
            session.render_snow(framebuffer, wind, draw);
        }
    }

    fn apply_palette(&self, palette: &mut Palette) {
        if self.save_dialog.is_active() {
            return;
        }
        if let Ok(terrain) = self.session.borrow().terrain() {
            terrain.apply_hill_palette(palette);
        }
        palette.set(253, [10, 54, 10]);
        palette.set(254, [0, 47, 0]);
    }
}

use crate::data::hill_profile::HillTerrain;
use crate::jump::presentation;
use crate::jump::{
    FlightWind, JumpInput, JumpPhase, JumpPresentationContext, JumpSession, WindGaugePosition,
};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Event, Key, View};
use std::cell::RefCell;

pub struct JumpView {
    resources: ResourcesRef,
    store: StoreRef,
    hill_idx: usize,
    session: RefCell<JumpSession>,
    jumper_name: String,
}

impl JumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = *store.selected_hill.borrow();
        let hill = resources.hills.hill(hill_idx);
        let terrain = hill
            .ok_or_else(|| format!("Hill {} not found", hill_idx))
            .and_then(HillTerrain::load);

        let mut snow = SnowSystem::new();

        // Pascal: each new practice/competition round resets eka=true
        *store.eka.borrow_mut() = true;

        if terrain.is_ok() && hill.is_some() {
            let mut rng = store.rng.borrow_mut();
            let mut wind = store.wind.borrow_mut();
            wind.initialize(&mut rng, *store.wind_place.borrow());

            if *store.eka.borrow() {
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

                *store.eka.borrow_mut() = false;
            } else {
                // Pascal: on subsequent jumps, snow persists (no re-init).
                // Initialize with fixed count so snow stays visible.
                snow.set_count(50, &mut rng);
            }
        }

        let jumper_name = "TRAINEE".to_string();
        let session = JumpSession::new(
            terrain,
            hill,
            hill_idx,
            *store.start_gate.borrow(),
            snow,
            jumper_name.clone(),
        );

        Self {
            resources,
            store,
            hill_idx,
            session: RefCell::new(session),
            jumper_name,
        }
    }

    fn reset_jump_state(&self) {
        if let Some(hill) = self.resources.hills.hill(self.hill_idx) {
            // Pascal: wind continues between jumps, NOT re-initialized (only F5 resets it)
            self.session
                .borrow_mut()
                .reset_state(hill, *self.store.start_gate.borrow());
        }
    }

    fn reset_wind(&self) {
        {
            let mut rng = self.store.rng.borrow_mut();
            let mut wind = self.store.wind.borrow_mut();
            wind.initialize(&mut rng, *self.store.wind_place.borrow());
        }
    }
}

impl JumpView {
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
        let Some(phase) = session.state().map(|state| state.phase) else {
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

        let wind: FlightWind;
        if phase == JumpPhase::Result {
            let w = self.store.wind.borrow();
            wind = FlightWind {
                value: w.value,
                windy: w.windy,
                strength: w.strength,
            };
        } else if phase == JumpPhase::Info {
            // Pascal: info screen does NOT call Tuuli.Hae (line 1420 commented out)
            let w = self.store.wind.borrow();
            wind = FlightWind {
                value: w.value,
                windy: w.windy,
                strength: w.strength,
            };
            drop(w);
            session.tick(wind, &mut self.store.rng.borrow_mut());
        } else {
            // OnBar, Inrun, Flight: sample wind each frame (Pascal lines 1536, 1691)
            let mut rng = self.store.rng.borrow_mut();
            let mut w = self.store.wind.borrow_mut();
            let wind_value = w.sample(&mut rng);
            wind = FlightWind {
                value: wind_value,
                windy: w.windy,
                strength: w.strength,
            };
            drop(w);
            session.tick(wind, &mut rng);
        };
        if session
            .state()
            .is_some_and(|state| state.phase == JumpPhase::Result)
        {
            let mut rng = self.store.rng.borrow_mut();
            session.tick(wind, &mut rng);
        }

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
        self.build_elements()
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => Some(RouteTarget::Practice),
            Event::Keyboard(Key::F5) => {
                self.reset_wind();
                None
            }
            Event::Keyboard(Key::Enter) => {
                if self
                    .session
                    .get_mut()
                    .state()
                    .is_some_and(|state| state.phase == JumpPhase::Result)
                {
                    let _ = self.session.get_mut().outcome();
                    let _ = self.session.get_mut().replay_trace();
                    self.reset_jump_state();
                    return None;
                }

                if let Some(phase) = self.session.get_mut().state().map(|state| state.phase) {
                    if phase == JumpPhase::Info {
                        if let Some(state) = self.session.get_mut().state() {
                            *self.store.start_gate.borrow_mut() = state.start_gate;
                        }
                        self.session.get_mut().handle_input(JumpInput::LeaveInfo);
                        return None;
                    }
                    if phase == JumpPhase::Landing {
                        self.session.get_mut().handle_input(JumpInput::ShowResult);
                        return None;
                    }
                    self.session.get_mut().handle_input(JumpInput::Start);
                }
                None
            }
            Event::Keyboard(Key::Right) => {
                if let Some(phase) = self.session.get_mut().state().map(|state| state.phase) {
                    if phase == JumpPhase::Info {
                        if let Some(state) = self.session.get_mut().state() {
                            *self.store.start_gate.borrow_mut() = state.start_gate;
                        }
                        self.session.get_mut().handle_input(JumpInput::LeaveInfo);
                    } else if phase == JumpPhase::OnBar {
                        self.session.get_mut().handle_input(JumpInput::Start);
                    } else {
                        self.session.get_mut().handle_input(JumpInput::LeanForward);
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('+')) => {
                if let Some(state) = self.session.get_mut().state_mut() {
                    if state.phase == JumpPhase::Info {
                        state.handle_input(JumpInput::AdjustGate(1));
                        *self.store.start_gate.borrow_mut() = state.start_gate;
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('-')) => {
                if let Some(state) = self.session.get_mut().state_mut() {
                    if state.phase == JumpPhase::Info {
                        state.handle_input(JumpInput::AdjustGate(-1));
                        *self.store.start_gate.borrow_mut() = state.start_gate;
                    }
                }
                None
            }
            Event::Keyboard(Key::Left) => {
                if let Some(state) = self.session.get_mut().state_mut() {
                    if state.phase == JumpPhase::Flight {
                        state.handle_input(JumpInput::LeanBack);
                    }
                }
                None
            }
            Event::Keyboard(Key::Up) => {
                if let Some(state) = self.session.get_mut().state_mut() {
                    if state.phase == JumpPhase::Inrun {
                        state.handle_input(JumpInput::Takeoff);
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('t') | Key::Char('T')) => {
                if let Some(state) = self.session.get_mut().state_mut() {
                    if state.phase == JumpPhase::Flight {
                        state.handle_input(JumpInput::Telemark);
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('r') | Key::Char('R')) => {
                if let Some(state) = self.session.get_mut().state_mut() {
                    if state.phase == JumpPhase::Flight {
                        state.handle_input(JumpInput::TwoFooted);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if let Ok(mut session) = self.session.try_borrow_mut() {
            let wind = self.store.wind.borrow().value;
            let draw = session.state().is_some_and(|state| {
                matches!(
                    state.phase,
                    JumpPhase::Info | JumpPhase::OnBar | JumpPhase::Inrun | JumpPhase::Flight
                )
            });
            session.render_snow(framebuffer, wind, draw);
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        if let Ok(terrain) = self.session.borrow().terrain() {
            terrain.apply_hill_palette(palette);
        }
        // Pascal MuutaLogo(6) — green traffic light (overrides palette 253,254)
        palette.set(253, [10, 54, 10]);
        palette.set(254, [0, 47, 0]);
    }
}

use crate::data::hill_profile::HillTerrain;
use crate::jump::{FlightWind, JumpInput, JumpPhase, JumpSession};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Event, ImageRegion, Key, View};
use std::cell::RefCell;
use std::rc::Rc;

const FONT_DIM_TURQUOISE: u8 = 252;

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

    fn wind_elements(&self, els: &mut Vec<Element>, value: i32) {
        let position = self.store.wind.borrow().position();
        let x = position.x;
        let y = position.y;
        els.push(Element::fillbox(x + 4, y + 1, 35, 2, 248));
        els.push(Element::fillbox(x + 21, y + 1, 1, 2, 240));
        els.push(Element::fillbox(x + 21, y + 9, 1, 1, 247));
        if value > 0 {
            els.push(Element::fillbox(x + 22, y + 1, value / 3 + 1, 2, 236));
        }
        if value < 0 {
            let w = (-value) / 3 + 1;
            els.push(Element::fillbox(x + 21 - w, y + 1, w, 2, 237));
        }

        let text = format!("{:.1}", f64::from(value.abs()) / 10.0);
        if value < 0 {
            els.push(Element::text_color("-", x + 10, y + 5, FONT_GREET));
        }
        let mut chars = text.chars();
        if let Some(ones) = chars.next() {
            els.push(Element::text_color(
                ones.to_string(),
                x + 15,
                y + 5,
                FONT_GREET,
            ));
        }
        if let Some(tenths) = text.chars().nth(2) {
            els.push(Element::text_color(
                tenths.to_string(),
                x + 24,
                y + 5,
                FONT_GREET,
            ));
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

        let (terrain, state) = session
            .terrain_and_state_mut()
            .expect("loaded jump session state");

        let viewport = terrain.viewport_pixels(state.sx, state.sy, WIDTH, HEIGHT);
        let mut els = vec![Element::image_region(ImageRegion {
            pixels: Rc::clone(&viewport),
            src_w: WIDTH,
            src_h: HEIGHT,
            src_x: 0,
            src_y: 0,
            dst_x: 0,
            dst_y: 0,
            w: WIDTH,
            h: HEIGHT,
        })];

        let record_len = self
            .store
            .records
            .borrow()
            .hill_record(self.hill_idx)
            .map_or(0, |record| record.len);
        let hill_name_k = self
            .resources
            .hills
            .hill(self.hill_idx)
            .map(|h| format!("{} K{}", h.name, h.kr))
            .unwrap_or_default();

        if state.phase == JumpPhase::Info {
            els.push(Element::sprite(63, 227, 2));
            els.push(Element::sprite(64, 3, 150));
            els.push(Element::text_color_right(&hill_name_k, 308, 9, FONT_GOLD));
            els.push(Element::text_color_right(
                self.resources.langbase.lstr(65),
                308,
                19,
                FONT_GOLD,
            ));
            if record_len > 0 {
                if let Some(record) = self.store.records.borrow().hill_record(self.hill_idx) {
                    els.push(Element::text_color_right(&record.name, 308, 29, FONT_GOLD));
                    els.push(Element::text_color_right(
                        format!("{:.1}m", record.len as f64 / 10.0),
                        308,
                        39,
                        FONT_GOLD,
                    ));
                }
            }
            let label56 = self.resources.langbase.lstr(56);
            let label_w = self.resources.font.string_width(label56) as i32;
            let label58 = self.resources.langbase.lstr(58);
            let label58_w = self.resources.font.string_width(label58) as i32;
            els.push(Element::text_color(label58, 64, 19, FONT_DEFAULT));
            els.push(Element::text_color(
                format!("{}", state.start_gate),
                70 + label58_w,
                19,
                FONT_GOLD,
            ));
            els.push(Element::text_color("(+/-)", 67 + label58_w, 27, FONT_GREET));
            els.push(Element::text_color(
                self.resources.langbase.lstr(51),
                12,
                160,
                FONT_GREET,
            ));
            els.push(Element::text_color(label56, 12, 172, FONT_GREET));
            els.push(Element::text_color(
                &self.jumper_name,
                12 + label_w,
                172,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(
                self.resources.langbase.lstr(59),
                12,
                191,
                FONT_HELP,
            ));
        } else if state.phase == JumpPhase::Result {
            els.push(Element::sprite(63, 227, 2));
            els.push(Element::text_color_right(
                &self.jumper_name,
                308,
                9,
                FONT_DEFAULT,
            ));
            let style_min = *state.style_points.iter().min().unwrap_or(&0);
            let style_max = *state.style_points.iter().max().unwrap_or(&0);
            let mut found_min = false;
            let mut found_max = false;
            for (i, &point) in state.style_points.iter().enumerate() {
                let color = if point == style_min && !found_min {
                    found_min = true;
                    FONT_DIM_TURQUOISE
                } else if point == style_max && !found_max {
                    found_max = true;
                    FONT_DIM_TURQUOISE
                } else {
                    FONT_GREET
                };
                els.push(Element::text_color_right(
                    format!("{:.1}", f64::from(point) / 10.0),
                    308 - (i as i32) * 24,
                    21,
                    color,
                ));
            }
            els.push(Element::text_color_right(
                format!("{:.1}m", f64::from(state.distance) / 10.0),
                308,
                33,
                FONT_GREET,
            ));
            els.push(Element::text_color_right(
                format!("{:.1}", f64::from(state.score) / 10.0),
                308,
                45,
                FONT_GOLD,
            ));
            els.push(Element::text_color_right(
                self.resources.langbase.lstr(298),
                308,
                73,
                FONT_GREET,
            ));
        } else if state.phase == JumpPhase::Landing {
            els.push(Element::sprite(63, 227, 2));
            els.push(Element::text_color_right(
                &self.jumper_name,
                308,
                9,
                FONT_GREET,
            ));
            els.push(Element::text_color_right(
                format!("{:.1}m", f64::from(state.distance) / 10.0),
                308,
                33,
                FONT_GREET,
            ));
            for (i, &point) in state.style_points.iter().enumerate() {
                if state.style_revealed[i] {
                    els.push(Element::text_color_right(
                        format!("{:.1}", f64::from(point) / 10.0),
                        308 - (i as i32) * 24,
                        21,
                        FONT_GREET,
                    ));
                }
            }
        } else if state.phase == JumpPhase::Flight {
        }

        let (body_x, body_y) = state.body_position();
        let jumper_x = state.x - state.sx;
        let jumper_y = state.y - state.sy;
        if state.frame < 700
            && !matches!(
                state.phase,
                JumpPhase::Info | JumpPhase::Result | JumpPhase::Landing
            )
        {
            self.wind_elements(&mut els, wind.value);
        }
        if state.phase == JumpPhase::OnBar && (state.frame < 350 || (state.frame % 40) > 19) {
            els.push(Element::sprite(66, jumper_x + 60, jumper_y - 10));
        }
        let (body_anim, ski_anim) = state.anims(terrain);
        let replay_pos = (state.x, state.y);
        els.push(Element::sprite(
            body_anim,
            body_x - state.sx,
            body_y - state.sy - 2,
        ));
        els.push(Element::sprite(ski_anim, jumper_x, jumper_y - 1));
        session.record_render_frame(replay_pos, body_anim, ski_anim, wind.value);
        els
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

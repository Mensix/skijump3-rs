use crate::data::hill_profile::HillTerrain;
use crate::jump::animation::{
    flight_body_anim, flight_ski_anim, inrun_body_anim, landing_body_anim, slope_ski_anim,
    takeoff_body_anim,
};
use crate::palette_consts::*;
use crate::route::RouteTarget;
use crate::store::{ResourcesRef, StoreRef};
use engine::consts::{HEIGHT, WIDTH};
use engine::ui::{Element, Event, ImageRegion, Key, View};
use std::cell::RefCell;
use std::rc::Rc;

fn pascal_round(value: f64) -> i32 {
    if value >= 0.0 {
        (value + 0.5).floor() as i32
    } else {
        (value - 0.5).ceil() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_half_away_from_zero() {
        assert_eq!(pascal_round(0.5), 1);
        assert_eq!(pascal_round(1.5), 2);
        assert_eq!(pascal_round(2.5), 3);
        assert_eq!(pascal_round(37.5), 38);
        assert_eq!(pascal_round(38.5), 39);
        assert_eq!(pascal_round(-0.5), -1);
        assert_eq!(pascal_round(-1.5), -2);
        assert_eq!(pascal_round(0.0), 0);
        assert_eq!(pascal_round(0.1), 0);
        assert_eq!(pascal_round(0.9), 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JumpPhase {
    Info,
    OnBar,
    Inrun,
    Flight,
    Landing,
    Result,
}

#[derive(Debug, Clone)]
struct JumpState {
    phase: JumpPhase,
    matka: f64,
    px: f64,
    pxk: f64,
    base_maxspeed: f64,
    maxspeed: f64,
    distance_factor: f64,
    qx: f64,
    ramp_y: i32,
    vertical_pos: f64,
    vertical_speed: f64,
    flight_time: f64,
    lift: f64,
    body_angle: i32,
    ski_angle: i32,
    ski_swing: i32,
    landing_style: u8,
    height: i32,
    delta_height: [i32; 6],
    first_flight_frame: bool,
    distance: i32,
    landing_counter: i32,
    takeoff_requested: bool,
    lean_forward_requested: bool,
    lean_back_requested: bool,
    landing_requested: Option<u8>,
    x: i32,
    y: i32,
    sx: i32,
    sy: i32,
    frame: i32,
    info_counter: i32,
    start_gate: i32,
    takeoff_counter: u8,
    takeoff_phase: u8,
}

impl JumpState {
    fn new(
        terrain: &HillTerrain,
        maxspeed: f64,
        distance_factor: f64,
        lift: f64,
        start_gate: i32,
    ) -> Self {
        let matka = -f64::from(terrain.keula_x) + 10.0;
        let qx = f64::from(terrain.keula_x) + 0.5;
        let x = pascal_round(matka + qx);
        let y = terrain.profiili(x);
        let ramp_y = terrain.profiili(terrain.keula_x);
        Self {
            phase: JumpPhase::Info,
            matka,
            px: 0.0,
            pxk: 1.016,
            base_maxspeed: maxspeed,
            maxspeed,
            distance_factor,
            qx,
            ramp_y,
            vertical_pos: f64::from(y),
            vertical_speed: 0.0,
            flight_time: 0.0,
            lift,
            body_angle: 0,
            ski_angle: 0,
            ski_swing: 0,
            landing_style: 0,
            height: 0,
            delta_height: [0; 6],
            first_flight_frame: true,
            distance: 0,
            landing_counter: 0,
            takeoff_requested: false,
            lean_forward_requested: false,
            lean_back_requested: false,
            landing_requested: None,
            x,
            y,
            sx: 0,
            sy: 0,
            frame: 0,
            info_counter: 0,
            start_gate,
            takeoff_counter: 0,
            takeoff_phase: 0,
        }
    }

    fn adjust_start_gate(&mut self, delta: i32) {
        self.start_gate = (self.start_gate + delta).clamp(1, 30);
    }

    fn leave_info(&mut self) {
        if self.phase == JumpPhase::Info {
            self.maxspeed = self.base_maxspeed + f64::from(self.start_gate - 15);
            self.phase = JumpPhase::OnBar;
            self.frame = 0;
        }
    }

    fn start(&mut self) {
        if self.phase == JumpPhase::OnBar {
            self.phase = JumpPhase::Inrun;
            self.frame = 0;
        }
    }

    fn start_takeoff(&mut self) {
        self.takeoff_requested = true;
    }

    fn tick(&mut self, terrain: &HillTerrain) {
        self.frame += 1;
        match self.phase {
            JumpPhase::Inrun => self.tick_inrun(terrain),
            JumpPhase::Flight => self.tick_flight(terrain),
            JumpPhase::Landing => self.tick_landing(terrain),
            JumpPhase::Info => self.info_counter += 1,
            JumpPhase::OnBar | JumpPhase::Result => {}
        }
    }

    fn tick_inrun(&mut self, terrain: &HillTerrain) {
        let fx = self.x;
        let fy = self.y;
        self.matka += self.px * 0.01;
        self.x = pascal_round(self.matka + self.qx);

        if self.matka >= 0.0 {
            self.phase = JumpPhase::Flight;
            self.tick_flight_after_position_update(fx, fy, terrain);
            return;
        }

        self.y = terrain.profiili(self.x);
        self.vertical_pos = f64::from(self.y);

        if self.takeoff_counter > 0 {
            self.takeoff_counter = self.takeoff_counter.saturating_add(1);
        }
        if self.takeoff_requested && self.matka > -40.0 && self.takeoff_counter == 0 {
            self.takeoff_counter = 1;
        }
        self.takeoff_requested = false;

        if self.frame < 14 {
            self.px = 0.0;
        } else if self.frame < 28 {
            self.px = 37.0;
        }

        self.px = (self.px * self.pxk).min(self.maxspeed);

        if self.takeoff_counter > 0 {
            self.px += 0.21;
            self.vertical_speed += 1.21;
            self.body_angle += 12;
            if self.takeoff_counter > 16 {
                if self.takeoff_counter == 17 {
                    self.lift += 0.023;
                }
                self.lift += 0.013;
                self.vertical_speed -= 1.0;
                self.body_angle = 158;
            }
        }

        self.update_camera(fx, fy);
    }

    fn tick_flight(&mut self, terrain: &HillTerrain) {
        let fx = self.x;
        let fy = self.y;

        self.matka += self.px * 0.01;
        self.x = pascal_round(self.matka + self.qx);

        self.tick_flight_after_position_update(fx, fy, terrain);
    }

    fn tick_flight_after_position_update(&mut self, fx: i32, fy: i32, terrain: &HillTerrain) {
        if let Some(style) = self.landing_requested.take() {
            self.landing_style = style;
        }

        if self.lean_back_requested && self.body_angle <= 600 {
            self.body_angle += pascal_round(f64::from(self.body_angle) / 4.0);
        }
        if self.lean_forward_requested && self.landing_style == 0 && self.body_angle > 0 {
            self.body_angle -= pascal_round(f64::from(self.body_angle) / 5.0);
        }
        self.lean_back_requested = false;
        self.lean_forward_requested = false;

        if self.body_angle < 50 {
            self.lift += 0.0001 - f64::from(self.body_angle - 50) / 18_000.0;
        }
        self.lift -= (1.0 - f64::from(self.body_angle) / 900.0) / 1875.0;
        self.px -= (f64::from(self.body_angle) / 900.0) / 20.0;
        self.px += (245.0_f64.sqrt() - 16.0) / 400.0;
        self.flight_time += 0.01;
        self.lift = self.lift.max(0.105);

        if self.landing_style > 0 && self.body_angle < 600 {
            self.body_angle += 9 + (i32::from(self.landing_style) - 1) * 5;
            if self.lift < 1.0 {
                self.lift += 0.003;
            }
        }

        self.vertical_pos += (self.flight_time * self.flight_time * self.lift)
            - ((self.vertical_speed - 8.0) / 100.0);
        self.y = pascal_round(self.vertical_pos);

        self.update_ski_swing();

        if self.first_flight_frame {
            self.body_angle = 158;
            self.first_flight_frame = false;
            if self.takeoff_counter < 16 {
                self.ski_swing = 1;
            }
            if self.takeoff_counter > 16 {
                self.ski_swing = 4;
            }
            if self.takeoff_counter == 0 {
                self.ski_swing = 0;
            }
        }

        let prev_height = self.height;
        self.height = (terrain.profiili(self.x) - self.y).max(0);
        self.delta_height[(self.frame as usize) % 3] = prev_height - self.height;

        if self.height == 0 {
            self.distance = self.distance(terrain);
            self.phase = JumpPhase::Landing;
            self.landing_counter = 0;
        }

        self.update_camera(fx, fy);
    }

    fn tick_landing(&mut self, terrain: &HillTerrain) {
        let fx = self.x;
        let fy = self.y;

        self.landing_counter += 1;
        self.matka += self.px * 0.008;
        self.x = pascal_round(self.matka + self.qx);
        self.y = terrain.profiili(self.x);
        self.vertical_pos = f64::from(self.y);

        if self.landing_counter > 120 {
            self.phase = JumpPhase::Result;
        }

        self.update_camera(fx, fy);
    }

    fn update_ski_swing(&mut self) {
        if self.ski_swing <= 0 {
            return;
        }

        match self.ski_swing {
            1 => {
                if self.ski_angle == 0 {
                    self.ski_angle = -51 - (16 - i32::from(self.takeoff_counter)) * 6;
                    if self.ski_angle < -105 {
                        self.ski_angle = -105;
                    }
                } else {
                    self.ski_angle -= 4;
                }
                if self.ski_angle < (i32::from(self.takeoff_counter) - 16) * 14 {
                    self.ski_swing = 2;
                }
            }
            2 => {
                if self.ski_angle < 0 {
                    self.ski_angle += 2;
                }
                if self.ski_angle > 0 {
                    self.ski_angle = 0;
                }
            }
            4 => {
                if self.ski_angle == 0 {
                    self.ski_angle = 70 + (i32::from(self.takeoff_counter) - 16) * 6;
                    if self.ski_angle > 130 {
                        self.ski_angle = 130;
                    }
                } else {
                    self.ski_angle += 3;
                }
                if self.ski_angle > (i32::from(self.takeoff_counter) - 16) * 14 {
                    self.ski_swing = 5;
                }
            }
            5 => {
                if self.ski_angle > 0 {
                    self.ski_angle -= 1;
                }
                if self.ski_angle < 0 {
                    self.ski_angle = 0;
                }
            }
            _ => {}
        }

        if self.ski_angle == 0 {
            self.ski_swing = 0;
        }
    }

    fn update_camera(&mut self, fx: i32, fy: i32) {
        if self.x >= 160 && self.x < 864 {
            self.sx += self.x - fx;
        }
        if self.y >= 100 && self.y < 412 {
            self.sy += self.y - fy;
        }
        self.sx = self.sx.min(704);
        self.sy = self.sy.min(312);
    }

    fn distance(&self, _terrain: &HillTerrain) -> i32 {
        let vertical_delta = self.vertical_pos - f64::from(self.ramp_y);
        pascal_round(
            (self.matka * self.matka + vertical_delta * vertical_delta).sqrt()
                * self.distance_factor
                * 0.5,
        ) * 5
    }

    fn lean_forward(&mut self) {
        self.lean_forward_requested = true;
    }

    fn lean_back(&mut self) {
        self.lean_back_requested = true;
    }

    fn set_landing(&mut self, style: u8) {
        self.landing_requested = Some(style);
    }

    fn anims(&mut self, terrain: &HillTerrain) -> (u16, u16) {
        match self.phase {
            JumpPhase::Info => (163, slope_ski_anim(terrain.maki_kulma(self.x))),
            JumpPhase::OnBar => (163, slope_ski_anim(terrain.maki_kulma(self.x))),
            JumpPhase::Inrun => {
                let ski = slope_ski_anim(terrain.maki_kulma(self.x));
                if self.takeoff_counter > 0 {
                    (takeoff_body_anim(&mut self.takeoff_phase), ski)
                } else {
                    (inrun_body_anim(ski), ski)
                }
            }
            JumpPhase::Flight => {
                let body = if self.takeoff_counter > 0 && self.takeoff_phase < 25 {
                    takeoff_body_anim(&mut self.takeoff_phase)
                } else {
                    flight_body_anim(self.body_angle)
                };
                let ski = if self.height < 6 && self.matka > 20.0 {
                    if self.ski_angle == 0 {
                        self.ski_swing = 0;
                    }
                    slope_ski_anim(terrain.maki_kulma(self.x) / (self.height + 1))
                } else {
                    flight_ski_anim(self.ski_angle)
                };
                (body, ski)
            }
            JumpPhase::Landing | JumpPhase::Result => {
                let ski = slope_ski_anim(terrain.maki_kulma(self.x));
                (landing_body_anim(ski, self.landing_style), ski)
            }
        }
    }

    fn status(&self) -> &'static str {
        match self.phase {
            JumpPhase::Info => "INFO - +/- GATE, ENTER TO BAR",
            JumpPhase::OnBar => "ON BAR - ENTER/RIGHT TO START",
            JumpPhase::Inrun if self.takeoff_counter > 0 => "TAKEOFF",
            JumpPhase::Inrun => "INRUN - UP TO TAKE OFF",
            JumpPhase::Flight => "FLIGHT - LEFT/RIGHT, T/R LANDING",
            JumpPhase::Landing => "LANDING / OUTRUN",
            JumpPhase::Result => "RESULT - ENTER FOR HILLS",
        }
    }
}

pub struct JumpView {
    resources: ResourcesRef,
    store: StoreRef,
    hill_idx: usize,
    terrain: Result<HillTerrain, String>,
    state: RefCell<Option<JumpState>>,
}

impl JumpView {
    pub fn new(resources: ResourcesRef, store: StoreRef) -> Self {
        let hill_idx = *store.selected_hill.borrow();
        let hill = resources.hills.hill(hill_idx);
        let terrain = hill
            .ok_or_else(|| format!("Hill {} not found", hill_idx))
            .and_then(HillTerrain::load);
        if terrain.is_ok() && hill.is_some() {
            let mut rng = store.rng.borrow_mut();
            let mut wind = store.wind.borrow_mut();
            wind.initialize(&mut rng, *store.wind_place.borrow());
        }
        let state = match (&terrain, hill) {
            (Ok(terrain), Some(hill)) => Some(JumpState::new(
                terrain,
                hill.vx_final as f64,
                hill.pk(),
                hill.pl_save(),
                *store.start_gate.borrow(),
            )),
            _ => None,
        };

        Self {
            resources,
            store,
            hill_idx,
            terrain,
            state: RefCell::new(state),
        }
    }

    fn new_state(&self) -> Option<JumpState> {
        let terrain = self.terrain.as_ref().ok()?;
        let hill = self.resources.hills.hill(self.hill_idx)?;
        self.reset_wind();
        Some(JumpState::new(
            terrain,
            hill.vx_final as f64,
            hill.pk(),
            hill.pl_save(),
            *self.store.start_gate.borrow(),
        ))
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
            els.push(Element::text_color("-", x + 10, y + 5, FONT_DEFAULT));
        }
        let mut chars = text.chars();
        if let Some(ones) = chars.next() {
            els.push(Element::text_color(
                ones.to_string(),
                x + 15,
                y + 5,
                FONT_DEFAULT,
            ));
        }
        if let Some(tenths) = text.chars().nth(2) {
            els.push(Element::text_color(
                tenths.to_string(),
                x + 24,
                y + 5,
                FONT_DEFAULT,
            ));
        }
    }

    fn hill_header(&self, els: &mut Vec<Element>) {
        if let Some(hill) = self.resources.hills.hill(self.hill_idx) {
            els.push(Element::text_color(
                format!("{} K{}", hill.name, hill.kr),
                8,
                8,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(
                format!(
                    "FRONT{}  VX{}  PK{:.2}  PL{:.4}",
                    hill.front_index,
                    hill.vx_final,
                    hill.pk(),
                    hill.pl_save()
                ),
                8,
                18,
                FONT_HELP,
            ));
        }
    }
}

impl View<RouteTarget> for JumpView {
    fn elements(&self) -> Vec<Element> {
        let Ok(terrain) = &self.terrain else {
            let mut els = vec![Element::fillbox(0, 0, WIDTH as i32, HEIGHT as i32, 0)];
            if let Err(err) = &self.terrain {
                els.push(Element::text_color(err, 20, 80, FONT_DEFAULT));
            }
            els.push(Element::text_color("PRESS ESC", 20, 95, FONT_HELP));
            return els;
        };

        let mut state_ref = self.state.borrow_mut();
        let state = state_ref.as_mut().expect("terrain-loaded jump state");
        let wind_value = if state.phase == JumpPhase::Result {
            self.store.wind.borrow().value
        } else {
            let mut rng = self.store.rng.borrow_mut();
            let mut wind = self.store.wind.borrow_mut();
            wind.sample(&mut rng)
        };
        state.tick(terrain);

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

        self.hill_header(&mut els);
        els.push(Element::text_color(
            format!("KEULAX {}", terrain.keula_x),
            8,
            28,
            FONT_HELP,
        ));

        els.push(Element::text_color(state.status(), 8, 38, FONT_HELP));
        if state.phase == JumpPhase::Info {
            els.push(Element::text_color(
                format!(
                    "{} {} (+/-)",
                    self.resources.langbase.lstr(58),
                    state.start_gate
                ),
                64,
                19,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(
                "PRESS ENTER/RIGHT TO CONTINUE",
                12,
                188,
                FONT_HELP,
            ));
        } else if state.phase == JumpPhase::Result {
            els.push(Element::text_color(
                format!("DISTANCE {:.1}m", f64::from(state.distance) / 10.0),
                8,
                50,
                FONT_DEFAULT,
            ));
        } else if state.phase == JumpPhase::Flight {
            els.push(Element::text_color(
                format!("ANGLE {} HEIGHT {}", state.body_angle, state.height),
                8,
                50,
                FONT_HELP,
            ));
        }

        let jumper_x = state.x - state.sx;
        let jumper_y = state.y - state.sy;
        if state.frame < 700 && state.phase != JumpPhase::Result {
            self.wind_elements(&mut els, wind_value);
        }
        let (body_anim, ski_anim) = state.anims(terrain);
        els.push(Element::sprite(body_anim, jumper_x, jumper_y - 2));
        els.push(Element::sprite(ski_anim, jumper_x, jumper_y - 1));
        els.push(Element::text_color("ESC: HILLS", 8, 188, FONT_HELP));
        els
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
                    .state
                    .get_mut()
                    .as_ref()
                    .is_some_and(|state| state.phase == JumpPhase::Result)
                {
                    *self.state.get_mut() = self.new_state();
                    return None;
                }

                if let Some(state) = self.state.get_mut() {
                    if state.phase == JumpPhase::Info {
                        *self.store.start_gate.borrow_mut() = state.start_gate;
                        state.leave_info();
                        return None;
                    }
                    if state.phase == JumpPhase::Landing {
                        state.phase = JumpPhase::Result;
                        return None;
                    }
                    state.start();
                }
                None
            }
            Event::Keyboard(Key::Right) => {
                if let Some(state) = self.state.get_mut() {
                    if state.phase == JumpPhase::Info {
                        *self.store.start_gate.borrow_mut() = state.start_gate;
                        state.leave_info();
                    } else if state.phase == JumpPhase::OnBar {
                        state.start();
                    } else {
                        state.lean_forward();
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('+')) => {
                if let Some(state) = self.state.get_mut() {
                    if state.phase == JumpPhase::Info {
                        state.adjust_start_gate(1);
                        *self.store.start_gate.borrow_mut() = state.start_gate;
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('-')) => {
                if let Some(state) = self.state.get_mut() {
                    if state.phase == JumpPhase::Info {
                        state.adjust_start_gate(-1);
                        *self.store.start_gate.borrow_mut() = state.start_gate;
                    }
                }
                None
            }
            Event::Keyboard(Key::Left) => {
                if let Some(state) = self.state.get_mut() {
                    state.lean_back();
                }
                None
            }
            Event::Keyboard(Key::Up) => {
                if let Some(state) = self.state.get_mut() {
                    state.start_takeoff();
                }
                None
            }
            Event::Keyboard(Key::Char('t') | Key::Char('T')) => {
                if let Some(state) = self.state.get_mut() {
                    state.set_landing(1);
                }
                None
            }
            Event::Keyboard(Key::Char('r') | Key::Char('R')) => {
                if let Some(state) = self.state.get_mut() {
                    state.set_landing(2);
                }
                None
            }
            _ => None,
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        if let Ok(terrain) = &self.terrain {
            terrain.apply_hill_palette(palette);
        }
    }
}

use crate::data::hill_profile::HillTerrain;
use crate::jump::animation::{
    crash_risk, fall_body_anim, flight_body_anim, flight_ski_anim, inrun_body_anim,
    landing_body_anim, post_landing_body_anim, slope_ski_anim, takeoff_body_anim,
};
use crate::palette_consts::*;
use crate::pascal_random::PascalRandom;
use crate::route::RouteTarget;
use crate::snow::SnowSystem;
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

fn nsqrt(value: f64) -> f64 {
    let root = value.abs().sqrt();
    if value < 0.0 {
        -root
    } else {
        root
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlightWind {
    value: i32,
    windy: i32,
    strength: i32,
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
    hill_kr: i32,
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
    fall_type: u8,
    grade: i32,
    start_anim: i32,
    style_base: i32,
    style_points: [i32; 5],
    score: i32,
    skis_stuck: bool,
    detached_matka: f64,
    detached_vertical_pos: f64,
    detached_px: f64,
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
        hill_kr: i32,
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
            hill_kr,
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
            fall_type: 0,
            grade: 0,
            start_anim: 100,
            style_base: 195,
            style_points: [0; 5],
            score: 0,
            skis_stuck: false,
            detached_matka: matka,
            detached_vertical_pos: f64::from(y),
            detached_px: 0.0,
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

    fn tick(&mut self, terrain: &HillTerrain, wind: FlightWind, rng: &mut PascalRandom) {
        self.frame += 1;
        match self.phase {
            JumpPhase::Inrun => self.tick_inrun(terrain, wind, rng),
            JumpPhase::Flight => self.tick_flight(terrain, wind, rng),
            JumpPhase::Landing => self.tick_landing(terrain),
            JumpPhase::Info => self.info_counter += 1,
            JumpPhase::OnBar | JumpPhase::Result => {}
        }
    }

    fn tick_inrun(&mut self, terrain: &HillTerrain, wind: FlightWind, rng: &mut PascalRandom) {
        let fx = self.x;
        let fy = self.y;
        self.matka += self.px * 0.01;
        self.x = pascal_round(self.matka + self.qx);

        if self.matka >= 0.0 {
            self.phase = JumpPhase::Flight;
            self.tick_flight_after_position_update(fx, fy, terrain, wind, Some(rng));
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

    fn tick_flight(&mut self, terrain: &HillTerrain, wind: FlightWind, rng: &mut PascalRandom) {
        let fx = self.x;
        let fy = self.y;

        self.matka += self.px * 0.01;
        self.x = pascal_round(self.matka + self.qx);

        self.tick_flight_after_position_update(fx, fy, terrain, wind, Some(rng));
    }

    fn tick_flight_after_position_update(
        &mut self,
        fx: i32,
        fy: i32,
        terrain: &HillTerrain,
        wind: FlightWind,
        mut rng: Option<&mut PascalRandom>,
    ) {
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
        if wind.value > 0 {
            self.lift -= (1.0 - f64::from(self.body_angle) / 900.0) / 1875.0;
            self.lift += f64::from(2 * wind.value).sqrt().sqrt() / 65_500.0;
        } else {
            self.lift -= (1.0 - f64::from(self.body_angle) / 900.0) / 1875.0;
            self.lift -= f64::from(-2 * wind.value).sqrt().sqrt() / 65_500.0;
        }
        self.px -= (f64::from(self.body_angle) / 900.0) / 20.0;
        self.px += (nsqrt(f64::from(4 * wind.value + 245)) - 16.0) / 400.0;
        self.flight_time += 0.01;

        if let Some(rng) = rng.as_deref_mut() {
            if rng.random_i32(30_000) < wind.windy + 10 + wind.strength {
                self.frame = 0;
                if rng.random_i32(2) == 1 {
                    self.style_base -= 5;
                }
                let gust_angle = rng.random_i32(15) - 6;
                self.body_angle += gust_angle;
                if gust_angle > 0 {
                    self.ski_swing = 3;
                    self.lift -= f64::from(rng.random_i32(wind.strength + 50)) / 15_000.0;
                }
                if gust_angle < 0 {
                    self.ski_swing = 6;
                    self.lift += f64::from(rng.random_i32(wind.strength + 50)) / 15_000.0;
                }
            }
        }

        self.lift = self.lift.max(0.105);

        self.vertical_pos += (self.flight_time * self.flight_time * self.lift)
            - ((self.vertical_speed - 8.0) / 100.0);
        self.y = pascal_round(self.vertical_pos);

        self.update_ski_swing(&mut rng);

        if self.landing_style > 0 && self.body_angle < 600 {
            self.body_angle += 9 + (i32::from(self.landing_style) - 1) * 5;
            if self.lift < 1.0 {
                self.lift += 0.003;
            }
        }

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
            if let Some(rng) = rng.as_mut() {
                self.prepare_landing(terrain, rng);
            }
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
        self.detached_matka += self.detached_px * 0.008;
        if self.skis_stuck {
            self.matka = self.detached_matka;
        }
        self.x = pascal_round(self.matka + self.qx);
        self.y = terrain.profiili(self.x);
        self.vertical_pos = f64::from(self.y);
        self.detached_vertical_pos =
            f64::from(terrain.profiili(pascal_round(self.detached_matka + self.qx)));

        if self.fall_type > 0 {
            if self.landing_counter > 50 && self.detached_px > 0.0 {
                self.detached_px -= 0.8;
            }
            if self.detached_px < 0.0 {
                self.detached_px = 0.0;
                if self.skis_stuck {
                    self.phase = JumpPhase::Result;
                }
            }
        }

        if self.x > 1050 {
            self.phase = JumpPhase::Result;
        }

        self.update_camera(fx, fy);
    }

    fn prepare_landing(&mut self, terrain: &HillTerrain, rng: &mut PascalRandom) {
        self.grade = if self.hill_kr != 0 {
            pascal_round(f64::from(self.distance) / f64::from(self.hill_kr)) * 10
        } else {
            0
        };

        let slope_angle = terrain.maki_kulma(self.x);
        let landing_quality =
            pascal_round(f64::from(slope_angle) * 1.34 + f64::from(self.body_angle) / 10.0);
        let mut risk = crash_risk(slope_angle) as i32;
        if f64::from(self.distance) < (20.0 / 3.0) * f64::from(self.hill_kr) {
            risk = 1;
        }
        if landing_quality < 63 {
            risk = pascal_round(f64::from(risk) * (1.0 + f64::from(63 - landing_quality) * 0.075));
        }

        if self.landing_style == 0 || landing_quality < 56 {
            self.fall_type = if self.landing_style == 0 { 1 } else { 2 };
        }
        if self.landing_style == 1 {
            risk *= 3;
            if landing_quality < 60 {
                self.style_base -= 5;
            }
            if landing_quality < 64 {
                self.style_base -= 5;
            }
        }
        if rng.random_i32(1000) < risk {
            self.fall_type = 3;
        }

        self.calculate_score(rng);

        if self.fall_type > 0 {
            self.grade = i32::from(self.fall_type);
        }
        self.skis_stuck = rng.random_i32(2) != 0;
        self.start_anim = if self.landing_style == 2 { 50 } else { 100 };
        self.detached_matka = self.matka;
        self.detached_vertical_pos = self.vertical_pos;
        self.detached_px = self.px;
    }

    fn calculate_score(&mut self, rng: &mut PascalRandom) {
        let mut base = self.style_base;
        let short_jump_penalty_count = pascal_round(
            (f64::from(self.hill_kr) + f64::from(self.hill_kr) / 20.0
                - (f64::from(self.distance) / 10.0))
                / 6.0,
        );
        if short_jump_penalty_count > 0 {
            base -= short_jump_penalty_count * 5;
        }

        if self.fall_type > 0 {
            base -= 100;
        } else if self.landing_style == 2 {
            base -= 15 + rng.random_i32(2) * 5;
        }

        self.style_points[0] = base;
        for i in 1..5 {
            let temp = rng.random_i32(4);
            self.style_points[i] = base - (temp - 1) * 5;
        }

        let mut min_style = 200;
        let mut max_style = 0;
        for point in &mut self.style_points {
            *point = (*point).clamp(0, 200);
            min_style = min_style.min(*point);
            max_style = max_style.max(*point);
        }

        self.score = self.style_points.iter().sum::<i32>() - min_style - max_style;
        if self.hill_kr != 0 {
            self.score += pascal_round(
                ((f64::from(self.distance) / 10.0) - (f64::from(self.hill_kr) * 2.0 / 3.0))
                    * (180.0 / f64::from(self.hill_kr))
                    * 10.0,
            );
        }
        self.style_base = base;
    }

    fn update_ski_swing(&mut self, rng: &mut Option<&mut PascalRandom>) {
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
            3 => {
                self.ski_angle -= rng.as_deref_mut().map_or(0, |rng| rng.random_i32(50)) + 30;
                self.ski_swing = 2;
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
            6 => {
                self.ski_angle += rng.as_deref_mut().map_or(0, |rng| rng.random_i32(50)) + 30;
                self.ski_swing = 5;
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

    fn landing_body_anim_for_state(&self, terrain: &HillTerrain) -> u16 {
        let detached_x = pascal_round(self.detached_matka + self.qx);
        let detached_ski = slope_ski_anim(terrain.maki_kulma(detached_x));
        if self.fall_type > 0 {
            fall_body_anim(
                self.fall_type,
                self.landing_counter,
                self.body_angle,
                detached_ski,
                self.landing_style,
            )
        } else if self.phase == JumpPhase::Landing {
            post_landing_body_anim(
                self.landing_counter,
                self.start_anim,
                self.landing_style,
                self.grade,
            )
        } else {
            landing_body_anim(detached_ski, self.landing_style)
        }
    }

    fn body_position(&self) -> (i32, i32) {
        if self.phase == JumpPhase::Landing {
            (
                pascal_round(self.detached_matka + self.qx),
                pascal_round(self.detached_vertical_pos),
            )
        } else {
            (self.x, self.y)
        }
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
                (self.landing_body_anim_for_state(terrain), ski)
            }
        }
    }
}

pub struct JumpView {
    resources: ResourcesRef,
    store: StoreRef,
    hill_idx: usize,
    terrain: Result<HillTerrain, String>,
    state: RefCell<Option<JumpState>>,
    jumper_name: String,
    snow: RefCell<SnowSystem>,
    prev_camera: RefCell<(i32, i32)>,
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
                hill.kr as i32,
                hill.pl_save(),
                *store.start_gate.borrow(),
            )),
            _ => None,
        };

        let jumper_name = "TRAINEE".to_string();

        let mut snow = SnowSystem::new();
        snow.set_count(50, &mut store.rng.borrow_mut());

        let camera = match &state {
            Some(s) => (s.sx, s.sy),
            None => (0, 0),
        };

        Self {
            resources,
            store,
            hill_idx,
            terrain,
            state: RefCell::new(state),
            jumper_name,
            snow: RefCell::new(snow),
            prev_camera: RefCell::new(camera),
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
            hill.kr as i32,
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
        let wind = if state.phase == JumpPhase::Result {
            let wind = self.store.wind.borrow();
            FlightWind {
                value: wind.value,
                windy: wind.windy,
                strength: wind.strength,
            }
        } else {
            let mut rng = self.store.rng.borrow_mut();
            let mut wind = self.store.wind.borrow_mut();
            let wind_value = wind.sample(&mut rng);
            let wind_frame = FlightWind {
                value: wind_value,
                windy: wind.windy,
                strength: wind.strength,
            };
            drop(wind);
            state.tick(terrain, wind_frame, &mut rng);
            wind_frame
        };
        if state.phase == JumpPhase::Result {
            let mut rng = self.store.rng.borrow_mut();
            state.tick(terrain, wind, &mut rng);
        }

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
            let label58_w = self.resources.font.string_width(&label58) as i32;
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
            els.push(Element::text_color_right(
                &self.jumper_name,
                308,
                9,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(
                format!("DISTANCE {:.1}m", f64::from(state.distance) / 10.0),
                8,
                50,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color(
                format!("POINTS {:.1}", f64::from(state.score) / 10.0),
                8,
                60,
                FONT_DEFAULT,
            ));
            let style_min = *state.style_points.iter().min().unwrap_or(&0);
            let style_max = *state.style_points.iter().max().unwrap_or(&0);
            let mut found_min = false;
            let mut found_max = false;
            for (i, &point) in state.style_points.iter().enumerate() {
                let color = if point == style_min && !found_min {
                    found_min = true;
                    FONT_GREET
                } else if point == style_max && !found_max {
                    found_max = true;
                    FONT_GREET
                } else {
                    FONT_HELP
                };
                els.push(Element::text_color_right(
                    format!("{:.1}", f64::from(point) / 10.0),
                    308 - (i as i32) * 24,
                    21,
                    color,
                ));
            }
            if state.fall_type > 0 {
                els.push(Element::text_color(
                    format!("FALL {}", state.fall_type),
                    8,
                    80,
                    FONT_HELP,
                ));
            }
            if record_len > 0 {
                els.push(Element::text_color(
                    format!("HILL RECORD {:.1}m", record_len as f64 / 10.0),
                    8,
                    90,
                    FONT_GOLD,
                ));
                if state.fall_type == 0 && i64::from(state.distance) > record_len {
                    els.push(Element::text_color(
                        "TRAINING JUMP - RECORD NOT SAVED",
                        8,
                        100,
                        FONT_HELP,
                    ));
                }
            }
        } else if state.phase == JumpPhase::Landing {
            els.push(Element::sprite(63, 227, 2));
            els.push(Element::text_color_right(
                &self.jumper_name,
                308,
                9,
                FONT_DEFAULT,
            ));
            els.push(Element::text_color_right(
                format!("{:.1}m", f64::from(state.distance) / 10.0),
                308,
                33,
                FONT_DEFAULT,
            ));
            for (i, &point) in state.style_points.iter().enumerate() {
                els.push(Element::text_color_right(
                    format!("{:.1}", f64::from(point) / 10.0),
                    308 - (i as i32) * 24,
                    21,
                    FONT_DEFAULT,
                ));
            }
            if state.fall_type > 0 {
                els.push(Element::text_color(
                    format!("FALL {}", state.fall_type),
                    8,
                    80,
                    FONT_HELP,
                ));
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
            els.push(Element::sprite(67, jumper_x + 60, jumper_y - 10));
        }
        let (body_anim, ski_anim) = state.anims(terrain);
        els.push(Element::sprite(
            body_anim,
            body_x - state.sx,
            body_y - state.sy - 2,
        ));
        els.push(Element::sprite(ski_anim, jumper_x, jumper_y - 1));
        if state.phase == JumpPhase::Result {
            els.push(Element::text_color(
                "ENTER: AGAIN  ESC: HILLS",
                8,
                188,
                FONT_HELP,
            ));
        }
        els
    }

    fn handle_event(&mut self, event: Event) -> Option<RouteTarget> {
        match event {
            Event::Keyboard(Key::Escape) => {
                if self
                    .state
                    .get_mut()
                    .as_ref()
                    .is_some_and(|state| state.phase == JumpPhase::Result)
                {
                    Some(RouteTarget::Practice)
                } else {
                    if let Some(state) = self.state.get_mut() {
                        state.phase = JumpPhase::Result;
                    }
                    None
                }
            }
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
                    if state.phase == JumpPhase::Flight {
                        state.lean_back();
                    }
                }
                None
            }
            Event::Keyboard(Key::Up) => {
                if let Some(state) = self.state.get_mut() {
                    if state.phase == JumpPhase::Inrun {
                        state.start_takeoff();
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('t') | Key::Char('T')) => {
                if let Some(state) = self.state.get_mut() {
                    if state.phase == JumpPhase::Flight {
                        state.set_landing(1);
                    }
                }
                None
            }
            Event::Keyboard(Key::Char('r') | Key::Char('R')) => {
                if let Some(state) = self.state.get_mut() {
                    if state.phase == JumpPhase::Flight {
                        state.set_landing(2);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn render_snow(&self, framebuffer: &mut [u8]) {
        if let Ok(ref state) = self.state.try_borrow() {
            if let Some(state) = state.as_ref() {
                let prev = self.prev_camera.replace((state.sx, state.sy));
                let delta_x = prev.0 - state.sx;
                let delta_y = prev.1 - state.sy;
                let wind = self.store.wind.borrow().value;
                let draw = matches!(
                    state.phase,
                    JumpPhase::Info | JumpPhase::OnBar | JumpPhase::Inrun | JumpPhase::Flight
                );
                self.snow
                    .borrow_mut()
                    .update(framebuffer, delta_x, delta_y, wind, draw);
            }
        }
    }

    fn apply_palette(&self, palette: &mut engine::palette::Palette) {
        if let Ok(terrain) = &self.terrain {
            terrain.apply_hill_palette(palette);
        }
    }
}

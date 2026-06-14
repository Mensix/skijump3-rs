use crate::data::hill_profile::HillTerrain;
use crate::gfx::sprites::Sprite;
use crate::jump::animation::{
    fall_body_anim, flight_body_anim, flight_ski_anim, inrun_body_anim, inrun_transition_body_anim,
    landing_body_anim, post_landing_body_anim, slope_ski_anim, takeoff_body_anim,
};
use crate::jump::math::{self, nsqrt};
use crate::jump::scoring;
use crate::jump::types::{
    FallType, FlightWind, JumpInput, JumpOutcome, JumpPhase, JumpSnapshot, LandingStyle, SkiSwing,
};
use crate::rng::Random;
use crate::text::format::tenths_to_decimal;

#[derive(Debug, Clone)]
pub struct JumpState {
    pub(crate) phase: JumpPhase,
    pub(crate) travel: f64,
    pub(crate) px: f64,
    pub(crate) pxk: f64,
    pub(crate) base_maxspeed: f64,
    pub(crate) maxspeed: f64,
    pub(crate) distance_factor: f64,
    pub(crate) hill_kr: i32,
    pub(crate) qx: f64,
    pub(crate) ramp_y: i32,
    pub(crate) vertical_pos: f64,
    pub(crate) vertical_speed: f64,
    pub(crate) flight_time: f64,
    pub(crate) lift: f64,
    pub(crate) body_angle: i32,
    pub(crate) ski_angle: i32,
    pub(crate) ski_swing: SkiSwing,
    pub(crate) landing_style: LandingStyle,
    pub(crate) height: i32,
    pub(crate) delta_height: [i32; 6],
    pub(crate) first_flight_frame: bool,
    pub(crate) distance: f64,
    pub(crate) landing_counter: i32,
    pub(crate) fall_type: FallType,
    pub(crate) grade: i32,
    pub(crate) start_anim: i32,
    pub(crate) style_base: i32,
    pub(crate) style_points: [i32; 5],
    pub(crate) style_revealed: [bool; 5],
    pub(crate) score: i32,
    pub(crate) skis_stuck: bool,
    pub(crate) detached_travel: f64,
    pub(crate) detached_vertical_pos: f64,
    pub(crate) detached_px: f64,
    pub(crate) takeoff_requested: bool,
    pub(crate) lean_forward_requested: bool,
    pub(crate) lean_back_requested: bool,
    pub(crate) landing_requested: Option<LandingStyle>,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) sx: i32,
    pub(crate) sy: i32,
    pub(crate) frame: i32,
    pub(crate) info_counter: i32,
    pub(crate) start_gate: i32,
    result_pending: bool,
    pub(crate) takeoff_counter: u8,
    pub(crate) takeoff_phase: u8,
    silent_computer: bool,
}

impl JumpState {
    pub(crate) fn new(
        terrain: &HillTerrain,
        maxspeed: f64,
        distance_factor: f64,
        hill_kr: i32,
        lift: f64,
        start_gate: i32,
    ) -> Self {
        let travel = -f64::from(terrain.tip_x) + 10.0;
        let qx = f64::from(terrain.tip_x) + 0.5;
        let x = math::round(travel + qx);
        let y = terrain.height_at(x);
        let ramp_y = terrain.height_at(terrain.tip_x);
        Self {
            phase: JumpPhase::Info,
            travel,
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
            ski_swing: SkiSwing::None,
            landing_style: LandingStyle::None,
            height: 0,
            delta_height: [0; 6],
            first_flight_frame: true,
            distance: 0.0,
            landing_counter: 0,
            fall_type: FallType::None,
            grade: 0,
            start_anim: 100,
            style_base: 195,
            style_points: [0; 5],
            style_revealed: [false; 5],
            score: 0,
            skis_stuck: false,
            detached_travel: travel,
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
            result_pending: false,
            takeoff_counter: 0,
            takeoff_phase: 0,
            silent_computer: false,
        }
    }

    pub(crate) fn handle_input(&mut self, input: JumpInput) {
        match input {
            JumpInput::LeaveInfo => self.leave_info(),
            JumpInput::Start => self.start(),
            JumpInput::Takeoff => self.start_takeoff(),
            JumpInput::LeanForward => self.lean_forward(),
            JumpInput::LeanBack => self.lean_back(),
            JumpInput::Telemark => self.set_landing(LandingStyle::Telemark),
            JumpInput::TwoFooted => self.set_landing(LandingStyle::TwoFooted),
            JumpInput::AdjustGate(delta) => self.adjust_start_gate(delta),
            JumpInput::ShowResult => {
                if self.phase == JumpPhase::Landing {
                    self.phase = JumpPhase::Result;
                }
            }
        }
    }

    pub(crate) fn outcome(&self) -> Option<JumpOutcome> {
        match self.phase {
            JumpPhase::Result => Some(JumpOutcome {
                distance: self.distance,
                score: tenths_to_decimal(self.score),
                style_points: self.style_points.map(tenths_to_decimal),
                landing_style: self.landing_style,
                fall_type: self.fall_type,
                aborted: false,
            }),
            JumpPhase::Disqualified => Some(JumpOutcome {
                distance: 0.0,
                score: 0.0,
                style_points: [0.0; 5],
                landing_style: LandingStyle::Telemark,
                fall_type: FallType::None,
                aborted: false,
            }),
            _ => None,
        }
    }

    pub(crate) const fn snapshot(&self) -> JumpSnapshot {
        JumpSnapshot {
            phase: self.phase,
            frame: self.frame,
            x: self.x,
            table_distance: self.travel,
            y: self.y,
            height: self.height,
            delta_height_sum: self.delta_height[0] + self.delta_height[1] + self.delta_height[2],
            slope_angle: 0,
            distance: self.distance,
            body_angle: self.body_angle,
            ski_angle: self.ski_angle,
            speed: self.px,
            start_gate: self.start_gate,
        }
    }

    pub(crate) fn snapshot_with_terrain(&self, terrain: &HillTerrain) -> JumpSnapshot {
        let mut snapshot = self.snapshot();
        snapshot.slope_angle = terrain.hill_angle(self.x);
        snapshot
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

    const fn start_takeoff(&mut self) {
        self.takeoff_requested = true;
    }

    pub(crate) fn prepare_silent_computer_jump(&mut self, terrain: &HillTerrain) {
        self.phase = JumpPhase::Inrun;
        self.frame = 0;
        self.travel = -45.0;
        self.px = self.maxspeed;
        self.x = math::round(self.travel + self.qx);
        self.y = terrain.height_at(self.x);
        self.vertical_pos = f64::from(self.y);
        self.vertical_speed = 0.0;
        self.flight_time = 0.0;
        self.body_angle = 0;
        self.ski_angle = 0;
        self.ski_swing = SkiSwing::None;
        self.landing_style = LandingStyle::None;
        self.height = 0;
        self.delta_height = [0; 6];
        self.first_flight_frame = true;
        self.distance = 0.0;
        self.landing_counter = 0;
        self.fall_type = FallType::None;
        self.result_pending = false;
        self.takeoff_requested = false;
        self.takeoff_counter = 0;
        self.takeoff_phase = 0;
        self.silent_computer = true;
    }

    pub(crate) fn tick(
        &mut self,
        terrain: &HillTerrain,
        wind: FlightWind,
        rng: &mut Random,
        count_onbar_frames: bool,
    ) {
        match self.phase {
            JumpPhase::Info => {
                self.frame += 1;
                self.info_counter += 1;
            }
            JumpPhase::OnBar => {
                // Pascal: if (not treeni) then inc(counter);
                // Training mode: frame stays 0 so the start light is always on.
                if count_onbar_frames {
                    self.frame += 1;
                }
                // Pascal: laskuri > 700 → disqualified
                if count_onbar_frames && self.frame > 700 {
                    self.phase = JumpPhase::Disqualified;
                    self.score = 0;
                    self.distance = 0.0;
                    self.landing_style = LandingStyle::Telemark;
                    self.fall_type = FallType::None;
                }
            }
            JumpPhase::Inrun => {
                self.frame += 1;
                self.tick_inrun(terrain, wind, rng);
            }
            JumpPhase::Flight => {
                self.frame += 1;
                self.tick_flight(terrain, wind, rng);
            }
            JumpPhase::Landing => {
                self.frame += 1;
                self.tick_landing(terrain, rng);
            }
            JumpPhase::Result => {}
            JumpPhase::Disqualified => {}
        }
    }

    fn tick_inrun(&mut self, terrain: &HillTerrain, wind: FlightWind, rng: &mut Random) {
        let fx = self.x;
        let fy = self.y;
        self.travel += self.px * 0.01;
        self.x = math::round(self.travel + self.qx);

        if self.travel >= 0.0 {
            self.phase = JumpPhase::Flight;
            self.tick_flight_after_position_update(fx, fy, terrain, wind, Some(rng));
            return;
        }

        self.y = terrain.height_at(self.x);
        self.vertical_pos = f64::from(self.y);

        if self.takeoff_counter > 0 {
            self.takeoff_counter = self.takeoff_counter.saturating_add(1);
        }
        if self.takeoff_requested && self.travel > -40.0 && self.takeoff_counter == 0 {
            self.takeoff_counter = 1;
        }
        self.takeoff_requested = false;

        if !self.silent_computer && self.frame < 14 {
            self.px = 0.0;
        } else if !self.silent_computer && self.frame < 28 {
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
            self.takeoff_phase = self.takeoff_phase.saturating_add(1);
        }

        self.update_camera(fx, fy);
    }

    fn tick_flight(&mut self, terrain: &HillTerrain, wind: FlightWind, rng: &mut Random) {
        let fx = self.x;
        let fy = self.y;

        self.travel += self.px * 0.01;
        self.x = math::round(self.travel + self.qx);

        self.tick_flight_after_position_update(fx, fy, terrain, wind, Some(rng));
    }

    fn tick_flight_after_position_update(
        &mut self,
        fx: i32,
        fy: i32,
        terrain: &HillTerrain,
        wind: FlightWind,
        mut rng: Option<&mut Random>,
    ) {
        if let Some(style) = self.landing_requested.take() {
            self.landing_style = style;
        }

        if self.lean_back_requested && self.body_angle <= 600 {
            self.body_angle += math::round(f64::from(self.body_angle) / 4.0);
        }
        if self.lean_forward_requested
            && self.landing_style == LandingStyle::None
            && self.body_angle > 0
        {
            self.body_angle -= math::round(f64::from(self.body_angle) / 5.0);
        }
        self.lean_back_requested = false;
        self.lean_forward_requested = false;

        if self.body_angle < 50 {
            self.lift += 0.0001 - f64::from(self.body_angle - 50) / 18_000.0;
        }
        self.lift -= (1.0 - f64::from(self.body_angle) / 900.0) / 1875.0;
        if wind.value > 0 {
            self.lift += f64::from(2 * wind.value).sqrt().sqrt() / 65_500.0;
        } else {
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
                    self.ski_swing = SkiSwing::GustUp;
                    self.lift -= f64::from(rng.random_i32(wind.strength + 50)) / 15_000.0;
                }
                if gust_angle < 0 {
                    self.ski_swing = SkiSwing::GustDown;
                    self.lift += f64::from(rng.random_i32(wind.strength + 50)) / 15_000.0;
                }
            }
        }

        self.lift = self.lift.max(0.105);

        self.vertical_pos += (self.flight_time * self.flight_time)
            .mul_add(self.lift, -((self.vertical_speed - 8.0) / 100.0));
        self.y = math::round(self.vertical_pos);

        self.update_ski_swing(&mut rng);

        if self.height < 6 && self.travel > 20.0 && self.ski_angle == 0 {
            self.ski_swing = SkiSwing::None;
        }

        if self.takeoff_counter > 0 && self.takeoff_phase < 25 {
            self.takeoff_phase = self.takeoff_phase.saturating_add(1);
        }

        if self.landing_style != LandingStyle::None && self.body_angle < 600 {
            self.body_angle += 9 + (self.landing_style.offset() - 1) * 5;
            if self.lift < 1.0 {
                self.lift += 0.003;
            }
        }

        if self.first_flight_frame {
            self.body_angle = 158;
            self.first_flight_frame = false;
            if self.takeoff_counter < 16 {
                self.ski_swing = SkiSwing::LateUp;
            }
            if self.takeoff_counter > 16 {
                self.ski_swing = SkiSwing::LateDown;
            }
            if self.takeoff_counter == 0 {
                self.ski_swing = SkiSwing::None;
            }
        }

        let prev_height = self.height;
        self.height = (terrain.height_at(self.x) - self.y).max(0);
        self.delta_height[(self.frame as usize) % 3] = prev_height - self.height;

        if self.height == 0 {
            self.distance = self.distance();
            if let Some(rng) = rng.as_mut() {
                self.prepare_landing(terrain, rng);
                if self.silent_computer {
                    self.phase = JumpPhase::Result;
                    return;
                }
                self.phase = JumpPhase::Landing;
                self.landing_counter = 0;
                self.tick_landing(terrain, rng);
                return;
            }
            self.phase = JumpPhase::Landing;
            self.landing_counter = 0;
        }

        self.update_camera(fx, fy);
    }

    fn tick_landing(&mut self, terrain: &HillTerrain, rng: &mut Random) {
        if self.result_pending {
            self.phase = JumpPhase::Result;
            self.result_pending = false;
            return;
        }

        let fx = self.x;
        let fy = self.y;

        self.landing_counter += 1;
        let note = rng.random_i32(5) as usize;
        if rng.random_i32(20) == 1 {
            self.style_revealed[note] = true;
        }
        self.travel += self.px * 0.008;
        self.detached_travel += self.detached_px * 0.008;
        if self.skis_stuck {
            self.travel = self.detached_travel;
        }
        self.x = math::round(self.travel + self.qx);
        self.y = terrain.height_at(self.x);
        self.vertical_pos = f64::from(self.y);
        self.detached_vertical_pos =
            f64::from(terrain.height_at(math::round(self.detached_travel + self.qx)));

        if self.fall_type != FallType::None {
            if self.landing_counter > 50 && self.detached_px > 0.0 {
                self.detached_px -= 0.8;
            }
            if self.detached_px < 0.0 {
                self.detached_px = 0.0;
                if self.skis_stuck {
                    self.result_pending = true;
                }
            }
        }

        if self.x > 1050 {
            self.result_pending = true;
        }

        self.update_camera(fx, fy);
    }

    fn prepare_landing(&mut self, terrain: &HillTerrain, rng: &mut Random) {
        self.grade = if self.hill_kr != 0 {
            math::round(self.distance / f64::from(self.hill_kr) * 10.0) * 10
        } else {
            0
        };

        let landing_risk = scoring::landing_risk(
            terrain,
            self.x,
            self.distance,
            self.hill_kr,
            self.body_angle,
            self.landing_style,
        );
        self.fall_type = landing_risk.fall_type;
        self.style_base -= landing_risk.style_penalty;
        if rng.random_i32(1000) < landing_risk.risk {
            self.fall_type = FallType::Crash;
        }

        let score = scoring::calculate_score(
            self.style_base,
            self.hill_kr,
            self.distance,
            self.fall_type,
            self.landing_style,
            rng,
        );
        self.style_base = score.style_base;
        self.style_points = score.style_points;
        self.score = score.score;
        self.style_revealed = [false; 5];
        self.result_pending = false;

        if self.fall_type != FallType::None {
            self.grade = self.fall_type.as_grade();
        }
        self.skis_stuck = rng.random_i32(2) != 0;
        self.start_anim = if self.landing_style == LandingStyle::TwoFooted {
            50
        } else {
            100
        };
        self.detached_travel = self.travel;
        self.detached_vertical_pos = self.vertical_pos;
        self.detached_px = self.px;
    }

    fn update_ski_swing(&mut self, rng: &mut Option<&mut Random>) {
        if self.ski_swing == SkiSwing::None {
            return;
        }

        match self.ski_swing {
            SkiSwing::LateUp => {
                if self.ski_angle == 0 {
                    self.ski_angle = -51 - (16 - i32::from(self.takeoff_counter)) * 6;
                    if self.ski_angle < -105 {
                        self.ski_angle = -105;
                    }
                } else {
                    self.ski_angle -= 4;
                }
                if self.ski_angle < (i32::from(self.takeoff_counter) - 16) * 14 {
                    self.ski_swing = SkiSwing::ReturnUp;
                }
            }
            SkiSwing::ReturnUp => {
                if self.ski_angle < 0 {
                    self.ski_angle += 2;
                }
                if self.ski_angle > 0 {
                    self.ski_angle = 0;
                }
            }
            SkiSwing::GustUp => {
                self.ski_angle -= rng.as_deref_mut().map_or(0, |rng| rng.random_i32(50)) + 30;
                self.ski_swing = SkiSwing::ReturnUp;
            }
            SkiSwing::LateDown => {
                if self.ski_angle == 0 {
                    self.ski_angle = 70 + (i32::from(self.takeoff_counter) - 16) * 6;
                    if self.ski_angle > 130 {
                        self.ski_angle = 130;
                    }
                } else {
                    self.ski_angle += 3;
                }
                if self.ski_angle > (i32::from(self.takeoff_counter) - 16) * 14 {
                    self.ski_swing = SkiSwing::ReturnDown;
                }
            }
            SkiSwing::ReturnDown => {
                if self.ski_angle > 0 {
                    self.ski_angle -= 1;
                }
                if self.ski_angle < 0 {
                    self.ski_angle = 0;
                }
            }
            SkiSwing::GustDown => {
                self.ski_angle += rng.as_deref_mut().map_or(0, |rng| rng.random_i32(50)) + 30;
                self.ski_swing = SkiSwing::ReturnDown;
            }
            SkiSwing::None => {}
        }

        if self.ski_angle == 0 {
            self.ski_swing = SkiSwing::None;
        }
    }

    fn update_camera(&mut self, fx: i32, fy: i32) {
        if self.x >= 160 && self.x < 864 {
            self.sx += self.x - fx;
        }
        if self.y >= 100 && self.y < 412 {
            self.sy += self.y - fy;
        }
        self.sx = self.sx.clamp(0, 704);
        self.sy = self.sy.clamp(0, 312);
    }

    fn distance(&self) -> f64 {
        let vertical_delta = self.vertical_pos - f64::from(self.ramp_y);
        f64::from(math::round(
            self.travel.hypot(vertical_delta) * self.distance_factor * 0.5,
        )) * 0.5
    }

    const fn lean_forward(&mut self) {
        self.lean_forward_requested = true;
    }

    const fn lean_back(&mut self) {
        self.lean_back_requested = true;
    }

    const fn set_landing(&mut self, style: LandingStyle) {
        self.landing_requested = Some(style);
    }

    fn landing_body_anim_for_state(&self, terrain: &HillTerrain) -> u16 {
        let detached_x = math::round(self.detached_travel + self.qx);
        let detached_ski = slope_ski_anim(terrain.hill_angle(detached_x));
        if self.fall_type != FallType::None {
            fall_body_anim(
                self.fall_type,
                self.landing_counter,
                self.body_angle,
                detached_ski,
                self.landing_style,
            )
        } else if self.phase == JumpPhase::Landing || self.phase == JumpPhase::Result {
            post_landing_body_anim(
                self.landing_counter,
                self.start_anim,
                self.landing_style,
                self.grade,
                detached_ski,
            )
        } else {
            landing_body_anim(detached_ski, self.landing_style)
        }
    }

    pub(crate) fn body_position(&self) -> (i32, i32) {
        if self.phase == JumpPhase::Landing || self.phase == JumpPhase::Result {
            (
                math::round(self.detached_travel + self.qx),
                math::round(self.detached_vertical_pos),
            )
        } else {
            (self.x, self.y)
        }
    }

    pub(crate) fn anims(&self, terrain: &HillTerrain) -> (u16, u16) {
        match self.phase {
            JumpPhase::Info => (
                Sprite::IdleBody as u16,
                slope_ski_anim(terrain.hill_angle(self.x)),
            ),
            JumpPhase::OnBar => (
                Sprite::IdleBody as u16,
                slope_ski_anim(terrain.hill_angle(self.x)),
            ),
            JumpPhase::Inrun => {
                let ski = slope_ski_anim(terrain.hill_angle(self.x));
                if self.takeoff_counter > 0 {
                    (takeoff_body_anim(self.takeoff_phase), ski)
                } else if self.frame < 28 {
                    (inrun_transition_body_anim(self.frame), ski)
                } else {
                    (inrun_body_anim(ski), ski)
                }
            }
            JumpPhase::Flight => {
                let body = if self.takeoff_counter > 0 && self.takeoff_phase < 25 {
                    takeoff_body_anim(self.takeoff_phase)
                } else {
                    flight_body_anim(self.body_angle)
                };
                let ski = if self.height < 6 && self.travel > 20.0 {
                    slope_ski_anim(terrain.hill_angle(self.x) / (self.height + 1))
                } else {
                    flight_ski_anim(self.ski_angle)
                };
                (body, ski)
            }
            JumpPhase::Disqualified => (
                Sprite::IdleBody as u16,
                slope_ski_anim(terrain.hill_angle(self.x)),
            ),
            JumpPhase::Landing | JumpPhase::Result => {
                let ski = slope_ski_anim(terrain.hill_angle(self.x));
                (self.landing_body_anim_for_state(terrain), ski)
            }
        }
    }
}

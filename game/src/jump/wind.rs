use crate::jump::math;
use crate::jump::types::{FlightWind, JumpPhase};
use crate::rng::Random;

pub(crate) const WIND_POSITION_COUNT: u8 = 11;
const FIRST_JUMPER_RELATIVE_POSITION: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct Wind {
    pub strength: i32,
    pub windy: i32,
    pub value: i32,
    lower: i32,
    upper: i32,
    angle: f32,
    increasing: bool,
    place: u8,
    position: WindPosition,
    enabled: bool,
}

impl Default for Wind {
    fn default() -> Self {
        let mut wind = Self {
            strength: 0,
            windy: 0,
            value: 0,
            lower: 0,
            upper: 0,
            angle: 0.0,
            increasing: false,
            place: 0,
            position: WindPosition { x: 10, y: 180 },
            enabled: true,
        };
        wind.set_place(0);
        wind
    }
}

impl Wind {
    pub(crate) fn sample_flight_wind(
        &mut self,
        phase: Option<JumpPhase>,
        rng: &mut Random,
    ) -> FlightWind {
        let value = if matches!(
            phase,
            Some(JumpPhase::Info | JumpPhase::Result | JumpPhase::Disqualified)
        ) {
            self.value
        } else {
            self.sample(rng)
        };
        FlightWind {
            value,
            windy: self.windy,
            strength: self.strength,
        }
    }

    pub fn initialize(&mut self, rng: &mut Random, place: u8) {
        let temp1 = rng.random_i32(180);
        let temp2 = rng.random_i32(120);
        self.windy = temp2;
        self.lower = temp1 - temp2;
        self.upper = temp1 + temp2;
        self.angle = (rng.random_i32(temp2 * 2) + self.lower) as f32;
        self.strength = rng.random_i32(50);
        self.increasing = rng.random_i32(2) == 0;
        self.set_place(place);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.strength = 0;
            self.value = 0;
        }
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn sample(&mut self, rng: &mut Random) -> i32 {
        self.shift(rng);
        if !self.enabled {
            self.value = 0;
            return 0;
        }
        self.value = math::round(f64::from(
            (self.angle.to_radians()).cos() * self.strength as f32,
        ));
        self.value
    }

    pub fn advance(&mut self, rng: &mut Random) {
        self.shift(rng);
    }

    pub const fn set_place(&mut self, place: u8) {
        self.place = if place < WIND_POSITION_COUNT {
            place
        } else {
            WIND_POSITION_COUNT - 1
        };
        self.position = match self.place {
            0 => WindPosition { x: 10, y: 180 },
            1 => WindPosition { x: 10, y: 97 },
            2 => WindPosition { x: 268, y: 180 },
            3 => WindPosition { x: 150, y: 180 },
            4 => WindPosition { x: 268, y: 97 },
            5 => WindPosition { x: 268, y: 21 },
            6 => WindPosition { x: 150, y: 21 },
            7 => WindPosition { x: 56, y: 33 },
            _ => WindPosition { x: 10, y: 180 },
        };
    }

    pub const fn position(&self) -> WindPosition {
        self.position
    }

    #[cfg(test)]
    pub const fn place(&self) -> u8 {
        self.place
    }

    pub const fn is_jumper_relative(&self) -> bool {
        self.place >= FIRST_JUMPER_RELATIVE_POSITION
    }

    pub fn position_for_jumper(&self, jumper_screen_x: i32, jumper_screen_y: i32) -> WindPosition {
        let mut x = jumper_screen_x;
        let mut y = jumper_screen_y;
        match self.place {
            8 => {
                x += 10;
                y -= 20;
            }
            9 => {
                x += 15;
                y -= 5;
            }
            10 => {
                x -= 10;
                y += 12;
            }
            _ => return self.position,
        }
        WindPosition { x, y }
    }

    fn shift(&mut self, rng: &mut Random) {
        if self.increasing && self.angle > self.upper as f32 {
            self.increasing = false;
        }
        if !self.increasing && self.angle < self.lower as f32 {
            self.increasing = true;
        }
        if rng.random_i32(50) == 0 {
            self.increasing = !self.increasing;
        }

        let delta = rng.random_i32(4) as f32 / 5.0;
        if self.increasing {
            self.angle += delta;
        } else {
            self.angle -= delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_pascal_wind_for_seed_zero() {
        let mut rng = Random::new(0);
        let mut wind = Wind::default();
        wind.initialize(&mut rng, 0);

        assert_eq!(wind.windy, 71);
        assert_eq!(wind.lower, 27);
        assert_eq!(wind.upper, 169);
        assert!((wind.angle - 128.0).abs() < 0.000_01);
        assert_eq!(wind.strength, 42);
        assert!(!wind.increasing);

        let actual: Vec<i32> = (0..12).map(|_| wind.sample(&mut rng)).collect();
        assert_eq!(
            actual,
            vec![-26, -26, -25, -25, -25, -24, -24, -24, -24, -23, -23, -23]
        );
        assert!((wind.angle - 123.400_01).abs() < 0.000_1);
    }

    #[test]
    fn matches_pascal_wind_for_seed_5489() {
        let mut rng = Random::new(5489);
        let mut wind = Wind::default();
        wind.initialize(&mut rng, 0);

        assert_eq!(wind.windy, 16);
        assert_eq!(wind.lower, 130);
        assert_eq!(wind.upper, 162);
        assert!((wind.angle - 158.0).abs() < 0.000_01);
        assert_eq!(wind.strength, 41);
        assert!(wind.increasing);

        let actual: Vec<i32> = (0..12).map(|_| wind.sample(&mut rng)).collect();
        assert_eq!(
            actual,
            vec![-38, -38, -38, -38, -38, -39, -39, -39, -39, -39, -39, -39]
        );
        assert!((wind.angle - 161.200_01).abs() < 0.000_1);
    }

    #[test]
    fn all_fixed_positions_match_pascal() {
        let expected = [
            WindPosition { x: 10, y: 180 },
            WindPosition { x: 10, y: 97 },
            WindPosition { x: 268, y: 180 },
            WindPosition { x: 150, y: 180 },
            WindPosition { x: 268, y: 97 },
            WindPosition { x: 268, y: 21 },
            WindPosition { x: 150, y: 21 },
            WindPosition { x: 56, y: 33 },
        ];
        let mut wind = Wind::default();

        for (place, expected_position) in expected.into_iter().enumerate() {
            wind.set_place(place as u8);
            assert_eq!(wind.place(), place as u8);
            assert!(!wind.is_jumper_relative());
            assert_eq!(wind.position(), expected_position);
            assert_eq!(wind.position_for_jumper(100, 80), expected_position);
        }
    }

    #[test]
    fn all_jumper_relative_positions_match_pascal() {
        let expected = [
            (8, WindPosition { x: 110, y: 60 }),
            (9, WindPosition { x: 115, y: 75 }),
            (10, WindPosition { x: 90, y: 92 }),
        ];
        let mut wind = Wind::default();

        for (place, expected_position) in expected {
            wind.set_place(place);
            assert_eq!(wind.place(), place);
            assert!(wind.is_jumper_relative());
            assert_eq!(wind.position_for_jumper(100, 80), expected_position);
        }
    }
}

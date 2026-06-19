use crate::jump::math;
use crate::rng::Random;

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
            self.value = 0;
        }
    }

    pub fn sample(&mut self, rng: &mut Random) -> i32 {
        if !self.enabled {
            self.value = 0;
            return 0;
        }
        self.shift(rng);
        self.value = math::round(f64::from(
            (self.angle.to_radians()).cos() * self.strength as f32,
        ));
        self.value
    }

    pub fn advance_without_sampling(&mut self, rng: &mut Random) {
        if self.enabled {
            self.shift(rng);
        }
    }

    pub const fn set_place(&mut self, place: u8) {
        self.place = place;
        self.position = match place {
            2 => WindPosition { x: 10, y: 97 },
            3 => WindPosition { x: 268, y: 180 },
            4 => WindPosition { x: 150, y: 180 },
            5 => WindPosition { x: 268, y: 97 },
            6 => WindPosition { x: 268, y: 21 },
            7 => WindPosition { x: 150, y: 21 },
            8 => WindPosition { x: 56, y: 33 },
            _ => WindPosition { x: 10, y: 180 },
        };
    }

    #[must_use]
    pub const fn position(&self) -> WindPosition {
        self.position
    }

    #[must_use]
    pub const fn place(&self) -> u8 {
        self.place
    }

    /// For `place` 11-13 the wind meter follows the jumper instead of
    /// sitting at a fixed screen position. Offsets match TUULI.PAS `Tuo`.
    #[must_use]
    pub fn position_for_jumper(&self, jumper_screen_x: i32, jumper_screen_y: i32) -> WindPosition {
        let mut x = jumper_screen_x;
        let mut y = jumper_screen_y;
        match self.place {
            11 => { x += 10; y -= 20; }
            12 => { x += 15; y -= 5; }
            13 => { x -= 10; y += 12; }
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
}

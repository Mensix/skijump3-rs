use crate::pascal_random::PascalRandom;
use engine::consts::{HEIGHT, WIDTH};

const LUMI_MAX: usize = 256;
const SINE_LENGTH: usize = 512;
const BG_MIN: u8 = 64;
const BG_MAX: u8 = 215;

#[derive(Debug, Clone)]
struct Snowflake {
    x: i64,
    y: i64,
    gravity: i64,
    sin_pos: usize,
    c1: u16,
    c2: u16,
    style: u16,
}

#[derive(Debug, Clone)]
pub struct SnowSystem {
    flakes: Vec<Snowflake>,
    sine: Vec<i64>,
    count: u16,
    max: usize,
    perus_g: u16,
    g_variation: u16,
    side_movement: u16,
    sleet: bool,
}

/// Pascal `LMaara` calculation: random snow count used at event start.
/// Called once per event when `eka=true`.
pub fn calculate_lmaara(rng: &mut PascalRandom) -> u16 {
    let mut lmaara = rng.random_i32(2) * rng.random_i32(256);
    if lmaara > 0 && lmaara < 40 {
        lmaara += rng.random_i32(150);
    }
    if lmaara > 0 && rng.random_i32(4) == 0 {
        lmaara += 1000;
    }
    lmaara as u16
}

impl SnowSystem {
    #[must_use] 
    pub fn new() -> Self {
        let mut system = Self {
            flakes: vec![],
            sine: vec![0; SINE_LENGTH + 1],
            count: 0,
            max: 0,
            perus_g: 600,
            g_variation: 300,
            side_movement: 50,
            sleet: false,
        };
        system.compute_sine();
        system
    }

    fn compute_sine(&mut self) {
        for i in 0..=SINE_LENGTH {
            let angle = i as f64 * std::f64::consts::PI * 2.0 / SINE_LENGTH as f64;
            self.sine[i] = (angle.sin() * f64::from(self.side_movement)).round() as i64;
        }
    }

    pub fn set_count(&mut self, count: u16, rng: &mut PascalRandom) {
        self.perus_g = 600;
        self.g_variation = 300;
        self.side_movement = 50;
        self.sleet = false;
        self.count = count;
        self.max = count as usize;
        if count > 1000 {
            self.sleet = true;
            self.perus_g = 875;
            self.g_variation = 100;
            self.side_movement = 50;
            self.max = (count - 1000) as usize;
        }
        self.reset(rng);
    }

    #[must_use] 
    pub fn count(&self) -> u16 {
        self.count
    }

    fn reset(&mut self, rng: &mut PascalRandom) {
        self.compute_sine();
        // Pascal: always initializes all LumiMax (256) flakes regardless of Max
        self.flakes = (0..LUMI_MAX)
            .map(|_| {
                let x = i64::from(rng.random_i32(WIDTH as i32)) << 10;
                let y = i64::from(rng.random_i32(HEIGHT as i32)) << 10;
                let sin_pos = rng.random_i32(SINE_LENGTH as i32) as usize;
                let gravity = i64::from(rng.random_i32(i32::from(self.g_variation))) + i64::from(self.perus_g)
                    - i64::from(self.g_variation);
                let style = rng.random_i32(2) as u16;
                let style = if self.sleet && style == 1 {
                    rng.random_i32(2) as u16
                } else {
                    style
                };
                let c1 = Self::get_color(rng);
                let c2 = Self::get_color(rng);
                Snowflake {
                    x,
                    y,
                    gravity,
                    sin_pos,
                    c1,
                    c2,
                    style,
                }
            })
            .collect();
    }

    fn get_color(rng: &mut PascalRandom) -> u16 {
        let low = rng.random_i32(4) + 232;
        let high = rng.random_i32(4) + 232;
        (low as u16) | ((high as u16) << 8)
    }

    pub fn update(&mut self, buffer: &mut [u8], delta_x: i32, delta_y: i32, wind: i32, draw: bool) {
        let max = self.max.min(LUMI_MAX - 1);
        for flake in self.flakes.iter_mut().take(max + 1) {
            if draw {
                flake.x += self.sine[flake.sin_pos] + i64::from(delta_x) * 512 + i64::from(wind);
                flake.sin_pos = (flake.sin_pos + 1) & (SINE_LENGTH - 1);
                flake.y += flake.gravity + i64::from(delta_y) * 256;
            }

            let x = ((flake.x as i32 as u32) >> 10) as u16;
            let y = ((flake.y as i32 as u32) >> 10) as u16;
            let offset = x.wrapping_add(y.wrapping_mul(WIDTH as u16)) as usize;
            if offset < 63_679
                && offset + (WIDTH as usize) + 1 < buffer.len()
                && buffer[offset] >= BG_MIN
                && buffer[offset + 1] >= BG_MIN
                && buffer[offset] < BG_MAX
                && buffer[offset + 1] < BG_MAX
            {
                if flake.style == 1 && offset + 1 < buffer.len() {
                    buffer[offset] = flake.c1 as u8;
                    buffer[offset + 1] = (flake.c1 >> 8) as u8;
                    buffer[offset + (WIDTH as usize)] = flake.c2 as u8;
                    buffer[offset + (WIDTH as usize) + 1] = (flake.c2 >> 8) as u8;
                } else {
                    buffer[offset] = flake.c1 as u8;
                }
            }
        }
    }
}

impl Default for SnowSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flake_at(x: i64, y: i64, color: u16) -> Snowflake {
        Snowflake {
            x,
            y,
            gravity: 0,
            sin_pos: 0,
            c1: color,
            c2: color,
            style: 0,
        }
    }

    #[test]
    fn update_uses_pascal_wrapped_offset_for_offscreen_flakes() {
        let mut snow = SnowSystem::new();
        snow.flakes = vec![flake_at(0, 205_i64 << 10, 233)];
        snow.max = 0;

        let mut buffer = vec![BG_MIN; (WIDTH * HEIGHT) as usize];
        snow.update(&mut buffer, 0, 0, 0, false);

        assert_eq!(buffer[64], 233);
    }

    #[test]
    fn update_respects_pascal_inclusive_max_count() {
        let mut snow = SnowSystem::new();
        snow.flakes = vec![
            flake_at(10_i64 << 10, 10_i64 << 10, 233),
            flake_at(20_i64 << 10, 10_i64 << 10, 234),
        ];
        snow.max = 0;

        let mut buffer = vec![BG_MIN; (WIDTH * HEIGHT) as usize];
        snow.update(&mut buffer, 0, 0, 0, false);

        assert_eq!(buffer[10 + 10 * WIDTH as usize], 233);
        assert_eq!(buffer[20 + 10 * WIDTH as usize], BG_MIN);
    }
}

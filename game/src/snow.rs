use crate::pascal_random::PascalRandom;

const LUMI_MAX: usize = 256;
const SINE_LENGTH: usize = 512;
const BG_MIN: u8 = 64;
const BG_MAX: u8 = 215;
const SCREEN_W: u32 = 320;
const SCREEN_H: u32 = 200;

struct Snowflake {
    x: i64,
    y: i64,
    gravity: i64,
    sin_pos: usize,
    c1: u16,
    c2: u16,
    style: u16,
}

pub struct SnowSystem {
    flakes: Vec<Snowflake>,
    sine: Vec<i64>,
    max: usize,
    perus_g: u16,
    g_variation: u16,
    side_movement: u16,
    sleet: bool,
}

impl Default for SnowSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SnowSystem {
    pub fn new() -> Self {
        let mut system = Self {
            flakes: vec![],
            sine: vec![0; SINE_LENGTH + 1],
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
            self.sine[i] = (angle.sin() * self.side_movement as f64).round() as i64;
        }
    }

    pub fn set_count(&mut self, count: u16, rng: &mut PascalRandom) {
        self.perus_g = 600;
        self.g_variation = 300;
        self.side_movement = 50;
        self.sleet = false;
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

    fn reset(&mut self, rng: &mut PascalRandom) {
        self.compute_sine();
        let count = self.max.min(LUMI_MAX);
        self.flakes = (0..count)
            .map(|_| {
                let style = rng.random_i32(2) as u16;
                let style = if self.sleet && style == 1 {
                    rng.random_i32(2) as u16
                } else {
                    style
                };
                Snowflake {
                    x: (rng.random_i32(SCREEN_W as i32) as i64) << 10,
                    y: (rng.random_i32(SCREEN_H as i32) as i64) << 10,
                    sin_pos: rng.random_i32(SINE_LENGTH as i32) as usize,
                    gravity: (rng.random_i32(self.g_variation as i32) as i64 + self.perus_g as i64
                        - self.g_variation as i64),
                    style,
                    c1: Self::get_color(rng),
                    c2: Self::get_color(rng),
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
        for flake in &mut self.flakes {
            if draw {
                flake.x += self.sine[flake.sin_pos] + (delta_x as i64) * 512 + wind as i64;
                flake.sin_pos = (flake.sin_pos + 1) & (SINE_LENGTH - 1);
                flake.y += flake.gravity + (delta_y as i64) * 256;
            }
            if flake.x < 0 || flake.y < 0 {
                continue;
            }
            let screen_x = (flake.x >> 10) as usize;
            let screen_y = (flake.y >> 10) as usize;
            if screen_y >= SCREEN_H as usize || screen_x >= SCREEN_W as usize {
                continue;
            }
            let offset = screen_x + screen_y * SCREEN_W as usize;
            if buffer[offset] >= BG_MIN && buffer[offset] < BG_MAX {
                if flake.style == 1 && offset + 1 < buffer.len() {
                    buffer[offset] = flake.c1 as u8;
                    if offset + 1 < buffer.len() {
                        buffer[offset + 1] = (flake.c1 >> 8) as u8;
                    }
                    if offset + (SCREEN_W as usize) < buffer.len() {
                        buffer[offset + (SCREEN_W as usize)] = flake.c2 as u8;
                    }
                    if offset + (SCREEN_W as usize) + 1 < buffer.len() {
                        buffer[offset + (SCREEN_W as usize) + 1] = (flake.c2 >> 8) as u8;
                    }
                } else {
                    buffer[offset] = flake.c1 as u8;
                }
            }
        }
    }
}

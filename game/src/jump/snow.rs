use crate::rng::Random;
use engine::color::Rgba;
use engine::consts::{HEIGHT, WIDTH};
use engine::oxide::PointBatches;

const SNOW_MAX: usize = 256;
const SINE_LENGTH: usize = 512;
const SNOW_COLOR_BASE: u8 = 232;
const SNOW_COLORS: [Rgba; 4] = [
    Rgba::from_rgb6(40, 40, 41),
    Rgba::from_rgb6(48, 48, 49),
    Rgba::from_rgb6(55, 55, 56),
    Rgba::from_rgb6(63, 63, 63),
];
// Original `TummaLumi` darkens palette entries 232..235 for sleet.
const SLEET_COLORS: [Rgba; 4] = [
    Rgba::from_rgb6(37, 37, 37),
    Rgba::from_rgb6(42, 42, 42),
    Rgba::from_rgb6(46, 46, 46),
    Rgba::from_rgb6(51, 51, 51),
];

#[derive(Debug, Clone, Copy, Default)]
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
    flakes: [Snowflake; SNOW_MAX],
    sine: [i64; SINE_LENGTH + 1],
    count: u16,
    max: usize,
    sleet: bool,
}

pub fn calculate_snow_count(rng: &mut Random) -> u16 {
    let count = rng.random_i32(2) * rng.random_i32(256);
    let count = if count > 0 && count < 40 {
        count + rng.random_i32(150)
    } else {
        count
    };
    let count = if count > 0 && rng.random_i32(4) == 0 {
        count + 1000
    } else {
        count
    };
    count as u16
}

impl SnowSystem {
    pub fn new() -> Self {
        let mut system = Self {
            flakes: [Snowflake::default(); SNOW_MAX],
            sine: [0; SINE_LENGTH + 1],
            count: 0,
            max: 0,
            sleet: false,
        };
        system.compute_sine(50);
        system
    }

    fn compute_sine(&mut self, side_movement: u16) {
        for i in 0..=SINE_LENGTH {
            let angle = i as f64 * std::f64::consts::PI * 2.0 / SINE_LENGTH as f64;
            self.sine[i] = (angle.sin() * f64::from(side_movement)).round() as i64;
        }
    }

    pub fn set_count(&mut self, count: u16, rng: &mut Random) {
        let mut base_gravity = 600;
        let mut gravity_variation = 300;
        let side_movement = 50;
        let mut sleet = false;
        self.count = count;
        self.max = count as usize;
        if count > 1000 {
            sleet = true;
            base_gravity = 875;
            gravity_variation = 100;
            self.max = (count - 1000) as usize;
        }
        self.sleet = sleet;
        self.reset(rng, base_gravity, gravity_variation, side_movement, sleet);
    }

    pub fn clear_count(&mut self) {
        self.count = 0;
        self.max = 0;
    }

    pub const fn count(&self) -> u16 {
        self.count
    }

    fn reset(
        &mut self,
        rng: &mut Random,
        base_gravity: u16,
        gravity_variation: u16,
        side_movement: u16,
        sleet: bool,
    ) {
        self.compute_sine(side_movement);
        for flake in &mut self.flakes {
            let style = rng.random_i32(2) as u16;
            let style = if sleet && style == 1 {
                rng.random_i32(2) as u16
            } else {
                style
            };
            *flake = Snowflake {
                x: i64::from(rng.random_i32(WIDTH as i32)) << 10,
                y: i64::from(rng.random_i32(HEIGHT as i32)) << 10,
                gravity: i64::from(rng.random_i32(i32::from(gravity_variation)))
                    + i64::from(base_gravity)
                    - i64::from(gravity_variation),
                sin_pos: rng.random_i32(SINE_LENGTH as i32) as usize,
                c1: Self::snow_color_pair(rng),
                c2: Self::snow_color_pair(rng),
                style,
            };
        }
    }

    fn snow_color_pair(rng: &mut Random) -> u16 {
        let low = rng.random_i32(4) + 232;
        let high = rng.random_i32(4) + 232;
        (low as u16) | ((high as u16) << 8)
    }

    pub fn advance(&mut self, delta_x: i32, delta_y: i32, wind: i32) {
        if self.count == 0 {
            return;
        }
        let max = self.max.min(SNOW_MAX - 1);
        let width = i64::from(WIDTH) << 10;
        let height = i64::from(HEIGHT) << 10;
        for flake in &mut self.flakes[..=max] {
            flake.x =
                (flake.x + self.sine[flake.sin_pos] + i64::from(delta_x) * 512 + i64::from(wind))
                    .rem_euclid(width);
            flake.sin_pos = (flake.sin_pos + 1) & (SINE_LENGTH - 1);
            flake.y = (flake.y + flake.gravity + i64::from(delta_y) * 256).rem_euclid(height);
        }
    }

    pub(crate) fn pixel_draws(&self) -> PointBatches {
        if self.count == 0 {
            return PointBatches::empty();
        }
        let palette = if self.sleet {
            SLEET_COLORS
        } else {
            SNOW_COLORS
        };
        let mut counts = [0usize; 4];
        self.for_each_point(|_, _, index| {
            let offset = index.saturating_sub(SNOW_COLOR_BASE) as usize;
            if offset < counts.len() {
                counts[offset] += 1;
            }
        });

        let mut ranges = std::array::from_fn(|_| 0..0);
        let mut start = 0;
        for (range, count) in ranges.iter_mut().zip(counts) {
            *range = start..start + count;
            start += count;
        }
        let mut cursors = ranges.each_ref().map(|range| range.start);
        let mut points = vec![(0, 0); start];
        self.for_each_point(|x, y, index| {
            let offset = index.saturating_sub(SNOW_COLOR_BASE) as usize;
            if offset < cursors.len() {
                points[cursors[offset]] = (x, y);
                cursors[offset] += 1;
            }
        });

        PointBatches {
            colors: palette,
            points,
            ranges,
        }
    }

    fn for_each_point(&self, mut add: impl FnMut(i32, i32, u8)) {
        let max = self.max.min(SNOW_MAX - 1);
        for flake in &self.flakes[..=max] {
            let x = ((flake.x as i32 as u32) >> 10) as i32;
            let y = ((flake.y as i32 as u32) >> 10) as i32;
            let mut add_point = |x: i32, y: i32, index: u8| {
                if (0..WIDTH as i32).contains(&x) && (0..HEIGHT as i32).contains(&y) {
                    add(x, y, index);
                }
            };
            if flake.style == 1 {
                add_point(x, y, flake.c1 as u8);
                add_point(x + 1, y, (flake.c1 >> 8) as u8);
                add_point(x, y + 1, flake.c2 as u8);
                add_point(x + 1, y + 1, (flake.c2 >> 8) as u8);
            } else {
                add_point(x, y, flake.c1 as u8);
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
    fn pixel_draws_respect_inclusive_max_count() {
        let mut snow = SnowSystem::new();
        snow.flakes[0] = flake_at(10_i64 << 10, 10_i64 << 10, 233);
        snow.flakes[1] = flake_at(20_i64 << 10, 10_i64 << 10, 234);
        snow.max = 0;
        snow.count = 1;

        let pixels = snow.pixel_draws();
        assert_eq!(pixels.points, [(10, 10)]);
    }

    #[test]
    fn snowflakes_reenter_at_the_top_after_falling_offscreen() {
        let mut snow = SnowSystem::new();
        snow.flakes[0] = flake_at(10_i64 << 10, (HEIGHT as i64 - 1) << 10, 233);
        snow.flakes[0].gravity = 2 << 10;
        snow.max = 0;
        snow.count = 1;

        snow.advance(0, 0, 0);

        assert_eq!(snow.pixel_draws().points, [(10, 1)]);
    }

    #[test]
    fn drawing_does_not_advance_snow() {
        let mut snow = SnowSystem::new();
        snow.flakes[0] = flake_at(10_i64 << 10, 10_i64 << 10, 233);
        snow.max = 0;
        snow.count = 1;
        let before = snow.flakes[0];
        let before_pixels = snow.pixel_draws();
        let after_pixels = snow.pixel_draws();

        assert_eq!(snow.flakes[0].x, before.x);
        assert_eq!(snow.flakes[0].y, before.y);
        assert_eq!(snow.flakes[0].sin_pos, before.sin_pos);
        assert_eq!(before_pixels, after_pixels);
    }

    #[test]
    fn one_advance_moves_snow_once_regardless_of_draw_count() {
        let mut snow = SnowSystem::new();
        snow.flakes[0] = flake_at(10_i64 << 10, 10_i64 << 10, 233);
        snow.flakes[0].gravity = 600;
        snow.max = 0;
        snow.count = 1;
        snow.advance(0, 0, 0);
        let advanced = snow.flakes[0];
        let _ = snow.pixel_draws();
        let _ = snow.pixel_draws();

        assert_eq!(advanced.y, (10_i64 << 10) + 600);
        assert_eq!(snow.flakes[0].y, advanced.y);
        assert_eq!(snow.flakes[0].sin_pos, 1);
    }
}

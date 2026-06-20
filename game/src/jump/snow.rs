use crate::rng::Random;
use engine::color::Rgba;
use engine::consts::{HEIGHT, WIDTH};
use std::rc::Rc;

const SNOW_MAX: usize = 256;
const SINE_LENGTH: usize = 512;
const BG_MIN: u8 = 64;
const BG_MAX: u8 = 215;
const SNOW_COLOR_BASE: u8 = 232;
const SNOW_COLORS: [Rgba; 4] = [
    Rgba::from_rgb6(40, 40, 41),
    Rgba::from_rgb6(48, 48, 49),
    Rgba::from_rgb6(55, 55, 56),
    Rgba::from_rgb6(63, 63, 63),
];

fn snow_index_to_rgba(idx: u8) -> [u8; 4] {
    let offset = idx.saturating_sub(SNOW_COLOR_BASE) as usize;
    let rgba = SNOW_COLORS
        .get(offset)
        .copied()
        .unwrap_or(Rgba::rgb(255, 255, 255));
    [rgba.r, rgba.g, rgba.b, rgba.a]
}

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
}

/// Pascal `LMaara` calculation: random snow count used at event start.
/// Called once per event when `first_event=true`.
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
    #[must_use]
    pub fn new() -> Self {
        let mut system = Self {
            flakes: [Snowflake::default(); SNOW_MAX],
            sine: [0; SINE_LENGTH + 1],
            count: 0,
            max: 0,
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
        self.reset(rng, base_gravity, gravity_variation, side_movement, sleet);
    }

    pub fn clear_count(&mut self) {
        self.count = 0;
        self.max = 0;
    }

    #[must_use]
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
                c1: Self::get_color(rng),
                c2: Self::get_color(rng),
                style,
            };
        }
    }

    fn get_color(rng: &mut Random) -> u16 {
        let low = rng.random_i32(4) + 232;
        let high = rng.random_i32(4) + 232;
        (low as u16) | ((high as u16) << 8)
    }

    /// Draw snowflakes onto a viewport.
    ///
    /// * `rgba_buffer` — the RGBA viewport pixels (`WIDTH * HEIGHT * 4` bytes)
    /// * `mask` — indexed-pixel mask for position checking (`WIDTH * HEIGHT` bytes)
    pub fn update(
        &mut self,
        rgba_buffer: &mut [u8],
        mask: &[u8],
        delta_x: i32,
        delta_y: i32,
        wind: i32,
        draw: bool,
    ) {
        let max = self.max.min(SNOW_MAX - 1);
        let pixel_count = (WIDTH as usize) * (HEIGHT as usize);
        for flake in &mut self.flakes[..=max] {
            if draw {
                flake.x += self.sine[flake.sin_pos] + i64::from(delta_x) * 512 + i64::from(wind);
                flake.sin_pos = (flake.sin_pos + 1) & (SINE_LENGTH - 1);
                flake.y += flake.gravity + i64::from(delta_y) * 256;
            }

            let x = ((flake.x as i32 as u32) >> 10) as u16;
            let y = ((flake.y as i32 as u32) >> 10) as u16;
            let offset = x.wrapping_add(y.wrapping_mul(WIDTH as u16)) as usize;
            if offset < 63_679
                && offset + (WIDTH as usize) + 1 < pixel_count
                && mask[offset] >= BG_MIN
                && mask[offset + 1] >= BG_MIN
                && mask[offset] < BG_MAX
                && mask[offset + 1] < BG_MAX
            {
                let rgba_off = offset * 4;
                if flake.style == 1 {
                    let c1_low = flake.c1 as u8;
                    let c1_high = (flake.c1 >> 8) as u8;
                    let c2_low = flake.c2 as u8;
                    let c2_high = (flake.c2 >> 8) as u8;
                    let rgba_next_row = rgba_off + (WIDTH as usize) * 4;
                    if rgba_next_row + 7 < rgba_buffer.len() {
                        rgba_buffer[rgba_off..rgba_off + 4]
                            .copy_from_slice(&snow_index_to_rgba(c1_low));
                        rgba_buffer[rgba_off + 4..rgba_off + 8]
                            .copy_from_slice(&snow_index_to_rgba(c1_high));
                        rgba_buffer[rgba_next_row..rgba_next_row + 4]
                            .copy_from_slice(&snow_index_to_rgba(c2_low));
                        rgba_buffer[rgba_next_row + 4..rgba_next_row + 8]
                            .copy_from_slice(&snow_index_to_rgba(c2_high));
                    }
                } else {
                    rgba_buffer[rgba_off..rgba_off + 4]
                        .copy_from_slice(&snow_index_to_rgba(flake.c1 as u8));
                }
            }
        }
    }

    pub fn render_to_viewport(
        &mut self,
        viewport: &mut Rc<[u8]>,
        mask: &[u8],
        previous_camera: (i32, i32),
        camera: (i32, i32),
        wind: i32,
        draw: bool,
    ) {
        if self.count() == 0 {
            return;
        }
        let mut pixels = viewport.to_vec();
        self.update(
            &mut pixels,
            mask,
            previous_camera.0 - camera.0,
            previous_camera.1 - camera.1,
            wind,
            draw,
        );
        *viewport = pixels.into();
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

    fn make_buffer_and_mask() -> (Vec<u8>, Vec<u8>) {
        let pixel_count = (WIDTH * HEIGHT) as usize;
        let rgba = vec![0u8; pixel_count * 4];
        let mask = vec![BG_MIN; pixel_count];
        (rgba, mask)
    }

    #[test]
    fn update_uses_pascal_wrapped_offset_for_offscreen_flakes() {
        let mut snow = SnowSystem::new();
        snow.flakes[0] = flake_at(0, 205_i64 << 10, 233);
        snow.max = 0;

        let (mut rgba, mask) = make_buffer_and_mask();
        snow.update(&mut rgba, &mask, 0, 0, 0, false);

        let expected_rgba = snow_index_to_rgba(233);
        let pixel_start = 64 * 4;
        assert_eq!(&rgba[pixel_start..pixel_start + 4], &expected_rgba);
    }

    #[test]
    fn update_respects_pascal_inclusive_max_count() {
        let mut snow = SnowSystem::new();
        snow.flakes[0] = flake_at(10_i64 << 10, 10_i64 << 10, 233);
        snow.flakes[1] = flake_at(20_i64 << 10, 10_i64 << 10, 234);
        snow.max = 0;

        let (mut rgba, mask) = make_buffer_and_mask();
        snow.update(&mut rgba, &mask, 0, 0, 0, false);

        let expected_233 = snow_index_to_rgba(233);
        let pixel0 = (10 + 10 * WIDTH as usize) * 4;
        assert_eq!(&rgba[pixel0..pixel0 + 4], &expected_233);

        let pixel1 = (20 + 10 * WIDTH as usize) * 4;
        assert_eq!(&rgba[pixel1..pixel1 + 4], &[0, 0, 0, 0]);
    }
}

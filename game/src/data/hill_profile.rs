use crate::data::records::HillInfo;
use crate::loaders::assets::AssetStore;
use crate::parsers::pcx::{DecodedPcx, PcxParser};
use crate::parsers::AssetParser;
use std::rc::Rc;

pub const HILL_PROFILE_LEN: usize = 1300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HillTerrain {
    front_pixels: Rc<[u8]>,
    back_pixels: Rc<[u8]>,
    pub width: u16,
    pub height: u16,
    back_width: u16,
    back_height: u16,
    line_lengths: Vec<usize>,
    profile_y: Vec<i32>,
    pub keula_x: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_front_pcx_profile_and_takeoff_point() {
        let pcx =
            PcxParser::parse(include_bytes!("../../assets/FRONT1.PCX")).expect("valid FRONT1.PCX");
        let terrain = HillTerrain::from_front_pcx(pcx, 120, 0.89);

        assert_eq!(terrain.width, 1024);
        assert_eq!(terrain.height, 512);
        assert!(terrain.keula_x > 0);
        assert!(terrain.profiili(terrain.keula_x) > 0);
        assert_eq!(terrain.maki_kulma(terrain.keula_x), 0);
        assert_eq!(terrain.profiili(1299), terrain.profiili(1023));
    }
}

impl HillTerrain {
    pub fn load(info: &HillInfo) -> Result<Self, String> {
        let front_data = AssetStore::read(&format!("FRONT{}.PCX", info.front_index))
            .map_err(|e| e.to_string())?;
        let back_data =
            AssetStore::read(&format!("BACK{}.PCX", info.back_index)).map_err(|e| e.to_string())?;
        let front = PcxParser::parse(&front_data).map_err(|e| e.to_string())?;
        let mut back = PcxParser::parse(&back_data).map_err(|e| e.to_string())?;
        if info.back_mirror != 0 {
            Self::mirror_pixels(&mut back.pixels, back.width as usize, back.height as usize);
        }
        Ok(Self::from_pcxs(front, back, info.kr, info.pk()))
    }

    pub fn from_front_pcx(pcx: DecodedPcx, kr: i64, pk: f64) -> Self {
        Self::from_pcxs(pcx.clone(), pcx, kr, pk)
    }

    pub fn from_pcxs(front: DecodedPcx, back: DecodedPcx, kr: i64, pk: f64) -> Self {
        let width = front.width as usize;
        let height = front.height as usize;
        let mut pixels = front.pixels;
        let line_lengths = Self::line_lengths(&pixels, width, height);
        let profile_y = Self::profile_y(&line_lengths, width, height);
        let keula_x = Self::keula_x(&profile_y, width);
        Self::draw_distance_markers(&mut pixels, width, &profile_y, keula_x, kr, pk);

        Self {
            front_pixels: pixels.into(),
            back_pixels: back.pixels.into(),
            width: front.width,
            height: front.height,
            back_width: back.width,
            back_height: back.height,
            line_lengths,
            profile_y,
            keula_x,
        }
    }

    pub fn viewport_pixels(&self, scroll_x: i32, scroll_y: i32, w: u32, h: u32) -> Rc<[u8]> {
        let mut out = vec![0; w as usize * h as usize];
        for dy in 0..h as i32 {
            let front_y = scroll_y + dy;
            let back_y = scroll_y / 2 + dy;
            for dx in 0..w as i32 {
                let front_x = scroll_x + dx;
                let back_x = scroll_x / 2 + dx;
                let pixel = if self.is_front_pixel(front_x, front_y) {
                    self.front_pixel(front_x, front_y)
                } else {
                    self.back_pixel(back_x, back_y)
                };
                out[dy as usize * w as usize + dx as usize] = pixel;
            }
        }
        out.into()
    }

    fn is_front_pixel(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        (x as usize)
            < self
                .line_lengths
                .get(y as usize)
                .copied()
                .unwrap_or_default()
    }

    fn front_pixel(&self, x: i32, y: i32) -> u8 {
        let idx = y as usize * self.width as usize + x as usize;
        self.front_pixels.get(idx).copied().unwrap_or_default()
    }

    fn back_pixel(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.back_width as i32 || y >= self.back_height as i32 {
            return 0;
        }
        let idx = y as usize * self.back_width as usize + x as usize;
        self.back_pixels.get(idx).copied().unwrap_or_default()
    }

    fn mirror_pixels(pixels: &mut [u8], width: usize, height: usize) {
        for y in 0..height {
            let row = &mut pixels[y * width..(y + 1) * width];
            row.reverse();
        }
    }

    pub fn profiili(&self, x: i32) -> i32 {
        if x > 0 {
            self.profile_y.get(x as usize).copied().unwrap_or(0)
        } else {
            0
        }
    }

    pub fn maki_kulma(&self, x: i32) -> i32 {
        let value = self.profiili(x + 9)
            + self.profiili(x + 8)
            + self.profiili(x + 7)
            + self.profiili(x + 6)
            - self.profiili(x - 3)
            - self.profiili(x - 4)
            - self.profiili(x - 5)
            - self.profiili(x - 2);

        if x > self.keula_x - 15 && x <= self.keula_x {
            0
        } else {
            value
        }
    }

    fn line_lengths(pixels: &[u8], width: usize, height: usize) -> Vec<usize> {
        let mut lines = vec![0; height];
        for (y, line) in lines.iter_mut().enumerate() {
            let row = &pixels[y * width..(y + 1) * width];
            if let Some(x) = row.iter().rposition(|&p| p != 0) {
                *line = x + 1;
            }
        }
        lines
    }

    fn profile_y(line_lengths: &[usize], width: usize, height: usize) -> Vec<i32> {
        let mut profile = vec![0; HILL_PROFILE_LEN];
        for (x, y_out) in profile.iter_mut().take(width).enumerate() {
            let y = line_lengths
                .iter()
                .position(|&line_len| line_len > x)
                .unwrap_or(height.saturating_sub(1));
            *y_out = y as i32;
        }
        let last = profile[width.saturating_sub(1).min(HILL_PROFILE_LEN - 1)];
        for y_out in profile.iter_mut().skip(width) {
            *y_out = last;
        }
        profile
    }

    fn keula_x(profile_y: &[i32], width: usize) -> i32 {
        let mut keula_x = 0;
        let mut former_y = 0;
        for (x, &y) in profile_y.iter().take(width).enumerate() {
            if y - former_y > 3 {
                keula_x = x as i32;
            }
            former_y = y;
        }
        keula_x - 1
    }

    fn draw_distance_markers(
        pixels: &mut [u8],
        width: usize,
        profile_y: &[i32],
        keula_x: i32,
        kr: i64,
        pk: f64,
    ) {
        let keula_idx = keula_x.max(0) as usize;
        let drawable_width = width.min(profile_y.len());
        for x in keula_idx..drawable_width.saturating_sub(10) {
            let x2 = x as i64 - keula_x as i64;
            let y2 = profile_y[x] as i64 - profile_y[keula_idx] as i64;
            let hp = ((((x2 * x2 + y2 * y2) as f64).sqrt() * pk * 0.5).round() as i64) * 5;
            if hp >= (2 * kr * 10) / 3 && hp <= kr * 12 {
                let color = if hp < kr * 10 { 238 } else { 239 };
                for dy in 0..3 {
                    let y = profile_y[x] + dy + 1;
                    if y >= 0 {
                        let idx = y as usize * width + x;
                        if let Some(pixel) = pixels.get_mut(idx) {
                            *pixel = color;
                        }
                    }
                }
            }
        }
    }
}

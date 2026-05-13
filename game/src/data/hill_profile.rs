use crate::data::records::HillInfo;
use crate::loaders::assets::AssetStore;
use crate::parsers::pcx::{DecodedPcx, PcxParser};
use crate::parsers::AssetParser;
use std::rc::Rc;

pub const HILL_PROFILE_LEN: usize = 1300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HillTerrain {
    pub pixels: Rc<[u8]>,
    pub width: u16,
    pub height: u16,
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
        let data = AssetStore::read(&format!("FRONT{}.PCX", info.front_index))
            .map_err(|e| e.to_string())?;
        let pcx = PcxParser::parse(&data).map_err(|e| e.to_string())?;
        Ok(Self::from_front_pcx(pcx, info.kr, info.pk()))
    }

    pub fn from_front_pcx(pcx: DecodedPcx, kr: i64, pk: f64) -> Self {
        let width = pcx.width as usize;
        let height = pcx.height as usize;
        let mut pixels = pcx.pixels;
        let line_lengths = Self::line_lengths(&pixels, width, height);
        let profile_y = Self::profile_y(&line_lengths, width, height);
        let keula_x = Self::keula_x(&profile_y, width);
        Self::draw_distance_markers(&mut pixels, width, &profile_y, keula_x, kr, pk);

        Self {
            pixels: pixels.into(),
            width: pcx.width,
            height: pcx.height,
            profile_y,
            keula_x,
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

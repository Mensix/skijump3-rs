use crate::data::hill::HillInfo;
use crate::parsers::pcx::{DecodedPcx, PcxParser};
use crate::parsers::AssetParser;
use crate::save::files::FileStore;
use engine::palette::Palette;
use std::rc::Rc;

pub const HILL_PROFILE_LEN: usize = 1300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HillTerrain {
    front_pixels: Rc<[u8]>,
    back_pixels: Rc<[u8]>,
    front_palette: Palette,
    back_palette: Palette,
    pub width: u16,
    pub height: u16,
    back_width: u16,
    back_height: u16,
    line_lengths: Vec<usize>,
    profile_y: Vec<i32>,
    pub tip_x: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::AssetParser;

    #[test]
    fn extracts_front_pcx_profile_and_takeoff_point() {
        let pcx =
            PcxParser::parse(include_bytes!("../../assets/FRONT1.PCX")).expect("valid FRONT1.PCX");
        let terrain = HillTerrain::from_front_pcx(pcx, 120, 0.89);

        assert_eq!(terrain.width, 1024);
        assert_eq!(terrain.height, 512);
        assert!(terrain.tip_x > 0);
        assert!(terrain.height_at(terrain.tip_x) > 0);
        assert_eq!(terrain.hill_angle(terrain.tip_x), 0);
        assert_eq!(terrain.height_at(1299), terrain.height_at(1023));
    }

    #[test]
    fn back_pcx_loads_nonzero_pixels() {
        let front =
            PcxParser::parse(include_bytes!("../../assets/FRONT1.PCX")).expect("FRONT1.PCX");
        let back = PcxParser::parse(include_bytes!("../../assets/BACK0.PCX")).expect("BACK0.PCX");
        let terrain = HillTerrain::from_pcxs(front, back, 120, 0.89);

        assert_eq!(terrain.back_width, 1024);
        assert_eq!(terrain.back_height, 400);

        let pixel = terrain.back_pixel(50, 5);
        assert_ne!(
            pixel, 0,
            "back pixel at (50,5) should not be 0, got {}",
            pixel
        );

        let vp = terrain.viewport_pixels(0, 0, 320, 200);
        let row5_nonzero = vp[5 * 320..6 * 320].iter().filter(|&&p| p != 0).count();
        assert!(
            row5_nonzero > 0,
            "row 5 of viewport should have non-zero back pixels, got 0/320"
        );
        // Also check that row 5 at col 50 is non-zero
        assert_ne!(vp[5 * 320 + 50], 0, "vp pixel at (50,5) should be non-zero");
    }

    #[test]
    fn line_lengths_for_sky_rows_are_zero() {
        let pcx = PcxParser::parse(include_bytes!("../../assets/FRONT1.PCX")).expect("FRONT1.PCX");
        let terrain = HillTerrain::from_front_pcx(pcx, 120, 0.89);
        for y in 0..15 {
            assert_eq!(
                terrain.line_lengths[y], 0,
                "row {} should have line_length=0 (pure sky), got {}",
                y, terrain.line_lengths[y]
            );
        }
        assert!(terrain.line_lengths[200] > 0, "row 200 should have terrain");
    }
}

impl HillTerrain {
    pub fn load(files: &FileStore, info: &HillInfo) -> Result<Self, String> {
        let front_data = files
            .read(&format!("FRONT{}.PCX", info.front_index))
            .map_err(|e| e.to_string())?;
        let front = PcxParser::parse(&front_data).map_err(|e| e.to_string())?;
        let back_data = files
            .read(&format!("BACK{}.PCX", info.back_index))
            .map_err(|e| e.to_string())?;
        let mut back = PcxParser::parse(&back_data).map_err(|e| e.to_string())?;
        if info.back_mirror != 0 {
            Self::mirror_pixels(&mut back.pixels, back.width as usize, back.height as usize);
        }
        Ok(Self::from_pcxs(front, back, info.kr, info.pk()))
    }

    #[must_use]
    pub fn from_front_pcx(pcx: DecodedPcx, kr: i64, pk: f64) -> Self {
        let width = pcx.width as usize;
        let height = pcx.height as usize;
        let pixels = pcx.pixels;
        let line_lengths = Self::line_lengths(&pixels, width, height);
        let profile_y = Self::profile_y(&line_lengths, width, height);
        let tip_x = Self::tip_x(&profile_y, width);
        let mut front_pixels = pixels.clone();
        Self::draw_distance_markers(&mut front_pixels, width, &profile_y, tip_x, kr, pk);
        Self {
            front_pixels: front_pixels.into(),
            back_pixels: pixels.into(),
            front_palette: pcx.palette.clone(),
            back_palette: pcx.palette,
            width: pcx.width,
            height: pcx.height,
            back_width: pcx.width,
            back_height: pcx.height,
            line_lengths,
            profile_y,
            tip_x,
        }
    }

    #[must_use]
    pub fn from_pcxs(front: DecodedPcx, back: DecodedPcx, kr: i64, pk: f64) -> Self {
        let width = front.width as usize;
        let height = front.height as usize;
        let mut pixels = front.pixels;
        let line_lengths = Self::line_lengths(&pixels, width, height);
        let profile_y = Self::profile_y(&line_lengths, width, height);
        let tip_x = Self::tip_x(&profile_y, width);
        Self::draw_distance_markers(&mut pixels, width, &profile_y, tip_x, kr, pk);

        Self {
            front_pixels: pixels.into(),
            back_pixels: back.pixels.into(),
            front_palette: front.palette,
            back_palette: back.palette,
            width: front.width,
            height: front.height,
            back_width: back.width,
            back_height: back.height,
            line_lengths,
            profile_y,
            tip_x,
        }
    }

    pub fn apply_hill_palette(&self, palette: &mut Palette) {
        // FRONT images use low indices for terrain; BACK images use 65..213 for sky.
        // Pascal loads both PCXs into one indexed buffer, so we compose the two palettes here
        // while preserving standard UI colors in 216..255.
        for i in 0..=64 {
            palette.set(i, self.front_palette.color(i));
        }
        for i in 65..=215 {
            palette.set(i, self.back_palette.color(i));
        }
    }

    #[must_use]
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
        if x < 0 || y < 0 || x >= i32::from(self.width) || y >= i32::from(self.height) {
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
        if x < 0 || y < 0 || x >= i32::from(self.back_width) || y >= i32::from(self.back_height) {
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

    #[must_use]
    pub fn height_at(&self, x: i32) -> i32 {
        if x > 0 {
            self.profile_y.get(x as usize).copied().unwrap_or(0)
        } else {
            0
        }
    }

    #[must_use]
    pub fn hill_angle(&self, x: i32) -> i32 {
        let value = self.height_at(x + 9)
            + self.height_at(x + 8)
            + self.height_at(x + 7)
            + self.height_at(x + 6)
            - self.height_at(x - 3)
            - self.height_at(x - 4)
            - self.height_at(x - 5)
            - self.height_at(x - 2);

        if x > self.tip_x - 15 && x <= self.tip_x {
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
                .unwrap_or_else(|| height.saturating_sub(1));
            *y_out = y as i32;
        }
        let last = profile[width.saturating_sub(1).min(HILL_PROFILE_LEN - 1)];
        for y_out in profile.iter_mut().skip(width) {
            *y_out = last;
        }
        profile
    }

    fn tip_x(profile_y: &[i32], width: usize) -> i32 {
        let mut tip_x = 0;
        let mut former_y = 0;
        for (x, &y) in profile_y.iter().take(width).enumerate() {
            if y - former_y > 3 {
                tip_x = x as i32;
            }
            former_y = y;
        }
        tip_x - 1
    }

    fn draw_distance_markers(
        pixels: &mut [u8],
        width: usize,
        profile_y: &[i32],
        tip_x: i32,
        kr: i64,
        pk: f64,
    ) {
        let tip_idx = tip_x.max(0) as usize;
        let drawable_width = width.min(profile_y.len());
        for x in tip_idx..drawable_width.saturating_sub(10) {
            let x2 = x as i64 - i64::from(tip_x);
            let y2 = i64::from(profile_y[x]) - i64::from(profile_y[tip_idx]);
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

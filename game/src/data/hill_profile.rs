use crate::error::AssetError;
use crate::files::FileStore;
use crate::gfx::png::{load_grayscale_png, load_png};
use serde::Deserialize;
use std::rc::Rc;

pub const HILL_PROFILE_LEN: usize = 1300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HillTerrain {
    front_pixels: Rc<[u8]>,
    back_pixels: Rc<[u8]>,
    front_rgba: Rc<[u8]>,
    back_rgba: Rc<[u8]>,
    pub width: u16,
    pub height: u16,
    back_width: u16,
    back_height: u16,
    line_lengths: Vec<usize>,
    profile_y: Vec<i32>,
    pub tip_x: i32,
}

#[derive(Deserialize)]
struct TerrainMetadata {
    format_version: u32,
    width: u16,
    height: u16,
    back_width: u16,
    back_height: u16,
    tip_x: i32,
    line_lengths: Vec<i64>,
    profile_y: Vec<i64>,
}

impl HillTerrain {
    pub fn load(files: &FileStore, hill_idx: usize) -> Result<Self, AssetError> {
        let dir = format!("hills/generated/HILL{hill_idx}/");

        let meta_bytes = files.read(&format!("{dir}terrain.toml")).map_err(|e| {
            let path = format!("{dir}terrain.toml");
            AssetError::io(path, e)
        })?;
        let meta_str = std::str::from_utf8(&meta_bytes).map_err(|e| {
            let path = format!("{dir}terrain.toml");
            AssetError::utf8(path, e)
        })?;
        let meta: TerrainMetadata = toml::from_str(meta_str).map_err(|e| {
            let path = format!("{dir}terrain.toml");
            AssetError::toml(path, e)
        })?;

        if meta.format_version != 1 {
            return Err(AssetError::format_version(
                format!("{dir}terrain.toml"),
                1,
                meta.format_version,
            ));
        }

        let front_rgba_raw = files
            .read(&format!("{dir}front_rgba.png"))
            .map_err(|e| AssetError::io(format!("{dir}front_rgba.png"), e))?;
        let front_rgba_img = load_png(&front_rgba_raw)?;
        let front_mask_raw = files
            .read(&format!("{dir}front_mask.png"))
            .map_err(|e| AssetError::io(format!("{dir}front_mask.png"), e))?;
        let front_mask = load_grayscale_png(&front_mask_raw)?;

        let back_rgba_raw = files
            .read(&format!("{dir}back_rgba.png"))
            .map_err(|e| AssetError::io(format!("{dir}back_rgba.png"), e))?;
        let back_rgba_img = load_png(&back_rgba_raw)?;
        let back_mask_raw = files
            .read(&format!("{dir}back_mask.png"))
            .map_err(|e| AssetError::io(format!("{dir}back_mask.png"), e))?;
        let back_mask = load_grayscale_png(&back_mask_raw)?;

        let w = meta.width as usize;
        let h = meta.height as usize;
        let bw = meta.back_width as usize;
        let bh = meta.back_height as usize;

        if front_rgba_img.width as usize != w
            || front_rgba_img.height as usize != h
            || front_mask.width as usize != w
            || front_mask.height as usize != h
        {
            return Err(AssetError::Custom(format!(
                "Hill {hill_idx}: front image dimensions mismatch (expected {w}x{h})"
            )));
        }
        if back_rgba_img.width as usize != bw
            || back_rgba_img.height as usize != bh
            || back_mask.width as usize != bw
            || back_mask.height as usize != bh
        {
            return Err(AssetError::Custom(format!(
                "Hill {hill_idx}: back image dimensions mismatch (expected {bw}x{bh})"
            )));
        }

        let line_lengths: Vec<usize> = meta.line_lengths.iter().map(|&v| v as usize).collect();
        if line_lengths.len() != h {
            return Err(AssetError::Custom(format!(
                "Hill {hill_idx}: expected {h} line_lengths, got {}",
                line_lengths.len()
            )));
        }
        let profile_y: Vec<i32> = meta.profile_y.iter().map(|&v| v as i32).collect();
        if profile_y.len() != HILL_PROFILE_LEN {
            return Err(AssetError::Custom(format!(
                "Hill {hill_idx}: expected {HILL_PROFILE_LEN} profile_y entries, got {}",
                profile_y.len()
            )));
        }

        Ok(Self {
            front_pixels: front_mask.pixels.into(),
            back_pixels: back_mask.pixels.into(),
            front_rgba: front_rgba_img.pixels.into(),
            back_rgba: back_rgba_img.pixels.into(),
            width: meta.width,
            height: meta.height,
            back_width: meta.back_width,
            back_height: meta.back_height,
            line_lengths,
            profile_y,
            tip_x: meta.tip_x,
        })
    }

    // ------------------------------------------------------------------
    // Viewport / geometry methods (unchanged)
    // ------------------------------------------------------------------

    pub fn viewport_rgba_and_mask(
        &self,
        scroll_x: i32,
        scroll_y: i32,
        w: u32,
        h: u32,
    ) -> (Vec<u8>, Vec<u8>) {
        let wu = w as usize;
        let hu = h as usize;
        let mut rgba = vec![0u8; wu * hu * 4];
        let mut mask = vec![0u8; wu * hu];
        let width_u = self.width as usize;
        let back_w = self.back_width as usize;

        for dy in 0..h as i32 {
            let front_y = scroll_y + dy;
            let back_y = scroll_y / 2 + dy;
            let row_base = dy as usize * wu;
            for dx in 0..w as i32 {
                let front_x = scroll_x + dx;
                let back_x = scroll_x / 2 + dx;
                let is_front = if front_x >= 0
                    && front_y >= 0
                    && (front_x as usize) < width_u
                    && (front_y as usize) < self.height as usize
                {
                    (front_x as usize)
                        < self
                            .line_lengths
                            .get(front_y as usize)
                            .copied()
                            .unwrap_or_default()
                } else {
                    false
                };
                let out_idx = row_base + dx as usize;
                let (pixel, src_rgba, src_w, src_x, src_y) = if is_front {
                    let sy = front_y as usize;
                    let sx = front_x as usize;
                    let src_idx = sy * width_u + sx;
                    (
                        self.front_pixels.get(src_idx).copied().unwrap_or(0),
                        &self.front_rgba,
                        width_u,
                        sx,
                        sy,
                    )
                } else {
                    if back_x < 0 || back_y < 0 {
                        mask[out_idx] = 0;
                        continue;
                    }
                    let sx = back_x as usize;
                    let sy = back_y as usize;
                    if sx < back_w && sy < self.back_height as usize {
                        let src_idx = sy * back_w + sx;
                        (
                            self.back_pixels.get(src_idx).copied().unwrap_or(0),
                            &self.back_rgba,
                            back_w,
                            sx,
                            sy,
                        )
                    } else {
                        mask[out_idx] = 0;
                        continue;
                    }
                };
                mask[out_idx] = pixel;
                let rgba_src_offset = (src_y * src_w + src_x) * 4;
                let rgba_dst = &mut rgba[out_idx * 4..out_idx * 4 + 4];
                rgba_dst.copy_from_slice(&src_rgba[rgba_src_offset..rgba_src_offset + 4]);
            }
        }
        (rgba, mask)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_files() -> crate::files::FileStore {
        let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        crate::files::FileStore::new(assets, std::path::PathBuf::from("."))
    }

    #[test]
    fn back_pcx_loads_nonzero_pixels() {
        let terrain = HillTerrain::load(&test_files(), 0).expect("HILL0");
        let (rgba, mask) = terrain.viewport_rgba_and_mask(0, 0, 320, 200);
        assert_eq!(rgba.len(), 320 * 200 * 4);
        assert_eq!(mask.len(), 320 * 200);
        let sky = mask.iter().filter(|&&m| m > 64 && m <= 215).count();
        assert!(sky > 0, "expected some sky-background pixels");
    }

    #[test]
    fn extracts_front_pcx_profile_and_takeoff_point() {
        let terrain = HillTerrain::load(&test_files(), 0).expect("HILL0");
        assert!(terrain.tip_x > 0, "expected positive tip_x");
        let max_profile = terrain.profile_y.iter().max().copied().unwrap_or(0);
        assert!(max_profile > 0, "expected non-zero profile");
    }

    #[test]
    fn terrain_rejects_bad_format() {
        let err = HillTerrain::load(&test_files(), 9999)
            .unwrap_err()
            .to_string();
        assert!(err.contains("Failed"), "{err}");
    }

    #[test]
    fn viewport_produces_correct_dimensions() {
        let terrain = HillTerrain::load(&test_files(), 0).expect("HILL0");
        let (rgba, mask) = terrain.viewport_rgba_and_mask(50, 30, 100, 80);
        assert_eq!(rgba.len(), 100 * 80 * 4);
        assert_eq!(mask.len(), 100 * 80);
    }

    #[test]
    fn height_at_and_hill_angle() {
        let terrain = HillTerrain::load(&test_files(), 0).expect("HILL0");
        let h = terrain.height_at(terrain.tip_x);
        assert!(h >= 0, "height at tip should be non-negative");
        let angle = terrain.hill_angle(terrain.tip_x + 10);
        if terrain.tip_x + 10 > terrain.tip_x - 15 && terrain.tip_x + 10 <= terrain.tip_x {
            assert_eq!(angle, 0);
        }
    }

    #[test]
    fn load_hill0_has_correct_dimensions() {
        let terrain = HillTerrain::load(&test_files(), 0).expect("HILL0");
        assert_eq!(terrain.width, 1024);
        assert_eq!(terrain.height, 512);
    }

    #[test]
    fn load_hill1_has_correct_dimensions() {
        let terrain = HillTerrain::load(&test_files(), 1).expect("HILL1");
        assert_eq!(terrain.width, 1024);
        assert_eq!(terrain.height, 512);
    }
}

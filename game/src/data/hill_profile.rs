use crate::files::FileStore;
use crate::gfx::png::load_png;
use std::rc::Rc;

const PROFILE_LEN: usize = 1300;
const MARKER_RED: [u8; 4] = [255, 93, 93, 255];
const MARKER_BLUE: [u8; 4] = [93, 93, 255, 255];

#[derive(Debug, Clone, PartialEq)]
pub struct HillTerrain {
    front_visual: Rc<[u8]>,
    back_visual: Rc<[u8]>,
    pub width: u16,
    pub height: u16,
    back_width: u16,
    back_height: u16,
    line_lengths: Vec<usize>,
    profile_y: Vec<i32>,
    pub tip_x: i32,
    kr: i64,
    pk: f64,
}

impl HillTerrain {
    pub fn load(files: &FileStore, terrain_id: impl std::fmt::Display) -> Self {
        Self::load_with_markers(files, terrain_id, 0, 0.0)
    }

    pub fn load_with_markers(
        files: &FileStore,
        terrain_id: impl std::fmt::Display,
        kr: i64,
        pk: f64,
    ) -> Self {
        let terrain_id = terrain_id.to_string();
        let dir = format!("hills/generated/HILL{terrain_id}/");

        let front_visual_raw = files.read(&format!("{dir}front.png"));
        let front_visual = load_png(&front_visual_raw);

        let back_visual_raw = files.read(&format!("{dir}back.png"));
        let back_visual = load_png(&back_visual_raw);

        let width = front_visual.width as u16;
        let height = front_visual.height as u16;
        let back_width = back_visual.width as u16;
        let back_height = back_visual.height as u16;

        let (line_lengths, profile_y, tip_x) =
            Self::compute_terrain_from_front(&front_visual.pixels, width, height);

        Self {
            front_visual: front_visual.pixels.into(),
            back_visual: back_visual.pixels.into(),
            width,
            height,
            back_width,
            back_height,
            line_lengths,
            profile_y,
            tip_x,
            kr,
            pk,
        }
    }

    fn compute_terrain_from_front(
        pixels: &[u8],
        width: u16,
        height: u16,
    ) -> (Vec<usize>, Vec<i32>, i32) {
        let w = width as usize;
        let h = height as usize;

        let background = &pixels[0..3];
        let mut line_lengths = Vec::with_capacity(h);
        for y in 0..h {
            let mut last = None;
            for x in 0..w {
                let pixel = &pixels[(y * w + x) * 4..(y * w + x) * 4 + 4];
                if is_profile_pixel(pixel, background) {
                    last = Some(x);
                }
            }
            line_lengths.push(last.map_or(0, |x| x + 1));
        }

        let mut profile_y = Vec::with_capacity(PROFILE_LEN);
        for x in 0..w {
            let mut y = 0i32;
            for (candidate_y, &line_len) in line_lengths.iter().enumerate() {
                y = candidate_y as i32;
                if line_len > x {
                    break;
                }
            }
            profile_y.push(y);
        }
        profile_y.resize(PROFILE_LEN, *profile_y.last().unwrap_or(&0));

        let mut tip_x = 0i32;
        let mut former_y = 0i32;
        for (x, &y) in profile_y.iter().take(w).enumerate() {
            if y - former_y > 3 {
                tip_x = x as i32;
            }
            former_y = y;
        }
        (line_lengths, profile_y, tip_x - 1)
    }

    pub fn viewport_rgba_and_mask(
        &self,
        scroll_x: i32,
        scroll_y: i32,
        w: u32,
        h: u32,
    ) -> (Vec<u8>, Vec<u8>) {
        self.viewport_rgba_and_mask_with_back(scroll_x, scroll_y, w, h, true)
    }

    pub fn viewport_rgba_and_mask_with_back(
        &self,
        scroll_x: i32,
        scroll_y: i32,
        w: u32,
        h: u32,
        draw_back: bool,
    ) -> (Vec<u8>, Vec<u8>) {
        let wu = w as usize;
        let hu = h as usize;
        let mut rgba = vec![0u8; wu * hu * 4];
        for px in rgba.chunks_exact_mut(4) {
            px[3] = 255;
        }
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
                let out_idx = row_base + dx as usize;

                if draw_back && back_x >= 0 && back_y >= 0 {
                    let sx = back_x as usize;
                    let sy = back_y as usize;
                    if sx < back_w && sy < self.back_height as usize {
                        let rgba_src_offset = (sy * back_w + sx) * 4;
                        mask[out_idx] = 128;
                        let rgba_dst = &mut rgba[out_idx * 4..out_idx * 4 + 4];
                        rgba_dst.copy_from_slice(
                            &self.back_visual[rgba_src_offset..rgba_src_offset + 4],
                        );
                        rgba_dst[3] = 255;
                    }
                }

                let front_in_line = if front_x >= 0
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

                if front_in_line {
                    let sy = front_y as usize;
                    let sx = front_x as usize;
                    let rgba_src_offset = (sy * width_u + sx) * 4;
                    mask[out_idx] = 255;
                    let rgba_dst = &mut rgba[out_idx * 4..out_idx * 4 + 4];
                    rgba_dst
                        .copy_from_slice(&self.front_visual[rgba_src_offset..rgba_src_offset + 4]);
                    rgba_dst[3] = 255;
                }
            }
        }
        self.draw_distance_markers(&mut rgba, &mut mask, scroll_x, scroll_y, w, h);
        (rgba, mask)
    }

    fn draw_distance_markers(
        &self,
        rgba: &mut [u8],
        mask: &mut [u8],
        scroll_x: i32,
        scroll_y: i32,
        w: u32,
        h: u32,
    ) {
        if self.kr <= 0 || self.pk <= 0.0 || self.tip_x < 0 {
            return;
        }
        let wu = w as usize;
        let hu = h as usize;
        let tip_x = self.tip_x as usize;
        let Some(&tip_y) = self.profile_y.get(tip_x) else {
            return;
        };
        for x in tip_x..(self.width as usize).saturating_sub(10) {
            let Some(&ground_y) = self.profile_y.get(x) else {
                continue;
            };
            let dx = x as i32 - self.tip_x;
            let dy = ground_y - tip_y;
            let hp = ((f64::from(dx * dx + dy * dy).sqrt() * self.pk * 0.5).round() as i64) * 5;
            if hp < ((2.0 / 3.0) * self.kr as f64 * 10.0) as i64 || hp > self.kr * 12 {
                continue;
            }
            let color = if hp < self.kr * 10 {
                MARKER_RED
            } else {
                MARKER_BLUE
            };
            let sx = x as i32 - scroll_x;
            if sx < 0 || sx as usize >= wu {
                continue;
            }
            for marker_dy in 0..3 {
                let sy = ground_y + 1 + marker_dy - scroll_y;
                if sy < 0 || sy as usize >= hu {
                    continue;
                }
                let out_idx = sy as usize * wu + sx as usize;
                mask[out_idx] = 255;
                rgba[out_idx * 4..out_idx * 4 + 4].copy_from_slice(&color);
            }
        }
    }

    pub fn height_at(&self, x: i32) -> i32 {
        if x > 0 {
            self.profile_y.get(x as usize).copied().unwrap_or(0)
        } else {
            0
        }
    }

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

fn is_profile_pixel(pixel: &[u8], background: &[u8]) -> bool {
    &pixel[0..3] != background && pixel != MARKER_RED && pixel != MARKER_BLUE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::FileStore;

    fn test_files() -> FileStore {
        let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        FileStore::new(assets, std::path::PathBuf::from("."))
    }

    #[test]
    fn back_mask_loads_nonzero_pixels() {
        let terrain = HillTerrain::load(&test_files(), 1);
        let (rgba, mask) = terrain.viewport_rgba_and_mask(0, 0, 320, 200);
        assert_eq!(rgba.len(), 320 * 200 * 4);
        assert_eq!(mask.len(), 320 * 200);
        let sky = mask.iter().filter(|&&m| m > 64 && m <= 215).count();
        assert!(sky > 0, "expected some sky-background pixels");
    }

    #[test]
    fn extracts_front_profile_and_takeoff_point() {
        let terrain = HillTerrain::load(&test_files(), 1);
        assert!(terrain.tip_x > 0, "expected positive tip_x");
        let max_profile = terrain.profile_y.iter().max().copied().unwrap_or(0);
        assert!(max_profile > 0, "expected non-zero profile");
    }

    #[test]
    fn viewport_produces_correct_dimensions() {
        let terrain = HillTerrain::load(&test_files(), 1);
        let (rgba, mask) = terrain.viewport_rgba_and_mask(50, 30, 100, 80);
        assert_eq!(rgba.len(), 100 * 80 * 4);
        assert_eq!(mask.len(), 100 * 80);
    }

    #[test]
    fn height_at_and_hill_angle() {
        let terrain = HillTerrain::load(&test_files(), 1);
        let h = terrain.height_at(terrain.tip_x);
        assert!(h >= 0, "height at tip should be non-negative");
        let angle = terrain.hill_angle(terrain.tip_x + 10);
        if terrain.tip_x + 10 > terrain.tip_x - 15 && terrain.tip_x + 10 <= terrain.tip_x {
            assert_eq!(angle, 0);
        }
    }

    #[test]
    fn load_hill1_has_correct_dimensions() {
        let terrain = HillTerrain::load(&test_files(), 1);
        assert_eq!(terrain.width, 1024);
        assert_eq!(terrain.height, 512);
    }

    #[test]
    fn load_hill2_has_correct_dimensions() {
        let terrain = HillTerrain::load(&test_files(), 2);
        assert_eq!(terrain.width, 1024);
        assert_eq!(terrain.height, 512);
    }

    #[test]
    fn computes_takeoff_points_from_front_visuals() {
        let files = test_files();
        let expected = [
            272, 268, 269, 279, 281, 293, 258, 246, 275, 278, 253, 291, 243, 279, 258, 267, 264,
            268, 232, 278,
        ];
        for (hill_id, tip_x) in expected.into_iter().enumerate() {
            let terrain = HillTerrain::load(&files, hill_id);
            assert_eq!(terrain.tip_x, tip_x, "HILL{hill_id}");
        }
    }

    #[test]
    fn planica_distance_marker_is_rendered_runtime() {
        let terrain = HillTerrain::load_with_markers(&test_files(), 19, 185, 1.06);
        let (rgba, mask) = terrain.viewport_rgba_and_mask(704, 312, 320, 200);
        let pos = rgba
            .chunks_exact(4)
            .position(|p| p == [255, 93, 93, 255])
            .expect("runtime red marker");
        assert_eq!(mask[pos], 255);
        assert_eq!(&rgba[pos * 4..pos * 4 + 4], &[255, 93, 93, 255]);
    }
}

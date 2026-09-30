use crate::data::hill::generated_hill_image_path;
use crate::files::FileStore;
use crate::gfx::png::load_png;
use engine::oxide::StaticImage;

const PROFILE_LEN: usize = 1300;
const MARKER_RED: [u8; 4] = [255, 93, 93, 255];
const MARKER_BLUE: [u8; 4] = [93, 93, 255, 255];

#[derive(Debug, Clone, Default)]
pub struct HillTerrain {
    front_layer: StaticImage,
    back_layer: StaticImage,
    profile_y: Vec<i32>,
    pub tip_x: i32,
}

impl HillTerrain {
    pub fn load_with_markers(
        files: &FileStore,
        front_index: &str,
        back_index: &str,
        back_brightness: i64,
        back_mirror: bool,
        kr: i64,
        pk: f64,
    ) -> Self {
        let front_visual_raw =
            files.read_save_or_asset(&generated_hill_image_path(front_index, "front"));
        let Some(front_visual) = load_png(&front_visual_raw) else {
            return Self::default();
        };

        let back_visual_raw =
            files.read_save_or_asset(&generated_hill_image_path(back_index, "back"));
        let Some(mut back_visual) = load_png(&back_visual_raw) else {
            return Self::default();
        };
        apply_back_brightness(&mut back_visual.pixels, back_brightness);
        if back_mirror {
            mirror_image_x(
                &mut back_visual.pixels,
                back_visual.width,
                back_visual.height,
            );
        }

        let width = front_visual.width as u16;
        let height = front_visual.height as u16;
        let (line_lengths, profile_y, tip_x) =
            Self::compute_terrain_from_front(&front_visual.pixels, width, height);
        let front_layer = make_front_layer(
            FrontVisual {
                pixels: &front_visual.pixels,
                width,
                height,
                line_lengths: &line_lengths,
                profile_y: &profile_y,
            },
            tip_x,
            kr,
            pk,
        );
        let back_layer = StaticImage::new(
            back_visual.pixels.clone(),
            back_visual.width,
            back_visual.height,
        );

        Self {
            front_layer,
            back_layer,
            profile_y,
            tip_x,
        }
    }

    pub(crate) fn front_layer(&self) -> StaticImage {
        self.front_layer.clone()
    }

    pub(crate) fn back_layer(&self) -> StaticImage {
        self.back_layer.clone()
    }

    fn compute_terrain_from_front(
        pixels: &[u8],
        width: u16,
        height: u16,
    ) -> (Vec<usize>, Vec<i32>, i32) {
        let w = width as usize;
        let h = height as usize;

        let background = &pixels[0..3];
        let alpha_meaningful = pixels.chunks_exact(4).any(|pixel| pixel[3] < 255);
        let mut line_lengths = Vec::with_capacity(h);
        for y in 0..h {
            let mut last = None;
            for x in 0..w {
                let pixel = &pixels[(y * w + x) * 4..(y * w + x) * 4 + 4];
                if is_profile_pixel(pixel, background, alpha_meaningful) {
                    last = Some(x);
                }
            }
            line_lengths.push(last.map_or(0, |x| x + 1));
        }

        let mut profile_y = Vec::with_capacity(PROFILE_LEN);
        for x in 0..w {
            let y = line_lengths
                .iter()
                .position(|&line_len| line_len > x)
                .unwrap_or(h) as i32;
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

struct FrontVisual<'a> {
    pixels: &'a [u8],
    width: u16,
    height: u16,
    line_lengths: &'a [usize],
    profile_y: &'a [i32],
}

fn make_front_layer(visual: FrontVisual<'_>, tip_x: i32, kr: i64, pk: f64) -> StaticImage {
    let FrontVisual {
        pixels,
        width,
        height,
        line_lengths,
        profile_y,
    } = visual;
    let width = width as usize;
    let height = height as usize;
    let mut layer = vec![0u8; width * height * 4];
    for y in 0..height {
        let line_len = line_lengths.get(y).copied().unwrap_or_default().min(width);
        let row_start = y * width * 4;
        let copy_len = line_len * 4;
        layer[row_start..row_start + copy_len]
            .copy_from_slice(&pixels[row_start..row_start + copy_len]);
        for pixel in layer[row_start..row_start + copy_len].chunks_exact_mut(4) {
            pixel[3] = 255;
        }
    }
    if kr > 0 && pk > 0.0 && tip_x >= 0 {
        draw_distance_markers(&mut layer, profile_y, tip_x, kr, pk, width, height);
    }
    StaticImage::new(layer, width as u32, height as u32)
}

fn mirror_image_x(pixels: &mut [u8], width: u32, height: u32) {
    let row_len = width as usize * 4;
    for row in pixels.chunks_exact_mut(row_len).take(height as usize) {
        for x in 0..width as usize / 2 {
            let left = x * 4;
            let right = (width as usize - 1 - x) * 4;
            for channel in 0..4 {
                row.swap(left + channel, right + channel);
            }
        }
    }
}

fn draw_distance_markers(
    rgba: &mut [u8],
    profile_y: &[i32],
    tip_x: i32,
    kr: i64,
    pk: f64,
    width: usize,
    height: usize,
) {
    let Some(&tip_y) = profile_y.get(tip_x as usize) else {
        return;
    };
    for x in tip_x..profile_y.len().saturating_sub(10) as i32 {
        let Some(&ground_y) = profile_y.get(x as usize) else {
            continue;
        };
        let dx = x - tip_x;
        let dy = ground_y - tip_y;
        let hp = ((f64::from(dx * dx + dy * dy).sqrt() * pk * 0.5).round() as i64) * 5;
        if hp < ((2.0 / 3.0) * kr as f64 * 10.0) as i64 || hp > kr * 12 {
            continue;
        }
        let color = if hp < kr * 10 {
            MARKER_RED
        } else {
            MARKER_BLUE
        };
        if x < 0 || x as usize >= width {
            continue;
        }
        for marker_dy in 0..3 {
            let y = ground_y + 1 + marker_dy;
            if y < 0 || y as usize >= height {
                continue;
            }
            let offset = y as usize * width + x as usize;
            rgba[offset * 4..offset * 4 + 4].copy_from_slice(&color);
        }
    }
}

fn is_profile_pixel(pixel: &[u8], background: &[u8], alpha_meaningful: bool) -> bool {
    if alpha_meaningful {
        pixel[3] >= 128
    } else {
        &pixel[0..3] != background
    }
}

fn apply_back_brightness(pixels: &mut [u8], brightness: i64) {
    let brightness = brightness.clamp(0, 255);
    for pixel in pixels.chunks_exact_mut(4) {
        for channel in &mut pixel[..3] {
            *channel = (i64::from(*channel) * brightness / 100).min(255) as u8;
        }
    }
}

pub fn profile_checksum(files: &FileStore, front_index: &str) -> Option<i64> {
    let data = files.read_save_or_asset(&generated_hill_image_path(front_index, "front"));
    let image = load_png(&data)?;
    let width = usize::try_from(image.width).ok()?;
    let height = usize::try_from(image.height).ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    let pixels = &image.pixels;
    let background = [pixels[0], pixels[1], pixels[2]];
    let mut line_lengths = vec![0usize; height];
    for (y, line_len) in line_lengths.iter_mut().enumerate() {
        let mut last = 0usize;
        for x in 0..width {
            let offset = (y * width + x) * 4;
            if pixels[offset..offset + 3] != background {
                last = x + 1;
            }
        }
        *line_len = last;
    }
    let profile_at = |x: usize| -> i64 {
        if x == 0 {
            return 0;
        }
        let xx = x.min(width - 1);
        line_lengths
            .iter()
            .position(|&line_len| line_len > xx)
            .map_or(height - 1, |y| y) as i64
    };
    let mut sum = 0i64;
    for temp in 0..1024 {
        let profile = profile_at(temp);
        sum += (profile * ((temp as i64 % 13) + (profile % 11))) % 13313;
    }
    Some(sum - 1500000)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HillProfileMismatch {
    pub front_index: String,
    pub exiting_cup: bool,
}

pub(crate) fn stored_profile_matches(files: &FileStore, front_index: &str, stored: i64) -> bool {
    if front_index.starts_with("SJH") {
        return true;
    }
    profile_checksum(files, front_index).is_some_and(|computed| computed == stored)
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
    fn extracts_front_profile_and_takeoff_point() {
        let terrain = terrain("1", "1", 100, false);
        assert!(terrain.tip_x > 0, "expected positive tip_x");
        let max_profile = terrain.profile_y.iter().max().copied().unwrap_or(0);
        assert!(max_profile > 0, "expected non-zero profile");
    }

    #[test]
    fn alpha_defines_profile_when_transparent_rgb_varies() {
        let mut pixels = vec![0u8; 4 * 4 * 4];
        for y in 0..4 {
            for x in 0..=y {
                let i = (y * 4 + x) * 4;
                pixels[i..i + 4].copy_from_slice(&[20, 30, 40, 255]);
            }
            for x in y + 1..4 {
                let i = (y * 4 + x) * 4;
                pixels[i..i + 4].copy_from_slice(&[200, 100, 50, 0]);
            }
        }
        let (lines, profile, _) = HillTerrain::compute_terrain_from_front(&pixels, 4, 4);
        assert_eq!(lines, vec![1, 2, 3, 4]);
        assert_eq!(&profile[..4], &[0, 1, 2, 3]);
    }

    #[test]
    fn height_at_and_hill_angle() {
        let terrain = terrain("1", "1", 100, false);
        let h = terrain.height_at(terrain.tip_x);
        assert!(h >= 0, "height at tip should be non-negative");
        let angle = terrain.hill_angle(terrain.tip_x + 10);
        if terrain.tip_x + 10 > terrain.tip_x - 15 && terrain.tip_x + 10 <= terrain.tip_x {
            assert_eq!(angle, 0);
        }
    }

    #[test]
    fn load_hill1_has_correct_dimensions() {
        let terrain = terrain("1", "1", 100, false);
        assert_eq!(terrain.front_layer.width(), 1024);
        assert_eq!(terrain.front_layer.height(), 512);
    }

    #[test]
    fn load_hill2_has_correct_dimensions() {
        let terrain = terrain("2", "2", 100, false);
        assert_eq!(terrain.front_layer.width(), 1024);
        assert_eq!(terrain.front_layer.height(), 512);
    }

    #[test]
    fn computes_takeoff_points_from_front_visuals() {
        let files = test_files();
        let expected = [
            272, 268, 269, 279, 281, 293, 258, 246, 275, 278, 253, 291, 243, 279, 258, 267, 264,
            268, 232, 278,
        ];
        for (hill_id, tip_x) in expected.into_iter().enumerate() {
            let hill_id = hill_id.to_string();
            let terrain =
                HillTerrain::load_with_markers(&files, &hill_id, &hill_id, 100, false, 0, 0.0);
            assert_eq!(terrain.tip_x, tip_x, "HILL{hill_id}");
        }
    }

    #[test]
    fn distance_markers_are_baked_into_front_layer() {
        let terrain =
            HillTerrain::load_with_markers(&test_files(), "19", "19", 100, false, 185, 1.06);
        assert!(terrain
            .front_layer
            .pixels()
            .chunks_exact(4)
            .any(|pixel| pixel == MARKER_RED));
    }

    fn terrain(front: &str, back: &str, brightness: i64, mirror: bool) -> HillTerrain {
        HillTerrain::load_with_markers(&test_files(), front, back, brightness, mirror, 0, 0.0)
    }

    #[test]
    fn front_and_back_indices_are_independent() {
        let mixed = terrain("1", "4", 100, false);
        let front = terrain("1", "1", 100, false);
        let back = terrain("4", "4", 100, false);

        assert_eq!(mixed.profile_y, front.profile_y);
        assert_eq!(mixed.tip_x, front.tip_x);
        assert_eq!(mixed.back_layer.pixels(), back.back_layer.pixels());
    }

    #[test]
    fn back_brightness_and_mirror_change_static_layer_only() {
        let normal = terrain("1", "4", 100, false);
        let dark = terrain("1", "4", 50, false);
        let mirrored = terrain("1", "4", 100, true);
        assert_ne!(normal.back_layer, dark.back_layer);
        assert_ne!(normal.back_layer, mirrored.back_layer);
        assert_eq!(normal.profile_y, dark.profile_y);
        assert_eq!(normal.tip_x, dark.tip_x);
        assert!(dark
            .back_layer
            .pixels()
            .chunks_exact(4)
            .zip(normal.back_layer.pixels().chunks_exact(4))
            .all(|(dark, original)| dark[..3]
                .iter()
                .zip(&original[..3])
                .all(|(dark, original)| dark <= original)));
    }
}

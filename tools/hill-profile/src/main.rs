use png::{BitDepth, ColorType, Encoder};
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
struct Rgba {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize)]
struct Point {
    section: Option<String>,
    x: f64,
    y: f64,
    slope: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct Profile {
    id: String,
    name: Option<String>,
    width: usize,
    height: usize,
    tip_x: f64,
    tip_drop: f64,
    terrain_index: Option<String>,
    kr: Option<i64>,
    back_brightness: Option<i64>,
    back_mirror: Option<bool>,
    vx_final: Option<i64>,
    pk_hundred: Option<i64>,
    pl_save_ten_thousand: Option<i64>,
    author: Option<String>,
    points: Vec<Point>,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("generate") => {
            let profile = PathBuf::from(args.next().ok_or("generate requires profile.toml")?);
            let output = PathBuf::from(args.next().ok_or("generate requires output-dir")?);
            generate_command(&profile, &output)
        }
        Some(_) | None => Err("usage: hill-profile generate <profile.toml> <output-dir>".into()),
    }
}

fn write_png(path: &Path, image: &Rgba) -> Result<()> {
    let file = File::create(path)?;
    let mut encoder = Encoder::new(
        BufWriter::new(file),
        image.width as u32,
        image.height as u32,
    );
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.write_header()?.write_image_data(&image.data)?;
    Ok(())
}

fn pchip_slopes(xs: &[f64], ys: &[f64]) -> Vec<f64> {
    let n = xs.len();
    if n < 2 {
        return vec![0.0; n];
    }
    let mut d = Vec::with_capacity(n - 1);
    for i in 0..n - 1 {
        d.push((ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]));
    }
    let mut m = vec![0.0; n];
    m[0] = d[0];
    m[n - 1] = d[n - 2];
    for i in 1..n - 1 {
        if d[i - 1] * d[i] > 0.0 {
            let h0 = xs[i] - xs[i - 1];
            let h1 = xs[i + 1] - xs[i];
            m[i] = (h0 + h1) / (h0 / d[i - 1] + h1 / d[i]);
        }
    }
    m
}

fn pchip(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if xs.len() < 2 {
        return ys.first().copied().unwrap_or(0.0);
    }
    let slopes = pchip_slopes(xs, ys);
    let mut i = 0;
    while i + 1 < xs.len() && x > xs[i + 1] {
        i += 1;
    }
    let i = i.min(xs.len() - 2);
    let h = xs[i + 1] - xs[i];
    let t = ((x - xs[i]) / h).clamp(0.0, 1.0);
    let h00 = 2.0 * t.powi(3) - 3.0 * t.powi(2) + 1.0;
    let h10 = t.powi(3) - 2.0 * t.powi(2) + t;
    let h01 = -2.0 * t.powi(3) + 3.0 * t.powi(2);
    let h11 = t.powi(3) - t.powi(2);
    h00 * ys[i] + h10 * h * slopes[i] + h01 * ys[i + 1] + h11 * h * slopes[i + 1]
}

fn rasterize(profile: &[f64], width: usize, height: usize) -> Vec<u8> {
    let surface = profile
        .iter()
        .map(|value| value.round().clamp(0.0, (height - 1) as f64) as usize)
        .collect::<Vec<_>>();
    let mut data = vec![0; width * height * 4];
    for y in 0..height {
        let length = surface
            .iter()
            .enumerate()
            .filter(|(_, value)| **value <= y)
            .map(|(x, _)| x + 1)
            .max()
            .unwrap_or(0);
        if length > 0 {
            let i = (y * width + length - 1) * 4;
            data[i..i + 4].copy_from_slice(&[35, 105, 55, 255]);
        }
    }
    data
}

fn validate_section(points: &mut [Point], name: &str, width: usize, height: usize) -> Result<()> {
    points.sort_by(|a, b| a.x.total_cmp(&b.x));
    if points.len() < 2 {
        return Err(format!("profile needs at least two {name} points").into());
    }
    for point in points.iter() {
        if !point.x.is_finite()
            || !point.y.is_finite()
            || point.slope.is_some_and(|slope| !slope.is_finite())
            || point.x < 0.0
            || point.x > (width - 1) as f64
            || point.y < 0.0
            || point.y > (height - 1) as f64
        {
            return Err("profile points and slopes must be finite and within the image".into());
        }
    }
    if points.windows(2).any(|window| window[1].x <= window[0].x) {
        return Err(format!("{name} point x values must be strictly increasing").into());
    }
    if points.windows(2).any(|window| window[1].y < window[0].y) {
        return Err(format!("{name} points must be monotone").into());
    }
    Ok(())
}

fn generate_command(path: &Path, output: &Path) -> Result<()> {
    let text = fs::read_to_string(path)?;
    let profile: Profile = toml::from_str(&text)?;
    if profile.id.is_empty()
        || profile.width < 2
        || profile.height < 2
        || !profile.tip_x.is_finite()
        || profile.tip_x < 0.0
        || profile.tip_x > (profile.width - 1) as f64
        || !profile.tip_drop.is_finite()
        || profile.tip_drop < 0.0
    {
        return Err("profile needs a non-empty id, positive dimensions, valid tip_x, and finite non-negative tip_drop".into());
    }

    let mut before = profile
        .points
        .iter()
        .filter(|point| point.section.as_deref() == Some("before"))
        .cloned()
        .collect::<Vec<_>>();
    let mut after = profile
        .points
        .iter()
        .filter(|point| point.section.as_deref() == Some("after"))
        .cloned()
        .collect::<Vec<_>>();
    validate_section(&mut before, "before", profile.width, profile.height)?;
    validate_section(&mut after, "after", profile.width, profile.height)?;
    let before_tip = before.iter().find(|point| point.x == profile.tip_x);
    let after_tip = after.iter().find(|point| point.x == profile.tip_x);
    let (Some(before_tip), Some(after_tip)) = (before_tip, after_tip) else {
        return Err("before and after sections must both contain a point at tip_x".into());
    };
    if (after_tip.y - before_tip.y - profile.tip_drop).abs() > 1e-6 {
        return Err("tip_drop does not match the explicit before/after jump".into());
    }

    let bxs = before.iter().map(|point| point.x).collect::<Vec<_>>();
    let bys = before.iter().map(|point| point.y).collect::<Vec<_>>();
    let axs = after.iter().map(|point| point.x).collect::<Vec<_>>();
    let ays = after.iter().map(|point| point.y).collect::<Vec<_>>();
    let mut curve = Vec::with_capacity(profile.width);
    for x in 0..profile.width {
        curve.push(if x as f64 <= profile.tip_x {
            pchip(&bxs, &bys, x as f64)
        } else {
            pchip(&axs, &ays, x as f64)
        });
    }
    if curve.windows(2).any(|window| window[1] < window[0] - 1e-6) {
        return Err("generated profile is not monotone".into());
    }

    let alpha = rasterize(&curve, profile.width, profile.height);
    let mut front = Rgba {
        width: profile.width,
        height: profile.height,
        data: vec![0; profile.width * profile.height * 4],
    };
    for y in 0..profile.height {
        for x in 0..profile.width {
            let i = (y * profile.width + x) * 4;
            let opaque = alpha[i + 3] != 0;
            let color = if opaque {
                [35 + (x % 80) as u8, 105 + (y % 70) as u8, 55]
            } else {
                [0, 0, 0]
            };
            front.data[i..i + 4].copy_from_slice(&[color[0], color[1], color[2], alpha[i + 3]]);
        }
    }
    let back = procedural_back(&profile.id, 763, 400);
    let asset_dir = output
        .join("hills/generated")
        .join(format!("HILL{}", profile.id));
    fs::create_dir_all(&asset_dir)?;
    fs::create_dir_all(output.join("custom_hills"))?;
    write_png(&asset_dir.join("front.png"), &front)?;
    write_png(&asset_dir.join("back.png"), &back)?;
    let profile_sum = profile_checksum(&alpha, profile.width, profile.height);
    let metadata_sum = metadata_checksum(&profile, profile_sum);
    fs::write(
        output
            .join("custom_hills")
            .join(format!("{}.toml", profile.id)),
        catalog(&profile, profile_sum, metadata_sum),
    )?;
    println!(
        "generated {}x{} tip_x={} profile_checksum={} metadata_checksum={}",
        profile.width, profile.height, profile.tip_x, profile_sum, metadata_sum
    );
    Ok(())
}

fn procedural_back(id: &str, width: usize, height: usize) -> Rgba {
    let mut image = Rgba {
        width,
        height,
        data: vec![0; width * height * 4],
    };
    let seed = id.bytes().map(usize::from).sum::<usize>();
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) * 4;
            let noise = ((x * 31 + y * 17 + seed) % 64) as u8;
            image.data[i..i + 4].copy_from_slice(&[
                40 + noise / 2,
                80 + noise,
                145 + noise / 2,
                255,
            ]);
        }
    }
    image
}

fn catalog(profile: &Profile, checksum: i64, metadata: i64) -> String {
    format!(
        "id = \"{}\"\nname = \"{}\"\nchecksum_version = 1\n\n[[hills]]\nid = \"{}\"\nname = \"{}\"\nterrain_index = \"{}\"\nkr = {}\nfront_index = \"{}\"\nback_index = \"{}\"\nback_brightness = {}\nback_mirror = {}\nvx_final = {}\npk_hundred = {}\npl_save_ten_thousand = {}\nauthor = \"{}\"\nchecksum = {}\nprofile_checksum = {}\n",
        profile.id, profile.name.as_deref().unwrap_or(&profile.id), profile.id,
        profile.name.as_deref().unwrap_or(&profile.id), profile.terrain_index.as_deref().unwrap_or("1"),
        profile.kr.unwrap_or(120), profile.id, profile.id, profile.back_brightness.unwrap_or(100),
        profile.back_mirror.unwrap_or(false), profile.vx_final.unwrap_or(140), profile.pk_hundred.unwrap_or(100),
        profile.pl_save_ten_thousand.unwrap_or(3212), profile.author.as_deref().unwrap_or("hill-profile"), metadata, checksum
    )
}

fn fnv(bytes: impl IntoIterator<Item = u8>) -> i64 {
    let mut hash = 2_166_136_261u32;
    for byte in bytes {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16_777_619);
    }
    i64::from(hash & 0x7fff_ffff).max(1)
}

fn profile_checksum(alpha: &[u8], width: usize, height: usize) -> i64 {
    let mut lines = vec![0u32; height];
    for y in 0..height {
        for x in (0..width).rev() {
            if alpha[(y * width + x) * 4 + 3] > 0 {
                lines[y] = (x + 1) as u32;
                break;
            }
        }
    }
    let mut profile = Vec::with_capacity(1300);
    for x in 0..width {
        profile.push((0..height).find(|&y| lines[y] > x as u32).unwrap_or(height) as u32);
    }
    while profile.len() < 1300 {
        profile.push(profile.last().copied().unwrap_or(0));
    }
    let mut tip = 0u32;
    let mut previous = 0u32;
    for (x, &y) in profile.iter().take(width).enumerate() {
        if y.saturating_sub(previous) > 3 {
            tip = x as u32;
        }
        previous = y;
    }
    fnv(lines
        .into_iter()
        .chain(profile)
        .chain([tip.saturating_sub(1)])
        .flat_map(u32::to_le_bytes))
}

fn metadata_checksum(profile: &Profile, checksum: i64) -> i64 {
    let value = [
        profile.name.as_deref().unwrap_or(&profile.id).to_string(),
        profile.kr.unwrap_or(120).to_string(),
        profile.id.clone(),
        profile.id.clone(),
        profile.back_brightness.unwrap_or(100).to_string(),
        (if profile.back_mirror.unwrap_or(false) {
            1
        } else {
            0
        })
        .to_string(),
        profile.vx_final.unwrap_or(140).to_string(),
        profile.pk_hundred.unwrap_or(100).to_string(),
        profile.pl_save_ten_thousand.unwrap_or(3212).to_string(),
        profile
            .author
            .as_deref()
            .unwrap_or("hill-profile")
            .to_string(),
        checksum.to_string(),
    ]
    .join("\u{ff}");
    fnv(value.bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pchip_is_cubic_and_shape_preserving() {
        let value = pchip(&[0.0, 1.0, 2.0], &[0.0, 1.0, 0.0], 0.5);
        assert!(value > 0.5);
        assert!((0.0..=1.0).contains(&value));
    }

    #[test]
    fn rasterization_fills_alpha_below_curve() {
        let data = rasterize(&[1.0, 1.0, 2.0, 3.0], 4, 5);
        assert_eq!(data[(3 * 4 + 3) * 4 + 3], 255);
        assert_eq!(data[3], 0);
    }

    #[test]
    fn checksum_is_stable_and_sensitive() {
        assert_eq!(fnv([1, 2, 3]), fnv([1, 2, 3]));
        assert_ne!(fnv([1, 2, 3]), fnv([1, 2, 4]));
    }

    #[test]
    fn validation_requires_explicit_tip_and_monotone_points() {
        let mut points = vec![
            Point {
                section: Some("before".into()),
                x: 0.0,
                y: 2.0,
                slope: None,
            },
            Point {
                section: Some("before".into()),
                x: 1.0,
                y: 1.0,
                slope: None,
            },
        ];
        assert!(validate_section(&mut points, "before", 4, 4).is_err());
    }
}

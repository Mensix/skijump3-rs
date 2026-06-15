use std::env;
use std::error::Error;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

const PROFILE_LEN: usize = 1300;

#[derive(Debug)]
struct Args {
    root: PathBuf,
    source_dir: PathBuf,
    sjh_files: Vec<PathBuf>,
}

#[derive(Debug)]
struct LegacyHill {
    id: String,
    name: String,
    kr: i64,
    front_index: String,
    back_index: String,
    back_brightness: i64,
    back_mirror: i64,
    vx_final: i64,
    pk_hundred: i64,
    pl_save_ten_thousand: i64,
    author: String,
    checksum: i64,
    profile_checksum: i64,
}

#[derive(Debug)]
struct RgbaImage {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl RgbaImage {
    fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut [u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = (y * self.width + x) * 4;
        Some(&mut self.data[i..i + 4])
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;
    for sjh in &args.sjh_files {
        convert_one(&args, sjh)?;
    }
    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
    let mut root = env::current_dir()?;
    let mut source_dir = root.clone();
    let mut sjh_files = Vec::new();
    let mut raw = env::args().skip(1);
    while let Some(arg) = raw.next() {
        match arg.as_str() {
            "--root" => root = PathBuf::from(raw.next().ok_or("--root requires a path")?),
            "--source-dir" => {
                source_dir = PathBuf::from(raw.next().ok_or("--source-dir requires a path")?)
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}").into()),
            _ => sjh_files.push(PathBuf::from(arg)),
        }
    }
    if sjh_files.is_empty() {
        print_usage();
        return Err("at least one .SJH file is required".into());
    }
    Ok(Args {
        root: root.canonicalize().unwrap_or(root),
        source_dir: source_dir.canonicalize().unwrap_or(source_dir),
        sjh_files,
    })
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run -p convert-custom-hill -- [--root PATH] [--source-dir PATH] FILE.SJH..."
    );
}

fn convert_one(args: &Args, sjh_path: &Path) -> Result<(), Box<dyn Error>> {
    let sjh_path = sjh_path
        .canonicalize()
        .unwrap_or_else(|_| sjh_path.to_path_buf());
    let hill = read_sjh(&sjh_path)?;
    let search_dirs = [
        sjh_path.parent().unwrap_or(Path::new(".")),
        &args.source_dir,
        &args.root,
        args.root.parent().unwrap_or(&args.root),
    ];
    let front_path = find_source_file(&format!("FRONT{}.PCX", hill.front_index), &search_dirs)?;
    let back_path = find_source_file(&format!("BACK{}.PCX", hill.back_index), &search_dirs)?;
    let mut front = read_pcx_rgba(&front_path, true)?;
    let back = read_pcx_rgba(&back_path, false)?;
    let (line_lengths, profile_y, tip_x) = terrain_from_front(&front);
    bake_markers(
        &mut front,
        &profile_y,
        tip_x,
        hill.kr,
        hill.pk_hundred as f64 / 100.0,
    );

    let terrain_dir = args
        .root
        .join("hills")
        .join("generated")
        .join(format!("HILL{}", hill.front_index));
    fs::create_dir_all(&terrain_dir)?;
    write_png(&terrain_dir.join("front_visual.png"), &front)?;
    write_png(&terrain_dir.join("back_visual.png"), &back)?;
    write_terrain_toml(
        &terrain_dir.join("terrain.toml"),
        &front,
        &back,
        tip_x,
        &line_lengths,
        &profile_y,
    )?;

    let custom_dir = args.root.join("custom_hills");
    fs::create_dir_all(&custom_dir)?;
    write_custom_toml(&custom_dir.join(format!("{}.toml", hill.id)), &hill)?;
    println!(
        "converted {} -> custom_hills/{}.toml, HILL{}",
        sjh_path.display(),
        hill.id,
        hill.front_index
    );
    Ok(())
}

fn read_sjh(path: &Path) -> Result<LegacyHill, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 13 || !lines[0].starts_with('*') {
        return Err(format!("{} is not a supported SJH file", path.display()).into());
    }
    Ok(LegacyHill {
        id: path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("SJH filename must be valid UTF-8")?
            .to_string(),
        name: lines[1].to_string(),
        kr: lines[2].parse()?,
        front_index: lines[3].to_string(),
        back_index: lines[4].to_string(),
        back_brightness: lines[5].parse()?,
        back_mirror: lines[6].parse()?,
        vx_final: lines[7].parse()?,
        pk_hundred: lines[8].parse()?,
        pl_save_ten_thousand: lines[9].parse()?,
        author: lines[10].to_string(),
        checksum: lines[11].parse()?,
        profile_checksum: lines[12].parse()?,
    })
}

fn find_source_file(name: &str, dirs: &[&Path]) -> Result<PathBuf, Box<dyn Error>> {
    for dir in dirs {
        let path = dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(format!(
        "{name} not found in: {}",
        dirs.iter()
            .map(|d| d.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
    .into())
}

fn read_pcx_rgba(path: &Path, transparent_zero: bool) -> Result<RgbaImage, Box<dyn Error>> {
    let data = fs::read(path)?;
    if data.len() < 897 || data[0] != 0x0a || data[2] != 1 || data[3] != 8 || data[65] != 1 {
        return Err(format!("{} is not an 8-bit single-plane RLE PCX", path.display()).into());
    }
    let xmin = u16::from_le_bytes([data[4], data[5]]) as usize;
    let ymin = u16::from_le_bytes([data[6], data[7]]) as usize;
    let xmax = u16::from_le_bytes([data[8], data[9]]) as usize;
    let ymax = u16::from_le_bytes([data[10], data[11]]) as usize;
    let width = xmax - xmin + 1;
    let height = ymax - ymin + 1;
    let bytes_per_line = u16::from_le_bytes([data[66], data[67]]) as usize;
    let palette_pos = data.len() - 769;
    if data[palette_pos] != 0x0c {
        return Err(format!("{} has no VGA palette", path.display()).into());
    }
    let palette = &data[palette_pos + 1..];
    let mut decoded = Vec::with_capacity(bytes_per_line * height);
    let mut pos = 128;
    while pos < palette_pos && decoded.len() < bytes_per_line * height {
        let byte = data[pos];
        pos += 1;
        if byte & 0xc0 == 0xc0 {
            let count = (byte & 0x3f) as usize;
            let value = data[pos];
            pos += 1;
            decoded.extend(std::iter::repeat(value).take(count));
        } else {
            decoded.push(byte);
        }
    }
    if decoded.len() < bytes_per_line * height {
        return Err(format!("{} RLE data ended early", path.display()).into());
    }
    let mut rgba = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let idx = decoded[y * bytes_per_line + x] as usize;
            let out = (y * width + x) * 4;
            rgba[out] = palette[idx * 3];
            rgba[out + 1] = palette[idx * 3 + 1];
            rgba[out + 2] = palette[idx * 3 + 2];
            rgba[out + 3] = if transparent_zero && idx == 0 { 0 } else { 255 };
        }
    }
    Ok(RgbaImage {
        width,
        height,
        data: rgba,
    })
}

fn terrain_from_front(front: &RgbaImage) -> (Vec<usize>, Vec<i32>, i32) {
    let mut line_lengths = Vec::with_capacity(front.height);
    for y in 0..front.height {
        let mut last = None;
        for x in 0..front.width {
            if front.data[(y * front.width + x) * 4 + 3] != 0 {
                last = Some(x);
            }
        }
        line_lengths.push(last.map_or(0, |x| x + 1));
    }

    let mut profile_y = Vec::with_capacity(PROFILE_LEN);
    for x in 0..front.width {
        let mut y = 0;
        for (candidate_y, &line_len) in line_lengths.iter().enumerate() {
            y = candidate_y as i32;
            if line_len > x {
                break;
            }
        }
        profile_y.push(y);
    }
    profile_y.resize(PROFILE_LEN, *profile_y.last().unwrap_or(&0));

    let mut tip_x = 0;
    let mut former_y = 0;
    for (x, &y) in profile_y.iter().take(front.width).enumerate() {
        if y - former_y > 3 {
            tip_x = x as i32;
        }
        former_y = y;
    }
    (line_lengths, profile_y, tip_x - 1)
}

fn bake_markers(front: &mut RgbaImage, profile_y: &[i32], tip_x: i32, kr: i64, pk: f64) {
    for x in tip_x.max(0) as usize..front.width.saturating_sub(10) {
        let x2 = x as i32 - tip_x;
        let y2 = profile_y[x] - profile_y[tip_x as usize];
        let hp = (((x2 * x2 + y2 * y2) as f64).sqrt() * pk * 0.5).round() as i64 * 5;
        if hp >= ((2.0 / 3.0) * kr as f64 * 10.0) as i64 && hp <= kr * 12 {
            let color = if hp < kr * 10 {
                [255, 93, 93, 255]
            } else {
                [93, 93, 255, 255]
            };
            for dy in 0..3 {
                let y = profile_y[x] + 1 + dy;
                if let Some(px) = front.pixel_mut(x, y as usize) {
                    px.copy_from_slice(&color);
                }
            }
        }
    }
}

fn write_png(path: &Path, img: &RgbaImage) -> Result<(), Box<dyn Error>> {
    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, img.width as u32, img.height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&img.data)?;
    Ok(())
}

fn write_terrain_toml(
    path: &Path,
    front: &RgbaImage,
    back: &RgbaImage,
    tip_x: i32,
    line_lengths: &[usize],
    profile_y: &[i32],
) -> Result<(), Box<dyn Error>> {
    fs::write(
        path,
        format!(
            "format_version = 1\nwidth = {}\nheight = {}\nback_width = {}\nback_height = {}\ntip_x = {}\nline_lengths = [{}]\nprofile_y = [{}]\n",
            front.width,
            front.height,
            back.width,
            back.height,
            tip_x,
            line_lengths.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "),
            profile_y.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "),
        ),
    )?;
    Ok(())
}

fn write_custom_toml(path: &Path, hill: &LegacyHill) -> Result<(), Box<dyn Error>> {
    fs::write(
        path,
        format!(
            "id = \"{}\"\nname = \"{} custom hill\"\n\n[[hills]]\nid = \"{}\"\nname = \"{}\"\nterrain_index = \"{}\"\nkr = {}\nfront_index = \"{}\"\nback_index = \"{}\"\nback_brightness = {}\nback_mirror = {}\nvx_final = {}\npk_hundred = {}\npl_save_ten_thousand = {}\nauthor = \"{}\"\nchecksum = {}\nprofile_checksum = {}\n",
            hill.id,
            hill.name,
            hill.id,
            hill.name,
            hill.front_index,
            hill.kr,
            hill.front_index,
            hill.back_index,
            hill.back_brightness,
            hill.back_mirror != 0,
            hill.vx_final,
            hill.pk_hundred,
            hill.pl_save_ten_thousand,
            hill.author,
            hill.checksum,
            hill.profile_checksum,
        ),
    )?;
    Ok(())
}

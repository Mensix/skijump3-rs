use std::env;
use std::error::Error;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Args {
    root: PathBuf,
    source_dir: PathBuf,
    sjh_files: Vec<PathBuf>,
    write_custom: bool,
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
    let mut write_custom = true;
    let mut raw = env::args().skip(1);
    while let Some(arg) = raw.next() {
        match arg.as_str() {
            "--root" => root = PathBuf::from(raw.next().ok_or("--root requires a path")?),
            "--source-dir" => {
                source_dir = PathBuf::from(raw.next().ok_or("--source-dir requires a path")?)
            }
            "--assets-only" => write_custom = false,
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
        write_custom,
    })
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run -p convert-custom-hill -- [--root PATH] [--source-dir PATH] [--assets-only] FILE.SJH..."
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
    let front = read_pcx_rgba(&front_path)?;
    let back = read_pcx_rgba(&back_path)?;

    let terrain_dir = args
        .root
        .join("hills")
        .join("generated")
        .join(format!("HILL{}", hill.front_index));
    fs::create_dir_all(&terrain_dir)?;
    write_png(&terrain_dir.join("front.png"), &front)?;
    write_png(&terrain_dir.join("back.png"), &back)?;
    if args.write_custom {
        let custom_dir = args.root.join("custom_hills");
        fs::create_dir_all(&custom_dir)?;
        write_custom_toml(&custom_dir.join(format!("{}.toml", hill.id)), &hill)?;
    }
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

fn read_pcx_rgba(path: &Path) -> Result<RgbaImage, Box<dyn Error>> {
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
            decoded.extend(std::iter::repeat_n(value, count));
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
            rgba[out + 3] = 255;
        }
    }
    Ok(RgbaImage {
        width,
        height,
        data: rgba,
    })
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

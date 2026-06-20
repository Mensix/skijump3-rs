pub struct RgbaImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn load_png(data: &[u8]) -> RgbaImage {
    let img = image::load_from_memory(data).unwrap();
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    RgbaImage {
        pixels: rgba.into_raw(),
        width,
        height,
    }
}

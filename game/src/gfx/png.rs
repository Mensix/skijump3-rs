use crate::error::AssetError;

pub struct RgbaImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn load_png(data: &[u8]) -> Result<RgbaImage, AssetError> {
    let img = image::load_from_memory(data)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(RgbaImage {
        pixels: rgba.into_raw(),
        width,
        height,
    })
}



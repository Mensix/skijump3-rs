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

/// A grayscale image (1 byte per pixel).
pub struct GrayscaleImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Decode a PNG as 8-bit grayscale.
pub fn load_grayscale_png(data: &[u8]) -> Result<GrayscaleImage, AssetError> {
    let img = image::load_from_memory(data)?;
    let luma = img.to_luma8();
    let (width, height) = luma.dimensions();
    Ok(GrayscaleImage {
        pixels: luma.into_raw(),
        width,
        height,
    })
}

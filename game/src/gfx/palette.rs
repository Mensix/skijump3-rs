use engine::palette::Palette;
use serde::Deserialize;

use crate::error::AssetError;

/// A 256-entry 6-bit RGB palette asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgb6Palette {
    data: [u8; 768],
}

impl Rgb6Palette {
    pub fn from_toml_bytes(path: impl Into<String>, bytes: &[u8]) -> Result<Self, AssetError> {
        let path = path.into();
        let text = std::str::from_utf8(bytes).map_err(|e| AssetError::utf8(path.clone(), e))?;
        let toml: PaletteToml =
            toml::from_str(text).map_err(|e| AssetError::toml(path.clone(), e))?;
        if toml.format_version != 1 {
            return Err(AssetError::format_version(path, 1, toml.format_version));
        }
        Self::from_6bit_bytes(&toml.data)
    }

    pub fn from_6bit_bytes(bytes: &[u8]) -> Result<Self, AssetError> {
        if bytes.len() != 768 {
            return Err(AssetError::Custom(format!(
                "Palette: expected 768 bytes, got {}",
                bytes.len()
            )));
        }
        let mut data = [0u8; 768];
        data.copy_from_slice(bytes);
        Ok(Self { data })
    }

    #[must_use]
    pub fn into_palette(self) -> Palette {
        Palette::from_rgb6_bytes(&self.data).expect("validated 768-byte rgb6 palette")
    }
}

#[derive(Deserialize)]
struct PaletteToml {
    format_version: u32,
    data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_6bit_bytes_rejects_wrong_size() {
        assert!(Rgb6Palette::from_6bit_bytes(&[0; 767]).is_err());
        assert!(Rgb6Palette::from_6bit_bytes(&[0; 769]).is_err());
    }
}

use engine::color::Rgba;
use engine::sprite::SpriteColorRecolor;
use serde::Deserialize;

use crate::error::AssetError;
use crate::views::replay::PlaybackMode;

/// A 256-entry 6-bit RGB palette, used for sprite RGBA precomputation.
/// Each channel stores a 6-bit value (0-63).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgb6Palette {
    data: [u8; 768],
}

impl Rgb6Palette {
    /// Create from a TOML asset containing `format_version = 1` and 768 6-bit RGB bytes.
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

    /// Create from 768 bytes of pre-computed 6-bit RGB values.
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

    /// Return the 6-bit RGB triple for palette entry `idx`.
    pub fn color(&self, idx: usize) -> [u8; 3] {
        let off = idx * 3;
        [self.data[off], self.data[off + 1], self.data[off + 2]]
    }

    #[must_use]
    pub fn rgba_bytes(&self, idx: u8) -> [u8; 4] {
        if idx == 0 {
            return [0, 0, 0, 0];
        }
        let [r6, g6, b6] = self.color(idx as usize);
        [
            (u32::from(r6) * 255 / 63) as u8,
            (u32::from(g6) * 255 / 63) as u8,
            (u32::from(b6) * 255 / 63) as u8,
            255,
        ]
    }
}

#[derive(Deserialize)]
struct PaletteToml {
    format_version: u32,
    data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// RGBA UI color constants.
// ---------------------------------------------------------------------------

pub const FONT_DEFAULT: Rgba = Rgba::from_rgb6(63, 63, 63);
pub const FONT_HEADER: Rgba = Rgba::from_rgb6(63, 57, 9);
pub const FONT_GOLD: Rgba = FONT_HEADER;
pub const FONT_GREET: Rgba = Rgba::from_rgb6(9, 57, 63);
pub const FONT_NAME: Rgba = FONT_DEFAULT;
pub const FONT_NEW: Rgba = FONT_HEADER;
pub const FONT_BACK: Rgba = FONT_DEFAULT;
pub const FONT_HELP: Rgba = Rgba::from_rgb6(44, 44, 44);
pub const BG_ERASE: Rgba = Rgba::from_rgb6(5, 8, 20);
pub const BG_LIST: Rgba = BG_ERASE;
pub const BG_LEFT: Rgba = Rgba::from_rgb6(18, 13, 34); // 243
pub const BG_RIGHT: Rgba = Rgba::from_rgb6(34, 13, 18); // 244
pub const BG_ORDER: Rgba = BG_LEFT;

pub const BG_RIGHT_BRIGHT: Rgba = Rgba::from_rgb6(43, 16, 23);

// Additional fill/text colours from old palette indices
pub const BG_KOTH: Rgba = Rgba::from_rgb6(0, 25, 0); // NewScreen(1,2) -> MuutaMenu(1,4) -> ReplaceMenu col=4 KOTH
pub const BG_TEAMCUP: Rgba = Rgba::from_rgb6(28, 8, 24); // NewScreen(1,1) -> MuutaMenu(1,2) -> ReplaceMenu col=2
pub const FILL_BORDER: Rgba = Rgba::from_rgb6(23, 16, 43); // 248
pub const FILL_HIGHLIGHT: Rgba = Rgba::from_rgb6(52, 47, 0); // 251
pub const FILL_TURQUOISE: Rgba = Rgba::from_rgb6(0, 47, 52); // 252
pub const FILL_LINE: Rgba = Rgba::from_rgb6(5, 8, 22); // 9
pub const FILL_DIM: Rgba = Rgba::from_rgb6(20, 20, 20); // 244/245
pub const BLACK: Rgba = Rgba::rgb(0, 0, 0);

pub const JUMPER_SUIT_SOURCE_SHADE_1: u8 = 216;
pub const JUMPER_SUIT_SOURCE_SHADE_3: u8 = 218;
pub const JUMPER_SKI_SOURCE: u8 = 231;

const STANDARD_UI_COLOR_BASE: u8 = 216;

const STANDARD_UI_COLORS_RGB6: [[u8; 3]; 40] = [
    [53, 17, 53],
    [63, 0, 0],
    [43, 12, 43],
    [63, 0, 0],
    [49, 45, 0],
    [34, 31, 0],
    [63, 0, 0],
    [56, 54, 54],
    [63, 63, 21],
    [54, 52, 10],
    [42, 42, 42],
    [42, 20, 10],
    [21, 21, 21],
    [57, 45, 38],
    [63, 0, 0],
    [63, 63, 32],
    [40, 40, 41],
    [48, 48, 49],
    [55, 55, 56],
    [63, 63, 63],
    [56, 13, 13],
    [13, 53, 13],
    [23, 23, 63],
    [63, 23, 23],
    [63, 63, 63],
    [44, 44, 44],
    [0, 0, 0],
    [18, 13, 34],
    [34, 13, 18],
    [20, 20, 20],
    [63, 57, 9],
    [9, 57, 63],
    [23, 16, 43],
    [43, 16, 23],
    [26, 26, 26],
    [52, 47, 0],
    [0, 47, 52],
    [51, 51, 51],
    [38, 38, 38],
    [63, 63, 63],
];

#[must_use]
pub fn standard_ui_color(index: u8) -> Option<Rgba> {
    let offset = index.checked_sub(STANDARD_UI_COLOR_BASE)? as usize;
    let rgb = STANDARD_UI_COLORS_RGB6.get(offset)?;
    Some(Rgba::from_rgb6(rgb[0], rgb[1], rgb[2]))
}

#[must_use]
pub fn standard_ui_rgba_bytes(index: u8) -> Option<[u8; 4]> {
    let rgba = standard_ui_color(index)?;
    Some([rgba.r, rgba.g, rgba.b, rgba.a])
}

const SUIT_COLORS: [[u8; 4]; 8] = [
    [0, 53, 17, 53],
    [0, 55, 33, 11],
    [0, 11, 48, 18],
    [0, 24, 28, 63],
    [0, 63, 17, 17],
    [0, 33, 33, 33],
    [1, 10, 10, 10],
    [0, 45, 17, 63],
];

const SKI_COLORS: [[u8; 3]; 4] = [[63, 63, 32], [60, 60, 60], [33, 60, 33], [63, 43, 43]];

const SUIT_FADE_DOWN: [f32; 4] = [1.0, 0.87, 0.75, 0.63];
const SUIT_FADE_UP: [f32; 4] = [1.0, 1.50, 2.00, 2.50];

pub fn suit_shade_rgba(col: usize) -> [[u8; 3]; 4] {
    let col = col.min(SUIT_COLORS.len() - 1);
    let suit = SUIT_COLORS[col];
    let fade = if suit[0] == 0 {
        SUIT_FADE_DOWN
    } else {
        SUIT_FADE_UP
    };
    let mut colors = [[0u8; 3]; 4];
    for (i, &fd) in fade.iter().enumerate() {
        colors[i] = [
            (fd * f32::from(suit[1])).round().min(63.0) as u8,
            (fd * f32::from(suit[2])).round().min(63.0) as u8,
            (fd * f32::from(suit[3])).round().min(63.0) as u8,
        ];
    }
    colors
}

pub fn ski_rgb(col: usize) -> [u8; 3] {
    let col = col.min(SKI_COLORS.len() - 1);
    SKI_COLORS[col]
}

/// Return an arbitrary shade (0..4) of a suit colour as an Rgba.
#[must_use]
pub fn suit_color_shade(col: usize, shade: usize) -> Rgba {
    let rgb = suit_shade_rgba(col)[shade];
    Rgba::from_rgb6(rgb[0], rgb[1], rgb[2])
}

/// Return the ski colour as an Rgba.
#[must_use]
pub fn ski_color(col: usize) -> Rgba {
    let rgb = ski_rgb(col);
    Rgba::from_rgb6(rgb[0], rgb[1], rgb[2])
}

#[must_use]
pub fn start_light_recolor(is_dq: bool) -> SpriteColorRecolor {
    if is_dq {
        SpriteColorRecolor::new(vec![
            (253, Rgba::from_rgb6(54, 10, 10)),
            (254, Rgba::from_rgb6(47, 0, 0)),
        ])
    } else {
        SpriteColorRecolor::new(vec![
            (253, Rgba::from_rgb6(10, 54, 10)),
            (254, Rgba::from_rgb6(0, 47, 0)),
        ])
    }
}

#[must_use]
pub fn replay_speed_recolor(mode: PlaybackMode) -> SpriteColorRecolor {
    let active: u8 = match mode {
        PlaybackMode::Forward => 250,
        PlaybackMode::Rewind => 253,
        PlaybackMode::SpeedChange => 251,
        _ => 249,
    };
    let green = Rgba::from_rgb6(10, 63, 20);
    let black = Rgba::rgb(0, 0, 0);
    let mut pairs = Vec::with_capacity(5);
    for i in 249..=253 {
        pairs.push((i, if i == active { green } else { black }));
    }
    SpriteColorRecolor::new(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_6bit_bytes_preserves_values() {
        let mut raw = [0u8; 768];
        raw[0] = 63;
        raw[1] = 32;
        raw[2] = 0;
        let pal = Rgb6Palette::from_6bit_bytes(&raw).unwrap();
        assert_eq!(pal.color(0), [63, 32, 0]);
    }

    #[test]
    fn from_6bit_bytes_rejects_wrong_size() {
        assert!(Rgb6Palette::from_6bit_bytes(&[0; 767]).is_err());
        assert!(Rgb6Palette::from_6bit_bytes(&[0; 769]).is_err());
    }

    #[test]
    fn ui_background_colors_match_standard_ui_colors() {
        assert_eq!(BG_LEFT, Rgba::from_rgb6(18, 13, 34));
        assert_eq!(BG_RIGHT, Rgba::from_rgb6(34, 13, 18));
        assert_eq!(FILL_DIM, Rgba::from_rgb6(20, 20, 20));
    }
}

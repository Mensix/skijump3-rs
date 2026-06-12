use engine::color::Rgba;
use engine::sprite::SpriteColorRecolor;
use serde::Deserialize;

use crate::error::AssetError;
use crate::views::replay::PlaybackMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rgb6(u8, u8, u8);

impl Rgb6 {
    const BLACK: Self = Self(0, 0, 0);
    const TRANSPARENT_INDEX: u8 = 0;

    const fn rgba(self) -> Rgba {
        Rgba::from_rgb6(self.0, self.1, self.2)
    }
}

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
        if idx == Rgb6::TRANSPARENT_INDEX {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextTheme {
    pub default: Rgba,
    pub header: Rgba,
    pub gold: Rgba,
    pub greet: Rgba,
    pub name: Rgba,
    pub new: Rgba,
    pub back: Rgba,
    pub help: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundTheme {
    pub erase: Rgba,
    pub list: Rgba,
    pub left: Rgba,
    pub right: Rgba,
    pub order: Rgba,
    pub right_bright: Rgba,
    pub koth: Rgba,
    pub team_cup: Rgba,
    pub world_cup: Rgba,
    pub four_hills: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillTheme {
    pub border: Rgba,
    pub highlight: Rgba,
    pub turquoise: Rgba,
    pub line: Rgba,
    pub dim: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecolorTheme {
    pub start_dq_lit: Rgba,
    pub start_dq_dark: Rgba,
    pub start_ready_lit: Rgba,
    pub start_ready_dark: Rgba,
    pub replay_active: Rgba,
    pub replay_inactive: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub text: TextTheme,
    pub background: BackgroundTheme,
    pub fill: FillTheme,
    pub recolor: RecolorTheme,
    pub black: Rgba,
}

impl Theme {
    pub const DEFAULT: Self = Self {
        text: TextTheme {
            default: Rgb6(63, 63, 63).rgba(),
            header: Rgb6(63, 57, 9).rgba(),
            gold: Rgb6(63, 57, 9).rgba(),
            greet: Rgb6(9, 57, 63).rgba(),
            name: Rgb6(63, 63, 63).rgba(),
            new: Rgb6(63, 57, 9).rgba(),
            back: Rgb6(63, 63, 63).rgba(),
            help: Rgb6(44, 44, 44).rgba(),
        },
        background: BackgroundTheme {
            erase: Rgb6(5, 8, 20).rgba(),
            list: Rgb6(5, 8, 20).rgba(),
            left: Rgb6(18, 13, 34).rgba(),
            right: Rgb6(34, 13, 18).rgba(),
            order: Rgb6(18, 13, 34).rgba(),
            right_bright: Rgb6(43, 16, 23).rgba(),
            koth: Rgb6(0, 25, 0).rgba(),
            team_cup: Rgb6(28, 8, 24).rgba(),
            world_cup: Rgb6(47, 0, 0).rgba(),
            four_hills: Rgb6(10, 10, 10).rgba(),
        },
        fill: FillTheme {
            border: Rgb6(23, 16, 43).rgba(),
            highlight: Rgb6(52, 47, 0).rgba(),
            turquoise: Rgb6(0, 47, 52).rgba(),
            line: Rgb6(5, 8, 22).rgba(),
            dim: Rgb6(20, 20, 20).rgba(),
        },
        recolor: RecolorTheme {
            start_dq_lit: Rgb6(54, 10, 10).rgba(),
            start_dq_dark: Rgb6(47, 0, 0).rgba(),
            start_ready_lit: Rgb6(10, 54, 10).rgba(),
            start_ready_dark: Rgb6(0, 47, 0).rgba(),
            replay_active: Rgb6(10, 63, 20).rgba(),
            replay_inactive: Rgb6::BLACK.rgba(),
        },
        black: Rgb6::BLACK.rgba(),
    };
}

pub const THEME: Theme = Theme::DEFAULT;

pub const FONT_DEFAULT: Rgba = THEME.text.default;
pub const FONT_HEADER: Rgba = THEME.text.header;
pub const FONT_GOLD: Rgba = THEME.text.gold;
pub const FONT_GREET: Rgba = THEME.text.greet;
pub const FONT_NAME: Rgba = THEME.text.name;
pub const FONT_NEW: Rgba = THEME.text.new;
pub const FONT_BACK: Rgba = THEME.text.back;
pub const FONT_HELP: Rgba = THEME.text.help;
pub const BG_ERASE: Rgba = THEME.background.erase;
pub const BG_LIST: Rgba = THEME.background.list;
pub const BG_LEFT: Rgba = THEME.background.left;
pub const BG_RIGHT: Rgba = THEME.background.right;
pub const BG_ORDER: Rgba = THEME.background.order;
pub const BG_RIGHT_BRIGHT: Rgba = THEME.background.right_bright;
pub const BG_KOTH: Rgba = THEME.background.koth;
pub const BG_TEAMCUP: Rgba = THEME.background.team_cup;
pub const BG_WC: Rgba = THEME.background.world_cup;
pub const BG_4HILLS: Rgba = THEME.background.four_hills;
pub const FILL_BORDER: Rgba = THEME.fill.border;
pub const FILL_HIGHLIGHT: Rgba = THEME.fill.highlight;
pub const FILL_TURQUOISE: Rgba = THEME.fill.turquoise;
pub const FILL_LINE: Rgba = THEME.fill.line;
pub const FILL_DIM: Rgba = THEME.fill.dim;
pub const BLACK: Rgba = THEME.black;

pub const JUMPER_SUIT_SOURCE_SHADE_1: u8 = 216;
pub const JUMPER_SUIT_SOURCE_SHADE_3: u8 = 218;
pub const JUMPER_SKI_SOURCE: u8 = 231;

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

const SKI_COLORS: [Rgb6; 4] = [
    Rgb6(63, 63, 32),
    Rgb6(60, 60, 60),
    Rgb6(33, 60, 33),
    Rgb6(63, 43, 43),
];

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

/// Return an arbitrary shade (0..4) of a suit colour as an Rgba.
#[must_use]
pub fn suit_color_shade(col: usize, shade: usize) -> Rgba {
    let rgb = suit_shade_rgba(col)[shade];
    Rgba::from_rgb6(rgb[0], rgb[1], rgb[2])
}

/// Return the ski colour as an Rgba.
#[must_use]
pub fn ski_color(col: usize) -> Rgba {
    let col = col.min(SKI_COLORS.len() - 1);
    SKI_COLORS[col].rgba()
}

#[must_use]
pub fn start_light_recolor(is_dq: bool) -> SpriteColorRecolor {
    if is_dq {
        SpriteColorRecolor::new(vec![
            (253, THEME.recolor.start_dq_lit),
            (254, THEME.recolor.start_dq_dark),
        ])
    } else {
        SpriteColorRecolor::new(vec![
            (253, THEME.recolor.start_ready_lit),
            (254, THEME.recolor.start_ready_dark),
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
    let mut pairs = Vec::with_capacity(5);
    for i in 249..=253 {
        pairs.push((
            i,
            if i == active {
                THEME.recolor.replay_active
            } else {
                THEME.recolor.replay_inactive
            },
        ));
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
    fn theme_background_colors_match_expected_rgb6_values() {
        assert_eq!(BG_LEFT, Rgba::from_rgb6(18, 13, 34));
        assert_eq!(BG_RIGHT, Rgba::from_rgb6(34, 13, 18));
        assert_eq!(FILL_DIM, Rgba::from_rgb6(20, 20, 20));
    }
}

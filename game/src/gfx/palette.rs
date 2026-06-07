use engine::color::Rgba;
use engine::sprite::SpriteColorRecolor;

use crate::components::replay_playback::PlaybackMode;
use crate::gfx::pcx::PcxPalette;

pub const UI_PALETTE_BASE: usize = 216;

pub const STANDARD_UI_PALETTE: [[u8; 3]; 40] = [
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
    [46, 46, 63],
    [32, 32, 63],
    [63, 63, 63],
];

// ---------------------------------------------------------------------------
// RGBA UI color constants (migrated from palette-index legacy)
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
pub const BG_LEFT: Rgba = Rgba::from_rgb6(34, 13, 18);
pub const BG_RIGHT: Rgba = Rgba::from_rgb6(20, 20, 20);
pub const BG_ORDER: Rgba = BG_LEFT;

pub const BG_RIGHT_BRIGHT: Rgba = Rgba::from_rgb6(43, 16, 23);

// Additional fill/text colours from old palette indices
pub const FILL_BORDER: Rgba = Rgba::from_rgb6(23, 16, 43); // 248
pub const FILL_HIGHLIGHT: Rgba = Rgba::from_rgb6(52, 47, 0); // 251
pub const FILL_TURQUOISE: Rgba = Rgba::from_rgb6(0, 47, 52); // 252
pub const FILL_LINE: Rgba = Rgba::from_rgb6(5, 8, 22); // 9
pub const FILL_DIM: Rgba = Rgba::from_rgb6(20, 20, 20); // 244/245
pub const BLACK: Rgba = Rgba::rgb(0, 0, 0);

/// Fill palette slots 216..=255 with the standard UI palette colours.
pub fn apply_standard_ui_palette(palette: &mut PcxPalette) {
    for (i, &rgb) in STANDARD_UI_PALETTE.iter().enumerate() {
        palette.set(UI_PALETTE_BASE + i, rgb);
    }
}

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
mod tests {}

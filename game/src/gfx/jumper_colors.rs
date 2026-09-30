use engine::color::Rgba;

use crate::gfx::color::{Rgb6, Shade};

pub const JUMPER_SUIT_SOURCE_SHADE_1: u8 = 216;
pub const JUMPER_SUIT_SOURCE_SHADE_3: u8 = 218;
pub const JUMPER_BIB_SOURCE_SHADE_1: u8 = 220;
pub const JUMPER_BIB_SOURCE_SHADE_3: u8 = 221;
pub const JUMPER_SKI_SOURCE: u8 = 231;

const BIB_SHADES: [Rgb6; 2] = [Rgb6([49, 45, 0]), Rgb6([34, 31, 0])];

pub struct SuitStyle {
    pub base: Rgb6,
    pub fade_up: bool,
}

pub const SUITS: [SuitStyle; 8] = [
    SuitStyle {
        base: Rgb6([53, 17, 53]),
        fade_up: false,
    },
    SuitStyle {
        base: Rgb6([55, 33, 11]),
        fade_up: false,
    },
    SuitStyle {
        base: Rgb6([11, 48, 18]),
        fade_up: false,
    },
    SuitStyle {
        base: Rgb6([24, 28, 63]),
        fade_up: false,
    },
    SuitStyle {
        base: Rgb6([63, 17, 17]),
        fade_up: false,
    },
    SuitStyle {
        base: Rgb6([33, 33, 33]),
        fade_up: false,
    },
    SuitStyle {
        base: Rgb6([10, 10, 10]),
        fade_up: true,
    },
    SuitStyle {
        base: Rgb6([45, 17, 63]),
        fade_up: false,
    },
];

pub const SKIS: [Rgb6; 4] = [
    Rgb6([63, 63, 32]),
    Rgb6([60, 60, 60]),
    Rgb6([33, 60, 33]),
    Rgb6([63, 43, 43]),
];

const SUIT_FADE_DOWN: [f32; 4] = [1.0, 0.87, 0.75, 0.63];
const SUIT_FADE_UP: [f32; 4] = [1.0, 1.50, 2.00, 2.50];

pub fn shade(base: Rgb6, fade_up: bool, level: Shade) -> Rgba {
    let fade = if fade_up {
        SUIT_FADE_UP
    } else {
        SUIT_FADE_DOWN
    };
    let factor = fade[level as usize];
    Rgba::from_rgb6(
        (factor * f32::from(base.0[0])).round().min(63.0) as u8,
        (factor * f32::from(base.0[1])).round().min(63.0) as u8,
        (factor * f32::from(base.0[2])).round().min(63.0) as u8,
    )
}

pub fn suit_shade_rgb(base: Rgb6, level: Shade) -> Rgba {
    let fade_up = SUITS
        .iter()
        .find(|style| style.base == base)
        .is_some_and(|style| style.fade_up);
    shade(base, fade_up, level)
}

pub fn jumper_bib_color_shade(shade: usize) -> Rgba {
    if shade == 3 {
        BIB_SHADES[1].to_rgba()
    } else {
        BIB_SHADES[0].to_rgba()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuitIdx(pub u8);

impl SuitIdx {
    pub fn rgb(self) -> Rgb6 {
        SUITS[(self.0 as usize).min(SUITS.len() - 1)].base
    }

    pub fn from_rgb(color: Rgb6) -> Self {
        Self(
            SUITS
                .iter()
                .position(|style| style.base == color)
                .unwrap_or(0) as u8,
        )
    }

    pub fn shade(self, level: Shade) -> Rgba {
        let style = &SUITS[(self.0 as usize).min(SUITS.len() - 1)];
        shade(style.base, style.fade_up, level)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkiIdx(pub u8);

impl SkiIdx {
    pub fn rgb(self) -> Rgb6 {
        SKIS[(self.0 as usize).min(SKIS.len() - 1)]
    }

    pub fn from_rgb(color: Rgb6) -> Self {
        Self(SKIS.iter().position(|ski| *ski == color).unwrap_or(0) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shades_match_pascal_loadsuit_table() {
        assert_eq!(SuitIdx(0).shade(Shade::S0), Rgb6([53, 17, 53]).to_rgba());
        assert_eq!(SuitIdx(0).shade(Shade::S1), Rgb6([46, 15, 46]).to_rgba());
        assert_eq!(SuitIdx(0).shade(Shade::S2), Rgb6([40, 13, 40]).to_rgba());
        assert_eq!(SuitIdx(0).shade(Shade::S3), Rgb6([33, 11, 33]).to_rgba());
        assert_eq!(SuitIdx(6).shade(Shade::S0), Rgb6([10, 10, 10]).to_rgba());
        assert_eq!(SuitIdx(6).shade(Shade::S1), Rgb6([15, 15, 15]).to_rgba());
        assert_eq!(SuitIdx(6).shade(Shade::S2), Rgb6([20, 20, 20]).to_rgba());
        assert_eq!(SuitIdx(6).shade(Shade::S3), Rgb6([25, 25, 25]).to_rgba());
    }

    #[test]
    fn suit_index_roundtrips_through_rgb() {
        for idx in 0..8 {
            assert_eq!(SuitIdx::from_rgb(SuitIdx(idx).rgb()).0, idx);
        }
        for idx in 0..4 {
            assert_eq!(SkiIdx::from_rgb(SkiIdx(idx).rgb()).0, idx);
        }
        assert_eq!(SuitIdx::from_rgb(Rgb6([1, 2, 3])).0, 0);
    }
}

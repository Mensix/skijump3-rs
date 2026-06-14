use std::array;

use engine::sprite::{SpriteMaterial, SpriteMaterialId};

use crate::gfx::jumper_colors::{
    self, JUMPER_BIB_SOURCE_SHADE_1, JUMPER_BIB_SOURCE_SHADE_3, JUMPER_SKI_SOURCE,
    JUMPER_SUIT_SOURCE_SHADE_1, JUMPER_SUIT_SOURCE_SHADE_3,
};
use crate::gfx::theme::THEME;
use crate::views::replay::PlaybackMode;

const MATERIAL_JUMPER_BODY: u64 = 0x1000;
const MATERIAL_JUMPER_SKI: u64 = 0x2000;
const MATERIAL_START_LIGHT: u64 = 0x3000;
const MATERIAL_REPLAY_SPEED: u64 = 0x4000;

#[must_use]
pub fn jumper_body_material(suit_color: usize, has_bib: bool) -> SpriteMaterial {
    let bib_shade_1 = if has_bib {
        jumper_colors::jumper_bib_color_shade(1)
    } else {
        jumper_colors::suit_color_shade(suit_color, 1)
    };
    let bib_shade_3 = if has_bib {
        jumper_colors::jumper_bib_color_shade(3)
    } else {
        jumper_colors::suit_color_shade(suit_color, 3)
    };

    let suit_color = suit_color.min(7);
    let id =
        SpriteMaterialId::new(MATERIAL_JUMPER_BODY | suit_color as u64 | ((has_bib as u64) << 8));
    SpriteMaterial::with_id(
        id,
        &[
            (
                JUMPER_SUIT_SOURCE_SHADE_1,
                jumper_colors::suit_color_shade(suit_color, 1),
            ),
            (
                JUMPER_SUIT_SOURCE_SHADE_3,
                jumper_colors::suit_color_shade(suit_color, 3),
            ),
            (JUMPER_BIB_SOURCE_SHADE_1, bib_shade_1),
            (JUMPER_BIB_SOURCE_SHADE_3, bib_shade_3),
        ],
    )
}

#[must_use]
pub fn jumper_ski_material(ski_color: usize) -> SpriteMaterial {
    let ski_color = ski_color.min(3);
    let id = SpriteMaterialId::new(MATERIAL_JUMPER_SKI | ski_color as u64);
    SpriteMaterial::with_id(
        id,
        &[(JUMPER_SKI_SOURCE, jumper_colors::ski_color(ski_color))],
    )
}

#[must_use]
pub fn start_light_material(is_dq: bool) -> SpriteMaterial {
    if is_dq {
        SpriteMaterial::with_id(
            SpriteMaterialId::new(MATERIAL_START_LIGHT | 1),
            &[
                (253, THEME.material.start_dq_lit),
                (254, THEME.material.start_dq_dark),
            ],
        )
    } else {
        SpriteMaterial::with_id(
            SpriteMaterialId::new(MATERIAL_START_LIGHT),
            &[
                (253, THEME.material.start_ready_lit),
                (254, THEME.material.start_ready_dark),
            ],
        )
    }
}

#[must_use]
pub fn replay_speed_material(mode: PlaybackMode) -> SpriteMaterial {
    let active: u8 = match mode {
        PlaybackMode::Forward => 250,
        PlaybackMode::Rewind => 253,
        PlaybackMode::SpeedChange => 251,
        _ => 249,
    };
    let pairs: [(u8, engine::color::Rgba); 5] = array::from_fn(|i| {
        let idx = (249 + i) as u8;
        (
            idx,
            if idx == active {
                THEME.material.replay_active
            } else {
                THEME.material.replay_inactive
            },
        )
    });
    SpriteMaterial::with_id(
        SpriteMaterialId::new(MATERIAL_REPLAY_SPEED | u64::from(active)),
        &pairs,
    )
}

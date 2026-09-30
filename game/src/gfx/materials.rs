use std::array;

use engine::sprite::{SpriteMaterial, SpriteMaterialId};

use crate::gfx::color::{Rgb6, Shade};

use crate::gfx::jumper_colors::{
    self, JUMPER_BIB_SOURCE_SHADE_1, JUMPER_BIB_SOURCE_SHADE_3, JUMPER_SKI_SOURCE,
    JUMPER_SUIT_SOURCE_SHADE_1, JUMPER_SUIT_SOURCE_SHADE_3,
};
use crate::gfx::sprites::Sprite;
use crate::gfx::theme::THEME;
use crate::jump::replay_player::PlaybackMode;
use crate::ui::SpriteMaterialData;

const MATERIAL_JUMPER_BODY: u64 = 0x1000;
const MATERIAL_JUMPER_SKI: u64 = 0x2000;
const MATERIAL_START_LIGHT: u64 = 0x3000;
const MATERIAL_REPLAY_SPEED: u64 = 0x4000;

fn rgb_material_id(base: u64, rgb: Rgb6, extra: u64) -> u64 {
    base | u64::from(rgb.0[0])
        | (u64::from(rgb.0[1]) << 8)
        | (u64::from(rgb.0[2]) << 16)
        | (extra << 24)
}

pub fn jumper_body_material(suit_color: Rgb6, has_bib: bool) -> SpriteMaterialData {
    let bib_shade_1 = if has_bib {
        jumper_colors::jumper_bib_color_shade(1)
    } else {
        jumper_colors::suit_shade_rgb(suit_color, Shade::S1)
    };
    let bib_shade_3 = if has_bib {
        jumper_colors::jumper_bib_color_shade(3)
    } else {
        jumper_colors::suit_shade_rgb(suit_color, Shade::S3)
    };

    let id = rgb_material_id(MATERIAL_JUMPER_BODY, suit_color, u64::from(has_bib));
    SpriteMaterialData {
        id,
        overrides: vec![
            (
                JUMPER_SUIT_SOURCE_SHADE_1,
                jumper_colors::suit_shade_rgb(suit_color, Shade::S1),
            ),
            (
                JUMPER_SUIT_SOURCE_SHADE_3,
                jumper_colors::suit_shade_rgb(suit_color, Shade::S3),
            ),
            (JUMPER_BIB_SOURCE_SHADE_1, bib_shade_1),
            (JUMPER_BIB_SOURCE_SHADE_3, bib_shade_3),
        ],
    }
}

pub fn jumper_ski_material(ski_color: Rgb6) -> SpriteMaterialData {
    let id = rgb_material_id(MATERIAL_JUMPER_SKI, ski_color, 0);
    SpriteMaterialData {
        id,
        overrides: vec![(JUMPER_SKI_SOURCE, ski_color.to_rgba())],
    }
}

pub fn start_light_material(is_dq: bool) -> SpriteMaterialData {
    if is_dq {
        SpriteMaterialData {
            id: MATERIAL_START_LIGHT | 1,
            overrides: vec![
                (253, THEME.material.start_dq_lit),
                (254, THEME.material.start_dq_dark),
            ],
        }
    } else {
        SpriteMaterialData {
            id: MATERIAL_START_LIGHT,
            overrides: vec![
                (253, THEME.material.start_ready_lit),
                (254, THEME.material.start_ready_dark),
            ],
        }
    }
}

pub fn replay_speed_material(mode: PlaybackMode) -> SpriteMaterialData {
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
    SpriteMaterialData {
        id: MATERIAL_REPLAY_SPEED | u64::from(active),
        overrides: pairs.to_vec(),
    }
}

pub fn prebaked_sprite_materials() -> Vec<(u16, SpriteMaterial)> {
    let mut materials = Vec::new();

    // body and ski materials are created on-demand via ensure_material
    for is_dq in [false, true] {
        let material = start_light_material(is_dq);
        materials.push((Sprite::StartLight as u16, to_engine_material(material)));
    }

    for mode in [
        PlaybackMode::Pause,
        PlaybackMode::Forward,
        PlaybackMode::Rewind,
        PlaybackMode::PlayOnceThenPause,
        PlaybackMode::SpeedChange,
        PlaybackMode::OneStep,
    ] {
        let material = replay_speed_material(mode);
        materials.push((Sprite::ReplayModeIcon as u16, to_engine_material(material)));
    }

    materials
}

pub fn to_engine_material(material: SpriteMaterialData) -> SpriteMaterial {
    SpriteMaterial::with_id(SpriteMaterialId::new(material.id), &material.overrides)
}

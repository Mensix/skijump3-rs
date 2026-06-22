use crate::gfx::materials;
use crate::gfx::sprites;
use engine::consts::{HEIGHT, WIDTH};
use engine::oxide::{ImageRegionDraw, PaintCx};
use std::rc::Rc;

pub(crate) fn push_viewport(cx: &mut PaintCx<'_>, viewport: &Rc<[u8]>) {
    cx.image_region(ImageRegionDraw {
        pixels: Rc::clone(viewport),
        src_w: WIDTH,
        src_h: HEIGHT,
        src_x: 0,
        src_y: 0,
        dst_x: 0,
        dst_y: 0,
        w: WIDTH,
        h: HEIGHT,
    });
}

pub(crate) fn push_hill_record_marker(
    cx: &mut PaintCx<'_>,
    marker: Option<(i32, i32)>,
    sx: i32,
    sy: i32,
) {
    if let Some((hr_x, hr_y)) = marker {
        cx.sprite(
            sprites::Sprite::HillRecordMarker as u16,
            (hr_x - sx, hr_y - sy),
        );
    }
}

pub(crate) fn push_goal_marker(cx: &mut PaintCx<'_>, marker: Option<(i32, i32)>, sx: i32, sy: i32) {
    if let Some((goal_x, goal_y)) = marker {
        cx.sprite(
            sprites::Sprite::StartLight as u16,
            (goal_x - sx, goal_y - sy),
        );
    }
}

pub(crate) struct JumperSpriteSpec {
    pub(crate) body_anim: u16,
    pub(crate) ski_anim: u16,
    pub(crate) body_x: i32,
    pub(crate) body_y: i32,
    pub(crate) ski_x: i32,
    pub(crate) ski_y: i32,
    pub(crate) suit_color: [u8; 3],
    pub(crate) ski_color: [u8; 3],
    pub(crate) has_bib: bool,
}

pub(crate) fn push_jumper_sprites(cx: &mut PaintCx<'_>, spec: JumperSpriteSpec) {
    cx.sprite_with_material(
        spec.body_anim,
        (spec.body_x, spec.body_y),
        materials::jumper_body_material(spec.suit_color, spec.has_bib),
    );
    cx.sprite_with_material(
        spec.ski_anim,
        (spec.ski_x, spec.ski_y),
        materials::jumper_ski_material(spec.ski_color),
    );
}

#[cfg(test)]
mod tests {
    use crate::gfx::jumper_colors::{
        self, JUMPER_BIB_SOURCE_SHADE_1, JUMPER_BIB_SOURCE_SHADE_3, JUMPER_SKI_SOURCE,
        JUMPER_SUIT_SOURCE_SHADE_1, JUMPER_SUIT_SOURCE_SHADE_3,
    };

    use super::*;

    const SUIT_0: [u8; 3] = [53, 17, 53];
    const SKI_0: [u8; 3] = [63, 63, 32];

    #[test]
    fn standard_jumper_material_pairs() {
        let body_material = materials::jumper_body_material(SUIT_0, false);
        let ski_material = materials::jumper_ski_material(SKI_0);

        assert_eq!(
            body_material.get(JUMPER_SUIT_SOURCE_SHADE_1),
            Some(jumper_colors::suit_color_shade_rgb(SUIT_0, 1))
        );
        assert_eq!(
            body_material.get(JUMPER_SUIT_SOURCE_SHADE_3),
            Some(jumper_colors::suit_color_shade_rgb(SUIT_0, 3))
        );
        assert_eq!(
            body_material.get(JUMPER_BIB_SOURCE_SHADE_1),
            Some(jumper_colors::suit_color_shade_rgb(SUIT_0, 1))
        );
        assert_eq!(
            body_material.get(JUMPER_BIB_SOURCE_SHADE_3),
            Some(jumper_colors::suit_color_shade_rgb(SUIT_0, 3))
        );
        assert_eq!(
            body_material.get(0),
            None,
            "transparent has no material override"
        );
        assert_eq!(
            body_material.get(JUMPER_SKI_SOURCE),
            None,
            "ski index not in body material"
        );

        assert_eq!(
            ski_material.get(JUMPER_SKI_SOURCE),
            Some(jumper_colors::ski_color_rgb(SKI_0))
        );
        assert_eq!(
            ski_material.get(JUMPER_SUIT_SOURCE_SHADE_1),
            None,
            "suit index not in ski material"
        );
    }

    #[test]
    fn leader_bib_material_uses_pascal_bib_palette_entries() {
        let body_material = materials::jumper_body_material(SUIT_0, true);

        assert_eq!(
            body_material.get(JUMPER_BIB_SOURCE_SHADE_1),
            Some(jumper_colors::jumper_bib_color_shade(1))
        );
        assert_eq!(
            body_material.get(JUMPER_BIB_SOURCE_SHADE_3),
            Some(jumper_colors::jumper_bib_color_shade(3))
        );
        assert_ne!(
            body_material.get(JUMPER_BIB_SOURCE_SHADE_1),
            Some(jumper_colors::suit_color_shade_rgb(SUIT_0, 1))
        );
    }
}

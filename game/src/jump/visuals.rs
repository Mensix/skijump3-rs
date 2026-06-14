use crate::gfx::palette::{
    self, JUMPER_SKI_SOURCE, JUMPER_SUIT_SOURCE_SHADE_1, JUMPER_SUIT_SOURCE_SHADE_3,
};
use crate::gfx::sprites;
use engine::color::Rgba;
use engine::consts::{HEIGHT, WIDTH};
use engine::sprite::SpriteColorRecolor;
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

pub(crate) struct JumperSpriteSpec {
    pub(crate) body_anim: u16,
    pub(crate) ski_anim: u16,
    pub(crate) body_x: i32,
    pub(crate) body_y: i32,
    pub(crate) ski_x: i32,
    pub(crate) ski_y: i32,
    pub(crate) suit_color: usize,
    pub(crate) ski_color: usize,
    pub(crate) has_bib: bool,
}

pub(crate) fn push_jumper_sprites(cx: &mut PaintCx<'_>, spec: JumperSpriteSpec) {
    let body_recolor = SpriteColorRecolor::new(vec![
        (
            JUMPER_SUIT_SOURCE_SHADE_1,
            palette::suit_color_shade(spec.suit_color, 1),
        ),
        (
            JUMPER_SUIT_SOURCE_SHADE_3,
            palette::suit_color_shade(spec.suit_color, 3),
        ),
    ]);
    let ski_recolor = SpriteColorRecolor::new(vec![(
        JUMPER_SKI_SOURCE,
        palette::ski_color(spec.ski_color),
    )]);
    cx.sprite_remapped(
        spec.body_anim,
        (spec.body_x, spec.body_y),
        body_recolor,
    );
    cx.sprite_remapped(
        spec.ski_anim,
        (spec.ski_x, spec.ski_y),
        ski_recolor,
    );

    // Leader bib: yellow rectangle on the jumper body
    if spec.has_bib {
        cx.fill(
            (spec.body_x + 8, spec.body_y + 4, 6, 6),
            Rgba::from_rgb6(63, 57, 9),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_jumper_recolor_pairs() {
        let suit_color = 0usize;
        let ski_color = 0usize;
        let body_recolor = SpriteColorRecolor::new(vec![
            (
                JUMPER_SUIT_SOURCE_SHADE_1,
                palette::suit_color_shade(suit_color, 1),
            ),
            (
                JUMPER_SUIT_SOURCE_SHADE_3,
                palette::suit_color_shade(suit_color, 3),
            ),
        ]);
        let ski_recolor =
            SpriteColorRecolor::new(vec![(JUMPER_SKI_SOURCE, palette::ski_color(ski_color))]);

        assert_eq!(
            body_recolor.get(JUMPER_SUIT_SOURCE_SHADE_1),
            Some(palette::suit_color_shade(0, 1))
        );
        assert_eq!(
            body_recolor.get(JUMPER_SUIT_SOURCE_SHADE_3),
            Some(palette::suit_color_shade(0, 3))
        );
        assert_eq!(body_recolor.get(0), None, "transparent not recolored");
        assert_eq!(
            body_recolor.get(JUMPER_SKI_SOURCE),
            None,
            "ski index not in body recolor"
        );

        assert_eq!(
            ski_recolor.get(JUMPER_SKI_SOURCE),
            Some(palette::ski_color(ski_color))
        );
        assert_eq!(
            ski_recolor.get(JUMPER_SUIT_SOURCE_SHADE_1),
            None,
            "suit index not in ski recolor"
        );
    }
}

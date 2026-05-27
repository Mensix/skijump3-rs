use engine::consts::{FILL_RANGE_MAX, SHADOW_PIXEL};
use engine::ui::Element;

pub(crate) struct DitherRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: u8,
    pub is_box: bool,
}

pub(crate) fn count_fill_areas(elements: &[Element]) -> usize {
    elements.iter().map(count_fill_areas_in_element).sum()
}

fn count_fill_areas_in_element(element: &Element) -> usize {
    match element {
        Element::FillArea { .. } => 1,
        Element::Container(children) => count_fill_areas(children),
        _ => 0,
    }
}

pub(crate) fn is_fill_area_dither_color(color: u8) -> bool {
    color > SHADOW_PIXEL && color <= FILL_RANGE_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_fill_areas_empty() {
        assert_eq!(count_fill_areas(&[]), 0);
    }

    #[test]
    fn count_fill_areas_no_fillareas() {
        let els = vec![
            Element::fillbox(0, 0, 10, 10, 1),
            Element::text("hi", 0, 0, 2, false),
        ];
        assert_eq!(count_fill_areas(&els), 0);
    }

    #[test]
    fn count_fill_areas_single() {
        let els = vec![Element::fill_area(64)];
        assert_eq!(count_fill_areas(&els), 1);
    }

    #[test]
    fn count_fill_areas_nested_in_containers() {
        let els = vec![
            Element::container(vec![
                Element::fill_area(63),
                Element::container(vec![
                    Element::fill_area(63),
                    Element::fillbox(0, 0, 10, 10, 1),
                ]),
            ]),
            Element::fill_area(64),
        ];
        assert_eq!(count_fill_areas(&els), 3);
    }

    #[test]
    fn count_fill_areas_mixed() {
        let els = vec![
            Element::fillbox(0, 0, 10, 10, 1),
            Element::text("test", 0, 0, 2, false),
            Element::fill_area(63),
            Element::sprite(0, 0, 0),
        ];
        assert_eq!(count_fill_areas(&els), 1);
    }

    #[test]
    fn dither_rect_tracks_eligible_rects_in_pending() {
        let dr = DitherRect {
            x: 10,
            y: 20,
            w: 30,
            h: 40,
            color: 243,
            is_box: false,
        };
        assert_eq!(dr.x, 10);
        assert_eq!(dr.y, 20);
        assert_eq!(dr.w, 30);
        assert_eq!(dr.h, 40);
        assert_eq!(dr.color, 243);
        assert!(!dr.is_box);
    }

    #[test]
    fn dither_rect_box_default_is_box() {
        let dr = DitherRect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
            color: 244,
            is_box: true,
        };
        assert!(dr.is_box);
        assert_eq!(dr.color, 244);
    }

    #[test]
    fn is_fill_area_dither_color_eligible() {
        assert!(is_fill_area_dither_color(243));
        assert!(is_fill_area_dither_color(244));
        assert!(is_fill_area_dither_color(245));
    }

    #[test]
    fn is_fill_area_dither_color_ineligible() {
        assert!(!is_fill_area_dither_color(0));
        assert!(!is_fill_area_dither_color(100));
        assert!(!is_fill_area_dither_color(SHADOW_PIXEL));
        assert!(!is_fill_area_dither_color(FILL_RANGE_MAX + 1));
        assert!(!is_fill_area_dither_color(255));
    }

    #[test]
    fn is_fill_area_dither_color_boundaries() {
        assert!(is_fill_area_dither_color(SHADOW_PIXEL + 1));
        assert!(!is_fill_area_dither_color(SHADOW_PIXEL));
        assert!(is_fill_area_dither_color(FILL_RANGE_MAX));
        assert!(!is_fill_area_dither_color(FILL_RANGE_MAX + 1));
    }
}

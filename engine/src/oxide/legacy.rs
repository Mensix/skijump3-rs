use crate::oxide::draw::ImageRegionDraw;
use crate::oxide::input::UiEvent;
use crate::oxide::paint::PaintCx;
use crate::ui::{Element, Event, ImageRegion, Key};

pub fn event_from_ui(event: UiEvent) -> Option<Event> {
    match event {
        UiEvent::KeyDown(key) => Some(Event::Keyboard(key)),
        UiEvent::Text(c) => Some(Event::Keyboard(Key::Char(c))),
        UiEvent::Quit | UiEvent::Tick => None,
    }
}

pub fn paint_elements(cx: &mut PaintCx<'_>, elements: &[Element]) {
    for element in elements {
        paint_element(cx, element);
    }
}

fn paint_element(cx: &mut PaintCx<'_>, element: &Element) {
    match element {
        Element::Image(pixels, w, h) => cx.image(pixels.clone(), *w, *h),
        Element::ImageRegion(region) => cx.image_region(image_region_from_legacy(region)),
        Element::Text {
            text,
            x,
            y,
            color,
            right,
            center,
        } => {
            if *center {
                cx.center_text((*x, *y), *color, text);
            } else if *right {
                cx.right_text((*x, *y), *color, text);
            } else {
                cx.text((*x, *y), *color, text);
            }
        }
        Element::Sprite(idx, x, y) => cx.sprite(*idx, (*x, *y)),
        Element::Fillbox { x, y, w, h, color } => cx.fill((*x, *y, *w, *h), *color),
        Element::FillArea { thing } => cx.dither_fill(*thing),
        Element::Box { x, y, w, h, color } => cx.stroke((*x, *y, *w, *h), *color),
        Element::SpriteRemapped(idx, x, y, recolor) => {
            cx.sprite_remapped(*idx, (*x, *y), recolor.clone());
        }
        Element::Container(children) => paint_elements(cx, children),
    }
}

fn image_region_from_legacy(region: &ImageRegion) -> ImageRegionDraw {
    ImageRegionDraw {
        pixels: region.pixels.clone(),
        src_w: region.src_w,
        src_h: region.src_h,
        src_x: region.src_x,
        src_y: region.src_y,
        dst_x: region.dst_x,
        dst_y: region.dst_y,
        w: region.w,
        h: region.h,
    }
}

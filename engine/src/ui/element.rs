use crate::sprite::SpriteColorRemap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ImageRegion {
    pub pixels: Rc<[u8]>,
    pub src_w: u32,
    pub src_h: u32,
    pub src_x: i32,
    pub src_y: i32,
    pub dst_x: i32,
    pub dst_y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone)]
pub enum Element {
    Image(Rc<[u8]>, u32, u32),
    ImageRegion(ImageRegion),
    Text {
        text: String,
        x: i32,
        y: i32,
        color: u8,
        right: bool,
        center: bool,
    },
    Sprite(u16, i32, i32),
    Fillbox {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: u8,
    },
    FillArea {
        thing: u8,
    },
    Box {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: u8,
    },
    SpriteRemapped(u16, i32, i32, SpriteColorRemap),
    Container(Vec<Element>),
}

impl Element {
    pub fn text(text: impl Into<String>, x: i32, y: i32, color: u8, right: bool) -> Self {
        Self::Text {
            text: text.into(),
            x,
            y,
            color,
            right,
            center: false,
        }
    }

    pub fn right_text(text: impl Into<String>, x: i32, y: i32, color: u8) -> Self {
        Self::Text {
            text: text.into(),
            x,
            y,
            color,
            right: true,
            center: false,
        }
    }

    pub fn center_text(text: impl Into<String>, x: i32, y: i32, color: u8) -> Self {
        Self::Text {
            text: text.into(),
            x,
            y,
            color,
            right: false,
            center: true,
        }
    }

    pub fn sprite(idx: u16, x: i32, y: i32) -> Self {
        Self::Sprite(idx, x, y)
    }

    pub fn sprite_remapped(idx: u16, x: i32, y: i32, remap: SpriteColorRemap) -> Self {
        Self::SpriteRemapped(idx, x, y, remap)
    }

    pub fn image(pixels: impl Into<Rc<[u8]>>, w: u32, h: u32) -> Self {
        Self::Image(pixels.into(), w, h)
    }

    pub fn image_region(region: ImageRegion) -> Self {
        Self::ImageRegion(region)
    }

    pub fn fillbox(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self::Fillbox { x, y, w, h, color }
    }

    pub fn box_(x: i32, y: i32, w: i32, h: i32, color: u8) -> Self {
        Self::Box { x, y, w, h, color }
    }

    pub fn fill_area(thing: u8) -> Self {
        Self::FillArea { thing }
    }

    pub fn container(children: Vec<Element>) -> Self {
        Self::Container(children)
    }
}

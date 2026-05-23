use crate::sprite::SpriteData;
use crate::ui::paint::PaintCtx;
use crate::ui::Font;
use std::rc::Rc;

use crate::consts::{FILL_BRIGHTEN, FILL_RANGE_MAX, PATTERN_SPRITE, SHADOW_PIXEL, TILE_H, TILE_W};

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
    Container(Vec<Element>),
}

impl Element {
    pub fn render(&self, ctx: &mut PaintCtx, font: &Font, sprites: &[SpriteData]) {
        match self {
            Element::Image(pixels, w, h) => {
                let dst_w = ctx.width.min(*w);
                let dst_h = ctx.height.min(*h);
                for y in 0..dst_h {
                    let src_row = (y as usize) * (*w as usize);
                    let dst_row = (y as usize) * (ctx.width as usize);
                    let src = &pixels[src_row..src_row + dst_w as usize];
                    ctx.pixels[dst_row..dst_row + dst_w as usize].copy_from_slice(src);
                }
            }
            Element::ImageRegion(region) => {
                for y in 0..region.h as i32 {
                    let sy = region.src_y + y;
                    let dy = region.dst_y + y;
                    if sy < 0 || dy < 0 || sy >= region.src_h as i32 || dy >= ctx.height as i32 {
                        continue;
                    }
                    for x in 0..region.w as i32 {
                        let sx = region.src_x + x;
                        let dx = region.dst_x + x;
                        if sx < 0 || dx < 0 || sx >= region.src_w as i32 || dx >= ctx.width as i32 {
                            continue;
                        }
                        let src_idx = sy as usize * region.src_w as usize + sx as usize;
                        let dst_idx = dy as usize * ctx.width as usize + dx as usize;
                        if let Some(&pixel) = region.pixels.get(src_idx) {
                            ctx.pixels[dst_idx] = pixel;
                        }
                    }
                }
            }
            Element::Text {
                text,
                x,
                y,
                color,
                right,
                center,
            } => {
                let text_w = font.string_width(text) as i32;
                let fx = if *center {
                    x - text_w / 2
                } else if *right {
                    x - text_w
                } else {
                    *x
                };
                font.blit_string_color(ctx.pixels, ctx.width, text, fx, *y, *color);
            }
            Element::Fillbox { x, y, w, h, color } => {
                ctx.fill_rect(*x, *y, *w, *h, *color);
            }
            Element::FillArea { thing } => {
                let Some(pattern) = sprites.get(PATTERN_SPRITE) else {
                    return;
                };
                for py in 0..ctx.height {
                    for px in 0..ctx.width {
                        let cur = ctx.pixels[(py as usize) * (ctx.width as usize) + (px as usize)];
                        if cur <= SHADOW_PIXEL || cur > FILL_RANGE_MAX {
                            continue;
                        }
                        let (ax, ay) = if *thing == 64 {
                            ((px + 2) % TILE_W, (py + 7) % TILE_H)
                        } else {
                            (px % TILE_W, py % TILE_H)
                        };
                        let pi = (ay * TILE_W + ax) as usize;
                        if pi < pattern.data.len() && pattern.data[pi] != 0 {
                            let idx = (py as usize) * (ctx.width as usize) + (px as usize);
                            ctx.pixels[idx] = cur + FILL_BRIGHTEN;
                        }
                    }
                }
            }
            Element::Box { x, y, w, h, color } => {
                ctx.fill_rect(*x, *y, *w, 1, *color);
                ctx.fill_rect(*x, *y + *h - 1, *w, 1, *color);
                ctx.fill_rect(*x, *y, 1, *h, *color);
                ctx.fill_rect(*x + *w - 1, *y, 1, *h, *color);
            }
            Element::Container(children) => {
                for child in children {
                    child.render(ctx, font, sprites);
                }
            }
            Element::Sprite(idx, x, y) => {
                if let Some(s) = sprites.get(*idx as usize) {
                    s.blit_to(ctx.pixels, ctx.width, *x, *y);
                }
            }
        }
    }

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

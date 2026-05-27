use crate::sprite::SpriteColorRemap;
use std::rc::Rc;

use crate::bitmap::IndexedBitmap;
use crate::consts::{HEIGHT, WIDTH};

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

/// Render an Element::Image as an IndexedBitmap suitable for GPU overlay.
/// The image is placed at (0, 0) and clipped to the screen.
pub fn render_image_bitmap(pixels: &[u8], image_w: u32, image_h: u32) -> Option<IndexedBitmap> {
    let w = image_w.min(WIDTH);
    let h = image_h.min(HEIGHT);
    if w == 0 || h == 0 {
        return None;
    }
    let mut bitmap_pixels = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        let src_row = (y as usize) * (image_w as usize);
        let end = (src_row + w as usize).min(pixels.len());
        bitmap_pixels.extend_from_slice(&pixels[src_row..end]);
        bitmap_pixels.resize(bitmap_pixels.len() + (w as usize - (end - src_row)), 0);
    }
    Some(IndexedBitmap {
        pixels: bitmap_pixels,
        x: 0,
        y: 0,
        width: w,
        height: h,
    })
}

/// Render an Element::ImageRegion as an IndexedBitmap.
/// Clips to screen boundaries and translates to destination coordinates.
pub fn render_image_region_bitmap(region: &ImageRegion) -> Option<IndexedBitmap> {
    let dst_left = region.dst_x.max(0);
    let dst_top = region.dst_y.max(0);
    let dst_right = (region.dst_x + region.w as i32).min(WIDTH as i32);
    let dst_bottom = (region.dst_y + region.h as i32).min(HEIGHT as i32);
    let vis_w = (dst_right - dst_left).max(0) as u32;
    let vis_h = (dst_bottom - dst_top).max(0) as u32;

    if vis_w == 0 || vis_h == 0 {
        return None;
    }

    let src_off_x = dst_left - region.dst_x;
    let src_off_y = dst_top - region.dst_y;

    let mut pixels = vec![0u8; (vis_w * vis_h) as usize];
    for dy in 0..vis_h as i32 {
        let sy = region.src_y + src_off_y + dy;
        if sy < 0 || sy >= region.src_h as i32 {
            continue;
        }
        for dx in 0..vis_w as i32 {
            let sx = region.src_x + src_off_x + dx;
            if sx < 0 || sx >= region.src_w as i32 {
                continue;
            }
            let src_idx = (sy as usize) * (region.src_w as usize) + (sx as usize);
            if let Some(&pixel) = region.pixels.get(src_idx) {
                let di = (dy as usize) * (vis_w as usize) + (dx as usize);
                pixels[di] = pixel;
            }
        }
    }

    Some(IndexedBitmap {
        pixels,
        x: dst_left,
        y: dst_top,
        width: vis_w,
        height: vis_h,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_image_bitmap_small() {
        let w = 10u32;
        let h = 10u32;
        let pixels: Vec<u8> = (0..(w * h)).map(|i| (i % 256) as u8).collect();
        let result = render_image_bitmap(&pixels, w, h);
        assert!(result.is_some());
        let bm = result.unwrap();
        assert_eq!(bm.width, 10);
        assert_eq!(bm.height, 10);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.y, 0);
        assert_eq!(bm.pixels, pixels);
    }

    #[test]
    fn test_render_image_bitmap_wider_than_screen() {
        let image_w = WIDTH + 50;
        let image_h = 10u32;
        let pixels: Vec<u8> = (0..(image_w * image_h)).map(|i| (i % 256) as u8).collect();
        let result = render_image_bitmap(&pixels, image_w, image_h);
        assert!(result.is_some());
        let bm = result.unwrap();
        assert_eq!(bm.width, WIDTH);
        assert_eq!(bm.height, 10);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.y, 0);
        for y in 0..10usize {
            let src_start = y * (image_w as usize);
            let bm_start = y * (WIDTH as usize);
            assert_eq!(
                bm.pixels[bm_start..bm_start + WIDTH as usize],
                pixels[src_start..src_start + WIDTH as usize]
            );
        }
    }

    #[test]
    fn test_render_image_bitmap_taller_than_screen() {
        let image_w = 10u32;
        let image_h = HEIGHT + 50;
        let pixels: Vec<u8> = (0..(image_w * image_h)).map(|i| (i % 256) as u8).collect();
        let result = render_image_bitmap(&pixels, image_w, image_h);
        assert!(result.is_some());
        let bm = result.unwrap();
        assert_eq!(bm.width, 10);
        assert_eq!(bm.height, HEIGHT);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.y, 0);
    }

    #[test]
    fn test_render_image_bitmap_zero() {
        assert!(render_image_bitmap(&[], 0, 0).is_none());
        assert!(render_image_bitmap(&[1, 2, 3], 0, 5).is_none());
        assert!(render_image_bitmap(&[1, 2, 3], 5, 0).is_none());
    }

    #[test]
    fn test_render_image_region_bitmap_basic() {
        let src_w = 20u32;
        let src_h = 20u32;
        let pixels: Vec<u8> = (0..(src_w * src_h)).map(|i| (i % 256) as u8).collect();
        let region = ImageRegion {
            pixels: pixels.into(),
            src_w,
            src_h,
            src_x: 0,
            src_y: 0,
            dst_x: 5,
            dst_y: 5,
            w: 10,
            h: 10,
        };
        let result = render_image_region_bitmap(&region);
        assert!(result.is_some());
        let bm = result.unwrap();
        assert_eq!(bm.width, 10);
        assert_eq!(bm.height, 10);
        assert_eq!(bm.x, 5);
        assert_eq!(bm.y, 5);
        assert_eq!(bm.pixels[0], region.pixels[0]);
    }

    #[test]
    fn test_render_image_region_bitmap_partial_clip() {
        let src_w = 20u32;
        let src_h = 20u32;
        let pixels: Vec<u8> = (0..(src_w * src_h)).map(|i| (i % 256) as u8).collect();
        let region = ImageRegion {
            pixels: pixels.clone().into(),
            src_w,
            src_h,
            src_x: 5,
            src_y: 5,
            dst_x: -3,
            dst_y: -2,
            w: 10,
            h: 10,
        };
        let result = render_image_region_bitmap(&region);
        assert!(result.is_some());
        let bm = result.unwrap();
        assert_eq!(bm.width, 7);
        assert_eq!(bm.height, 8);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.y, 0);
        let expected_src_idx = (7usize) * (src_w as usize) + 8usize;
        assert_eq!(bm.pixels[0], pixels[expected_src_idx]);
    }

    #[test]
    fn test_render_image_region_bitmap_fully_offscreen() {
        let pixels: Vec<u8> = vec![0u8; 100];
        let region = ImageRegion {
            pixels: pixels.into(),
            src_w: 10,
            src_h: 10,
            src_x: 0,
            src_y: 0,
            dst_x: 400,
            dst_y: 300,
            w: 10,
            h: 10,
        };
        assert!(render_image_region_bitmap(&region).is_none());
    }

    #[test]
    fn test_render_image_region_src_oob() {
        let src_w = 5u32;
        let src_h = 5u32;
        let pixels: Vec<u8> = vec![1u8; 25];
        let region = ImageRegion {
            pixels: pixels.into(),
            src_w,
            src_h,
            src_x: 10,
            src_y: 0,
            dst_x: 0,
            dst_y: 0,
            w: 5,
            h: 5,
        };
        let result = render_image_region_bitmap(&region);
        assert!(result.is_some());
        let bm = result.unwrap();
        assert!(bm.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn test_render_image_bitmap_shorter_row() {
        let pixels = vec![10u8, 20, 30];
        let result = render_image_bitmap(&pixels, 10, 1);
        assert!(result.is_some());
        let bm = result.unwrap();
        assert_eq!(bm.width, 10);
        assert_eq!(bm.height, 1);
        assert_eq!(bm.pixels[0], 10);
        assert_eq!(bm.pixels[1], 20);
        assert_eq!(bm.pixels[2], 30);
        assert_eq!(bm.pixels[3], 0);
        assert_eq!(bm.pixels[9], 0);
    }
}

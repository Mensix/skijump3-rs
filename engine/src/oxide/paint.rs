use crate::color::Rgba;
use crate::oxide::draw::{
    CommandBuffer, DrawCommand, ImageRegionDraw, Point, Rect, SpriteDraw, TextAlign, TextRun,
};
use crate::sprite::SpriteColorRecolor;
use std::rc::Rc;

pub struct PaintCx<'a> {
    commands: &'a mut CommandBuffer,
}

impl<'a> PaintCx<'a> {
    pub fn new(commands: &'a mut CommandBuffer) -> Self {
        Self { commands }
    }

    pub fn fill(&mut self, rect: impl Into<Rect>, color: Rgba) {
        self.commands.push(DrawCommand::Fill(rect.into(), color));
    }

    pub fn stroke(&mut self, rect: impl Into<Rect>, color: Rgba) {
        self.commands.push(DrawCommand::Stroke(rect.into(), color));
    }

    pub fn dither_fill(&mut self, thing: u8) {
        self.commands.push(DrawCommand::DitherFill(thing));
    }

    pub fn text(&mut self, position: impl Into<Point>, color: Rgba, text: impl Into<String>) {
        self.text_aligned(position, color, text, TextAlign::Left);
    }

    pub fn right_text(&mut self, position: impl Into<Point>, color: Rgba, text: impl Into<String>) {
        self.text_aligned(position, color, text, TextAlign::Right);
    }

    pub fn center_text(
        &mut self,
        position: impl Into<Point>,
        color: Rgba,
        text: impl Into<String>,
    ) {
        self.text_aligned(position, color, text, TextAlign::Center);
    }

    pub fn text_aligned(
        &mut self,
        position: impl Into<Point>,
        color: Rgba,
        text: impl Into<String>,
        align: TextAlign,
    ) {
        self.commands.push(DrawCommand::Text(TextRun {
            text: text.into(),
            position: position.into(),
            color,
            align,
        }));
    }

    pub fn sprite(&mut self, idx: u16, position: impl Into<Point>) {
        self.commands.push(DrawCommand::Sprite(SpriteDraw {
            idx,
            position: position.into(),
        }));
    }

    pub fn sprite_remapped(
        &mut self,
        idx: u16,
        position: impl Into<Point>,
        recolor: SpriteColorRecolor,
    ) {
        self.commands.push(DrawCommand::SpriteRemapped {
            sprite: SpriteDraw {
                idx,
                position: position.into(),
            },
            recolor,
        });
    }

    pub fn image(&mut self, pixels: impl Into<Rc<[u8]>>, w: u32, h: u32) {
        self.commands.push(DrawCommand::Image {
            pixels: pixels.into(),
            w,
            h,
        });
    }

    pub fn image_region(&mut self, region: ImageRegionDraw) {
        self.commands.push(DrawCommand::ImageRegion(region));
    }
}

use crate::color::Rgba;
use crate::oxide::draw::{
    CommandBuffer, DrawCommand, ImageRegionDraw, Point, Rect, SpriteDraw, TextAlign, TextRun,
};
use crate::oxide::text::Font;
use crate::sprite::SpriteMaterial;
use std::rc::Rc;

pub struct PaintCx<'a> {
    commands: &'a mut CommandBuffer,
    font: &'a Font,
}

impl<'a> PaintCx<'a> {
    pub fn new(commands: &'a mut CommandBuffer, font: &'a Font) -> Self {
        Self { commands, font }
    }

    pub fn string_width(&self, text: &str) -> u32 {
        self.font.string_width(text)
    }

    pub fn fill(&mut self, rect: impl Into<Rect>, color: Rgba) {
        self.commands.push(DrawCommand::Fill(rect.into(), color));
    }

    pub fn stroke(&mut self, rect: impl Into<Rect>, color: Rgba) {
        self.commands.push(DrawCommand::Stroke(rect.into(), color));
    }

    pub fn pattern_fill(&mut self, rect: impl Into<Rect>, color: Rgba) {
        self.commands
            .push(DrawCommand::PatternFill(rect.into(), color));
    }

    pub fn pattern_stroke(&mut self, rect: impl Into<Rect>, color: Rgba) {
        self.commands
            .push(DrawCommand::PatternStroke(rect.into(), color));
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

    pub fn sprite_with_material(
        &mut self,
        idx: u16,
        position: impl Into<Point>,
        material: SpriteMaterial,
    ) {
        self.commands.push(DrawCommand::SpriteWithMaterial {
            sprite: SpriteDraw {
                idx,
                position: position.into(),
            },
            material,
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

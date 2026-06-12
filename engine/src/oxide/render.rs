use crate::element_renderer::ElementRenderContext;
use crate::oxide::draw::{CommandBuffer, DrawCommand, ImageRegionDraw, TextAlign};
use crate::sprite::SpriteData;
use crate::ui::{Element, Font, ImageRegion};
use crate::video::{Renderer, TextureId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    None,
    Texture(TextureId),
}

pub struct OxideRenderer {
    element_renderer: ElementRenderContext,
    element_buffer: Vec<Element>,
}

impl Default for OxideRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl OxideRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            element_renderer: ElementRenderContext::new(),
            element_buffer: Vec::new(),
        }
    }

    pub fn render_commands(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        commands: &CommandBuffer,
        background: Background,
    ) -> Result<(), String> {
        self.element_buffer.clear();
        self.element_buffer.extend(
            commands
                .commands()
                .iter()
                .cloned()
                .map(draw_command_to_element),
        );
        self.render_element_buffer(renderer, font, sprites, background)
    }

    fn render_element_buffer(
        &mut self,
        renderer: &mut Renderer,
        font: &Font,
        sprites: &[SpriteData],
        background: Background,
    ) -> Result<(), String> {
        let texture = match background {
            Background::None => None,
            Background::Texture(texture) => Some(texture),
        };
        self.element_renderer
            .render_frame(renderer, font, sprites, &self.element_buffer, texture)
    }
}

pub fn draw_command_to_element(command: DrawCommand) -> Element {
    match command {
        DrawCommand::Image { pixels, w, h } => Element::Image(pixels, w, h),
        DrawCommand::ImageRegion(region) => Element::ImageRegion(image_region_to_element(region)),
        DrawCommand::Text(run) => Element::Text {
            text: run.text,
            x: run.position.x,
            y: run.position.y,
            color: run.color,
            right: run.align == TextAlign::Right,
            center: run.align == TextAlign::Center,
        },
        DrawCommand::Sprite(sprite) => {
            Element::Sprite(sprite.idx, sprite.position.x, sprite.position.y)
        }
        DrawCommand::SpriteRemapped { sprite, recolor } => {
            Element::SpriteRemapped(sprite.idx, sprite.position.x, sprite.position.y, recolor)
        }
        DrawCommand::Fill(rect, color) => Element::Fillbox {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            color,
        },
        DrawCommand::Stroke(rect, color) => Element::Box {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            color,
        },
        DrawCommand::DitherFill(thing) => Element::FillArea { thing },
    }
}

fn image_region_to_element(region: ImageRegionDraw) -> ImageRegion {
    ImageRegion {
        pixels: region.pixels,
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
